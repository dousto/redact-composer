use std::any::{type_name, TypeId};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::successors;
use std::marker::PhantomData;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::{Bound, RangeBounds};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use twox_hash::XxHash64;

use crate::render::RenderSegment;
use crate::render::{
    tree::{Node, Tree},
    Result,
};
use crate::timing::RangeOps;
use crate::SegmentRef;
use crate::{CompositionOptions, Element};

use crate::error::RendererError::MissingContext;
use crate::render::context::TimingRelation::*;

#[cfg(test)]
mod test;

/// Provides access to common utilities, such as methods to lookup other composition tree nodes, or
/// Rng.
///
/// This struct is provided as an argument to [`Renderer::render`](crate::render::Renderer::render).
#[derive(Debug)]
pub struct CompositionContext<'a> {
    pub(crate) options: &'a CompositionOptions,
    pub(crate) tree: &'a Tree<RenderSegment>,
    pub(crate) start: &'a Node<RenderSegment>,
    pub(crate) type_cache: Option<&'a Vec<HashSet<TypeId>>>,
}

impl Copy for CompositionContext<'_> {}

impl Clone for CompositionContext<'_> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> CompositionContext<'a> {
    pub(crate) fn new(
        options: &'a CompositionOptions,
        tree: &'a Tree<RenderSegment>,
        start: &'a Node<RenderSegment>,
        type_cache: Option<&'a Vec<HashSet<TypeId>>>,
    ) -> CompositionContext<'a> {
        CompositionContext {
            options,
            tree,
            start,
            type_cache,
        }
    }

    /// Search the in-progress composition tree for nodes of type `Element`.
    /// Returns a [`CtxQuery`], allowing further specifications before running the search.
    pub fn find<Element: crate::Element>(&self) -> CtxQuery<Element, impl Fn(&Element) -> bool> {
        CtxQuery {
            ctx: self,
            timing: None,
            scope: None,
            where_fn: |_| true,
            __: PhantomData,
        }
    }

    /// Returns the composition beat length. A composition's tempo (BPM) is relative to this value.
    pub fn beat_length(&self) -> i32 {
        self.options.ticks_per_beat
    }

    /// Creates an [`Rng`] seeded from the currently rendering segment's seed.
    pub fn rng(&self) -> impl Rng {
        ChaCha12Rng::seed_from_u64(self.start.value.seed)
    }

    /// Creates an [`Rng`] seeded from a combination of the currently rendering segment's seed
    /// as well as the provided seed.
    pub fn rng_with_seed(&self, seed: impl Hash) -> impl Rng {
        let mut hasher = XxHash64::default();
        self.start.value.seed.hash(&mut hasher);
        seed.hash(&mut hasher);

        ChaCha12Rng::seed_from_u64(hasher.finish())
    }

    /// Search the in-progress composition tree for all [`Element`]s within the given
    /// [`TimingConstraint`] and [`SearchScope`] criteria that match the provided closure. Returns
    /// a vector of [`SegmentRef`]s referencing the matching [`Element`]s if any were found,
    /// or else [`None`]. This is useful if the timing data is required.
    fn get_all_segments_where<F: Element>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimingConstraint,
        scope: SearchScope,
    ) -> Option<Vec<SegmentRef<F>>> {
        let mut matching_segments: Vec<SegmentRef<F>> = vec![];

        let search_start = (match scope {
            SearchScope::WithinAncestor(t) => successors(Some(self.start), |node| {
                node.parent.map(|idx| &self.tree[idx])
            })
            .filter(|node| {
                successors(Some(&*node.value.segment.element), |&s| s.wrapped_element())
                    .any(|target| target.as_any().type_id() == t)
            })
            .last(),
            _ => None,
        })
        .unwrap_or(&self.tree[0]);

        for node in CtxIter::new::<F>(search_start, self.tree, self.type_cache, relation) {
            if self.is_in_scope(&scope, node)
                && node
                    .value
                    .segment
                    .element_as::<F>()
                    .is_some_and(&where_clause)
            {
                if let Ok(segment) = (&node.value.segment).try_into() {
                    matching_segments.insert(matching_segments.len(), segment);
                }
            }
        }

        if matching_segments.is_empty() {
            None
        } else {
            Some(matching_segments)
        }
    }

    fn is_in_scope(&self, scope: &SearchScope, node: &Node<RenderSegment>) -> bool {
        match scope {
            SearchScope::WithinAncestor(search_type) => {
                let mut cursor = self.start.parent;
                let mut opt_ancestor = None;

                while let Some(cursor_node) = cursor.and_then(|p_idx| self.tree.get(p_idx)) {
                    if successors(Some(&*cursor_node.value.segment.element), |&s| {
                        s.wrapped_element()
                    })
                    .any(|s| s.as_any().type_id() == *search_type)
                    {
                        opt_ancestor = Some(cursor_node);
                    }

                    cursor = cursor_node.parent;
                }

                if let Some(ancestor) = opt_ancestor {
                    cursor = Some(node.idx);
                    while let Some(cursor_node) = cursor.and_then(|idx| self.tree.get(idx)) {
                        if cursor_node.idx == ancestor.idx {
                            return true;
                        }
                        cursor = cursor_node.parent;
                    }
                }

                false
            }
            SearchScope::Within(search_type) => {
                let mut cursor = Some(node.idx);

                while let Some(ancestor) = cursor.and_then(|p_idx| self.tree.get(p_idx)) {
                    if successors(Some(&*ancestor.value.segment.element), |&s| {
                        s.wrapped_element()
                    })
                    .any(|s| s.as_any().type_id() == *search_type)
                    {
                        return true;
                    }

                    cursor = ancestor.parent;
                }

                false
            }
            SearchScope::Anywhere => true,
        }
    }
}

/// A context query builder. Initiate a query via [`CompositionContext::find`].
#[derive(Debug)]
pub struct CtxQuery<'a, S: Element, F: Fn(&S) -> bool> {
    ctx: &'a CompositionContext<'a>,
    timing: Option<TimingConstraint>,
    scope: Option<SearchScope>,
    where_fn: F,
    __: PhantomData<S>,
}

impl<'a, S: Element, F: Fn(&S) -> bool> CtxQuery<'a, S, F> {
    /// Restrict the search to segments matching a given [`TimingRelation`].
    pub fn with_timing<R: RangeBounds<i32>>(mut self, relation: TimingRelation, timing: R) -> Self {
        self.timing = Some(TimingConstraint::from((relation, timing)));

        self
    }

    /// Restrict the search to descendent segments a given [`Element`] type. This does
    /// not in itself impose any timing constraints for the search -- for that, use
    /// [`with_timing`](Self::with_timing).
    pub fn within<S2: Element>(mut self) -> Self {
        self.scope = Some(SearchScope::Within(TypeId::of::<S2>()));

        self
    }

    /// Restrict the search to segments generated within the initiator's ancestor of the
    /// given [`Element`]. This does not in itself impose any timing constraints for the
    /// search -- for that, use [`with_timing`](Self::with_timing).
    pub fn within_ancestor<S2: Element>(mut self) -> Self {
        self.scope = Some(SearchScope::WithinAncestor(TypeId::of::<S2>()));

        self
    }

    /// Restrict the search to segments matching the supplied closure.
    pub fn matching(self, where_fn: impl Fn(&S) -> bool) -> CtxQuery<'a, S, impl Fn(&S) -> bool> {
        CtxQuery {
            ctx: self.ctx,
            timing: self.timing,
            scope: self.scope,
            where_fn,
            __: self.__,
        }
    }

    /// Runs the context query, and returns a single optional result, or [`None`] if none are found.
    pub fn get(self) -> Option<SegmentRef<'a, S>> {
        self.ctx
            .get_all_segments_where::<S>(
                self.where_fn,
                self.timing.unwrap_or(TimingConstraint::from((
                    During,
                    self.ctx.start.value.segment.timing,
                ))),
                self.scope.unwrap_or(SearchScope::Anywhere),
            )
            .and_then(|mut v| {
                if v.first().is_none() {
                    None
                } else {
                    Some(v.swap_remove(0))
                }
            })
    }

    /// Runs the context query, and returns all results, or [`None`] if none are found.
    pub fn get_all(self) -> Option<Vec<SegmentRef<'a, S>>> {
        self.get_at_least(1)
    }

    /// Runs the context query. Returns all results if at least `min_requested` results are found,
    /// otherwise [`None`] is returned.
    pub fn get_at_least(self, min_requested: usize) -> Option<Vec<SegmentRef<'a, S>>> {
        if let Some(results) = self.ctx.get_all_segments_where::<S>(
            self.where_fn,
            self.timing.unwrap_or(TimingConstraint::from((
                Overlapping,
                self.ctx.start.value.segment.timing,
            ))),
            self.scope.unwrap_or(SearchScope::Anywhere),
        ) {
            if results.len() >= min_requested {
                return Some(results);
            }
        }

        None
    }

    /// Runs the context query, and returns a single result, or [`MissingContext`] error if none are found.
    pub fn require(self) -> Result<SegmentRef<'a, S>> {
        self.get()
            .ok_or(MissingContext(type_name::<S>().to_string()))
    }

    /// Runs the context query, and returns all results, or [`MissingContext`] error if none are found.
    pub fn require_all(self) -> Result<Vec<SegmentRef<'a, S>>> {
        self.require_at_least(1)
    }

    /// Runs the context query. If at least `min_requested` results are found they are returned,
    /// otherwise a [`MissingContext`] error is returned.
    pub fn require_at_least(self, min_requested: usize) -> Result<Vec<SegmentRef<'a, S>>> {
        self.get_at_least(min_requested)
            .ok_or(MissingContext(type_name::<S>().to_string()))
    }
}

/// Describes a timing relationship to reference time range.
#[derive(Debug)]
pub enum TimingRelation {
    /// Describes a relationship for a target whose time range fully includes the reference time range.
    During,
    /// Describes a relationship for a target whose time range shares any part of the reference time range.
    Overlapping,
    /// Describes a relationship for a target whose time range is fully enclosed within the reference time range.
    Within,
    /// Describes a relationship for a target whose time range begins within the reference time range.
    BeginningWithin,
    /// Describes a relationship for a target whose time range ends within the reference time range.
    EndingWithin,
    /// Describes a relationship for a target whose time range ends before/at the reference time range begin.
    Before,
    /// Describes a relationship for a target whose time range starts after/at the reference time range end.
    After,
}

/// Used to describe which portions of a composition tree to search during a context lookup.
#[derive(Debug)]
enum SearchScope {
    /// Describes the relationship for a target that is a descendent of a particular ancestor of the reference node type.
    WithinAncestor(TypeId),
    /// Describes the relationship for a target that is a descendent of a particular reference node type.
    Within(TypeId),
    /// Describes a scope that has no restrictions.
    Anywhere,
}

/// Describes a relationship between a target and reference time range.
#[derive(Debug)]
struct TimingConstraint {
    pub relation: TimingRelation,
    pub ref_range: (Bound<i32>, Bound<i32>),
}

impl<R: RangeBounds<i32>> From<(TimingRelation, R)> for TimingConstraint {
    fn from(value: (TimingRelation, R)) -> Self {
        TimingConstraint {
            relation: value.0,
            ref_range: (value.1.start_bound().cloned(), value.1.end_bound().cloned()),
        }
    }
}

impl TimingConstraint {
    // Determines if a target time range matches this relationship.
    fn matches<T: RangeBounds<i32>>(&self, target_range: &T) -> bool {
        match self.relation {
            During => target_range.contains_range(&self.ref_range),
            Overlapping => target_range.intersects(&self.ref_range),
            Within => target_range.is_contained_by(&self.ref_range),
            BeginningWithin => target_range.begins_within(&self.ref_range),
            EndingWithin => target_range.ends_within(&self.ref_range),
            Before => target_range.is_before(&self.ref_range),
            After => target_range.is_after(&self.ref_range),
        }
    }

    // Determines if a target time range could contain a matche for this relationship.
    fn could_match_within<T: RangeBounds<i32>>(&self, target_range: &T) -> bool {
        match self.relation {
            During | Overlapping => self.matches(target_range),
            Within | BeginningWithin | EndingWithin => self.ref_range.intersects(target_range),
            Before => match self.ref_range.start_bound() {
                Included(v) => target_range.intersects(&(Unbounded, Excluded(v))),
                Excluded(v) => target_range.intersects(&(Unbounded, Included(v))),
                Unbounded => false,
            },
            After => match self.ref_range.end_bound() {
                Included(v) => target_range.intersects(&(Excluded(v), Unbounded)),
                Excluded(v) => target_range.intersects(&(Included(v), Unbounded)),
                Unbounded => false,
            },
        }
    }
}

struct CtxIter<'a> {
    tree: &'a Tree<RenderSegment>,
    type_cache: Option<&'a Vec<HashSet<TypeId>>>,
    idx: usize,
    curr_nodes: Vec<&'a Node<RenderSegment>>,
    next_nodes: Vec<&'a Node<RenderSegment>>,
    time_relation: TimingConstraint,
    search_type: TypeId,
}

impl<'a> Iterator for CtxIter<'a> {
    type Item = &'a Node<RenderSegment>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.curr_nodes.get(self.idx) {
            if self
                .type_cache
                .map_or(true, |cache| cache[node.idx].contains(&self.search_type))
            {
                let mut child_nodes: Vec<&Node<RenderSegment>> = node
                    .children
                    .iter()
                    .map(|child_idx| &self.tree[*child_idx])
                    .filter(|n| n.value.rendered && self.might_have_items(n))
                    .collect();

                self.next_nodes.append(&mut child_nodes);
            }
            self.idx += 1;

            if self.time_relation.matches(&node.value.segment) {
                Some(node)
            } else {
                self.next()
            }
        } else if self.next_nodes.is_empty() {
            None
        } else {
            self.curr_nodes = vec![];
            self.curr_nodes.append(&mut self.next_nodes);
            self.idx = 0;

            self.next()
        }
    }
}

impl<'a> CtxIter<'a> {
    fn new<S: Element>(
        node: &'a Node<RenderSegment>,
        tree: &'a Tree<RenderSegment>,
        type_cache: Option<&'a Vec<HashSet<TypeId>>>,
        relation: TimingConstraint,
    ) -> CtxIter<'a> {
        CtxIter {
            tree,
            type_cache,
            idx: 0,
            curr_nodes: vec![node],
            next_nodes: vec![],
            time_relation: relation,
            search_type: TypeId::of::<S>(),
        }
    }

    fn might_have_items(&self, node: &Node<RenderSegment>) -> bool {
        self.time_relation.could_match_within(&node.value.segment)
    }
}
