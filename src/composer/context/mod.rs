use std::any::TypeId;
use std::iter::successors;
use std::marker::PhantomData;
use std::ops::Bound::{Excluded, Included, Unbounded};
use std::ops::{Bound, RangeBounds};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use super::TypedSegment;
use super::{
    render::{Node, Tree},
    RenderSegment, SegmentType,
};

use crate::composer::render;
use crate::error::RendererError::MissingContext;

#[cfg(test)]
mod test;

/// Describes a timing relationship to reference time range.
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

pub struct CtxQuery<'a, S: SegmentType, F: Fn(&S) -> bool> {
    ctx: &'a CompositionContext<'a>,
    timing: Option<TimeRelation>,
    scope: Option<SearchScope>,
    where_fn: F,
    __: PhantomData<S>,
}

impl<'a, S: SegmentType, F: Fn(&S) -> bool> CtxQuery<'a, S, F> {
    /// Restrict the search to segments matching a given time relationship.
    pub fn with_timing<R: RangeBounds<i32>>(
        mut self,
        relation: TimingRelation,
        time_range: &R,
    ) -> Self {
        let bounds = (
            time_range.start_bound().cloned(),
            time_range.end_bound().cloned(),
        );

        self.timing = Some(match relation {
            TimingRelation::During => TimeRelation::During(bounds),
            TimingRelation::Overlapping => TimeRelation::Overlapping(bounds),
            TimingRelation::Within => TimeRelation::Within(bounds),
            TimingRelation::BeginningWithin => TimeRelation::BeginningWithin(bounds),
            TimingRelation::EndingWithin => TimeRelation::EndingWithin(bounds),
            TimingRelation::Before => TimeRelation::Before(bounds),
            TimingRelation::After => TimeRelation::After(bounds),
        });

        self
    }

    /// Restrict the search to segments generated within a given [`SegmentType`].
    pub fn within<S2: SegmentType>(mut self) -> Self {
        self.scope = Some(SearchScope::Within(TypeId::of::<S2>()));

        self
    }

    /// Restrict the search to segments generated within the initiator's ancestor of the
    /// given [`SegmentType`].
    pub fn within_ancestor<S2: SegmentType>(mut self) -> Self {
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
    pub fn get(self) -> Option<TypedSegment<'a, S>> {
        self.ctx
            .get_all_segments_where::<S>(
                self.where_fn,
                self.timing.unwrap_or(TimeRelation::during(
                    &self.ctx.start.value.segment.time_range,
                )),
                self.scope.unwrap_or(SearchScope::Anywhere),
            )
            .and_then(|mut v| {
                if v.get(0).is_none() {
                    None
                } else {
                    Some(v.swap_remove(0))
                }
            })
    }

    /// Runs the context query, and returns all results, or [`None`] if none are found.
    pub fn get_all(self) -> Option<Vec<TypedSegment<'a, S>>> {
        self.get_at_least(1)
    }

    /// Runs the context query. Returns all results if at least `min_requested` results are found,
    /// otherwise [`None`] is returned.
    pub fn get_at_least(self, min_requested: usize) -> Option<Vec<TypedSegment<'a, S>>> {
        if let Some(results) = self.ctx.get_all_segments_where::<S>(
            self.where_fn,
            self.timing.unwrap_or(TimeRelation::overlapping(
                &self.ctx.start.value.segment.time_range,
            )),
            self.scope.unwrap_or(SearchScope::Anywhere),
        ) {
            if results.len() >= min_requested {
                return Some(results);
            }
        }

        None
    }

    /// Runs the context query, and returns a single result, or [`MissingContext`] error if none are found.
    pub fn require(self) -> render::Result<TypedSegment<'a, S>> {
        self.get().ok_or(MissingContext)
    }

    /// Runs the context query, and returns all results, or [`MissingContext`] error if none are found.
    pub fn require_all(self) -> render::Result<Vec<TypedSegment<'a, S>>> {
        self.require_at_least(1)
    }

    /// Runs the context query. If at least `min_requested` results are found they are returned,
    /// otherwise a [`MissingContext`] error is returned.
    pub fn require_at_least(
        self,
        min_requested: usize,
    ) -> render::Result<Vec<TypedSegment<'a, S>>> {
        self.get_at_least(min_requested).ok_or(MissingContext)
    }
}

/// Type used during the render of abstract CompositionSegments which allows lookup
/// of data from other composition tree nodes.
pub struct CompositionContext<'a> {
    pub tree: &'a Tree<RenderSegment>,
    pub start: &'a Node<RenderSegment>,
}

impl<'a> CompositionContext<'a> {
    pub fn new(
        tree: &'a Tree<RenderSegment>,
        start: &'a Node<RenderSegment>,
    ) -> CompositionContext<'a> {
        CompositionContext { tree, start }
    }

    /// Provides a reproducible source of randomness while rendering [`SegmentType`]s. This function
    /// creates and returns an [`Rng`] (currently implemented with [`ChaCha12Rng`]) seeded from a
    /// parent Rng of the [`super::Composer`].
    ///
    /// Note: Call this once and reuse the returned [`Rng`] if multiple random generations are
    /// required. Since this creates a new [`Rng`] every time it is called, calling it multiple
    /// times (from the same [`CompositionContext`]) will result in multiple [`Rng`]s seeded
    /// identically, producing the same random sequences.
    pub fn rng(&self) -> impl Rng {
        ChaCha12Rng::seed_from_u64(self.start.value.seed)
    }

    pub fn rng_of<T: SegmentType>(&self) -> impl Rng {
        let mut matching_node = self.start;

        while let None = matching_node.value.segment.segment_type_as::<T>() {
            if let Some(parent_id) = matching_node.parent {
                matching_node = &self.tree[parent_id];
            }
        }

        ChaCha12Rng::seed_from_u64(matching_node.value.seed)
    }

    /// Search the in-progress composition tree for nodes of type [`SegmentType`].
    /// Returns a [`CtxQuery`], allowing further specifications before running the search.
    pub fn find<S: SegmentType>(&self) -> CtxQuery<S, impl Fn(&S) -> bool> {
        CtxQuery {
            ctx: &self,
            timing: None,
            scope: None,
            where_fn: |_| true,
            __: Default::default(),
        }
    }

    /// Search the in-progress composition tree for all [`SegmentType`]s within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria that match the provided closure. Returns
    /// a vector of [`TypedSegment`]s referencing the matching [`SegmentType`]s if any were found,
    /// or else [`None`]. This is useful if the timing data is required.
    ///
    /// # Example
    /// ```no_run
    /// # use std::ops::Range;
    /// # use redact_composer::{
    /// #     composer::{
    /// #         TypedSegment,
    /// #         PlayNote,
    /// #         context::CompositionContext
    /// #     },
    /// #     musical::rhythm::STANDARD_BEAT_LENGTH
    /// # };
    /// use redact_composer::composer::context::TimingRelation::Within;
    /// # let context: CompositionContext = unimplemented!();
    /// # let time_range: Range<i32> = unimplemented!();
    ///
    /// // Get all middle C notes within time_range
    /// let middle_c = 60;
    /// let single_beat_notes: Option<Vec<TypedSegment<PlayNote>>> = context.find::<PlayNote>()
    /// .matching(|&note| note.note == middle_c)
    /// .with_timing(Within, &time_range)
    /// .get_all();
    /// ```
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    fn get_all_segments_where<F: SegmentType>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<Vec<TypedSegment<F>>> {
        let mut matching_segments: Vec<TypedSegment<F>> = vec![];

        for node in CtxIter::new(&self.tree[0], self.tree, relation) {
            if self.is_in_scope(&scope, node)
                && node
                    .value
                    .segment
                    .segment_type_as::<F>()
                    .map(&where_clause)
                    .unwrap_or(false)
            {
                if let Some(segment) = (&node.value.segment).try_into().ok() {
                    matching_segments.insert(matching_segments.len(), segment)
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
                    if successors(Some(&*cursor_node.value.segment.segment_type), |&s| {
                        s.wrapped_type()
                    })
                    .any(|s| s.as_any().type_id() == *search_type)
                    {
                        opt_ancestor = Some(cursor_node)
                    }

                    cursor = cursor_node.parent
                }

                if let Some(ancestor) = opt_ancestor {
                    cursor = Some(node.idx);
                    while let Some(cursor_node) = cursor.and_then(|idx| self.tree.get(idx)) {
                        if cursor_node.idx == ancestor.idx {
                            return true;
                        }
                        cursor = cursor_node.parent
                    }
                }

                false
            }
            SearchScope::Within(search_type) => {
                let mut cursor = Some(node.idx);

                while let Some(ancestor) = cursor.and_then(|p_idx| self.tree.get(p_idx)) {
                    if successors(Some(&*ancestor.value.segment.segment_type), |&s| {
                        s.wrapped_type()
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

trait RangeHelper<T>: RangeBounds<T> {
    fn is_empty(&self) -> bool;
    fn is_disjoint_from(&self, other: &impl RangeBounds<T>) -> bool;
    fn intersects(&self, other: &impl RangeBounds<T>) -> bool;
    fn is_before(&self, other: &impl RangeBounds<T>) -> bool;
    fn is_after(&self, other: &impl RangeBounds<T>) -> bool;
    fn begins_within(&self, other: &impl RangeBounds<T>) -> bool;
    fn ends_within(&self, other: &impl RangeBounds<T>) -> bool;
    fn contains_range(&self, other: &impl RangeBounds<T>) -> bool;
    fn is_contained_by(&self, other: &impl RangeBounds<T>) -> bool;
}

impl<T, R> RangeHelper<T> for R
where
    T: PartialOrd,
    R: RangeBounds<T>,
{
    fn is_empty(&self) -> bool {
        match (self.start_bound(), self.end_bound()) {
            (Bound::Included(s), Bound::Excluded(e)) => e <= s,
            (Bound::Excluded(s), Bound::Included(e)) => e <= s,
            (Bound::Excluded(s), Bound::Excluded(e)) => e <= s,
            (Bound::Included(s), Bound::Included(e)) => e < s,
            (Bound::Included(_), Bound::Unbounded) => false,
            (Bound::Excluded(_), Bound::Unbounded) => false,
            (Bound::Unbounded, Bound::Included(_)) => false,
            (Bound::Unbounded, Bound::Excluded(_)) => false,
            (Bound::Unbounded, Bound::Unbounded) => false,
        }
    }

    fn is_before(&self, other: &impl RangeBounds<T>) -> bool {
        <(Bound<&T>, Bound<&T>) as RangeHelper<T>>::is_empty(&(
            other.start_bound(),
            self.end_bound(),
        ))
    }

    fn is_after(&self, other: &impl RangeBounds<T>) -> bool {
        <(Bound<&T>, Bound<&T>) as RangeHelper<T>>::is_empty(&(
            self.start_bound(),
            other.end_bound(),
        ))
    }

    fn is_disjoint_from(&self, other: &impl RangeBounds<T>) -> bool {
        self.is_before(other) || self.is_after(other)
    }

    fn intersects(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_disjoint_from(other)
    }

    fn contains_range(&self, other: &impl RangeBounds<T>) -> bool {
        (match self.end_bound() {
            Included(b) => other.is_before(&(Excluded(b), Unbounded)),
            Excluded(b) => other.is_before(&(Included(b), Unbounded)),
            Unbounded => true,
        } && match self.start_bound() {
            Included(b) => other.is_after(&(Unbounded, Excluded(b))),
            Excluded(b) => other.is_after(&(Unbounded, Included(b))),
            Unbounded => true,
        })
    }

    fn is_contained_by(&self, other: &impl RangeBounds<T>) -> bool {
        other.contains_range(self)
    }

    fn begins_within(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_after(other) && other.contains_range(&(self.start_bound(), other.end_bound()))
    }

    fn ends_within(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_before(other) && other.contains_range(&(other.start_bound(), self.end_bound()))
    }
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
enum TimeRelation {
    /// Describes a relationship for a target whose time range fully includes the reference time range.
    During((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range shares any part of the reference time range.
    Overlapping((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range is fully enclosed within the reference time range.
    Within((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range begins within the reference time range.
    BeginningWithin((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range ends within the reference time range.
    EndingWithin((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range ends before/at the reference time range begin.
    Before((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range starts after/at the reference time range end.
    After((Bound<i32>, Bound<i32>)),
}

impl TimeRelation {
    /// Describes a relationship for a target whose time range fully includes the reference time range.
    pub fn during<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::During((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range shares any part of the reference time range.
    pub fn overlapping<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::Overlapping((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range is fully enclosed within the reference time range.
    pub fn within<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::Within((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends within the reference time range.
    pub fn beginning_within<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::BeginningWithin((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends within the reference time range.
    pub fn ending_within<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::EndingWithin((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends before/at the reference time range begin.
    pub fn before<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::Before((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range starts after/at the reference time range end.
    pub fn after<T: RangeBounds<i32>>(ref_range: &T) -> TimeRelation {
        Self::After((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    // Determines if a target time range matches this relationship.
    fn matches<T: RangeBounds<i32>>(&self, target_range: T) -> bool {
        match self {
            TimeRelation::During(ref_range) => target_range.contains_range(ref_range),
            TimeRelation::Overlapping(ref_range) => target_range.intersects(ref_range),
            TimeRelation::Within(ref_range) => target_range.is_contained_by(ref_range),
            TimeRelation::BeginningWithin(ref_range) => target_range.begins_within(ref_range),
            TimeRelation::EndingWithin(ref_range) => target_range.ends_within(ref_range),
            TimeRelation::Before(ref_range) => target_range.is_before(ref_range),
            TimeRelation::After(ref_range) => target_range.is_after(ref_range),
        }
    }
}

struct CtxIter<'a> {
    tree: &'a Tree<RenderSegment>,
    idx: usize,
    curr_nodes: Vec<&'a Node<RenderSegment>>,
    next_nodes: Vec<&'a Node<RenderSegment>>,
    time_relation: TimeRelation,
}

impl<'a> Iterator for CtxIter<'a> {
    type Item = &'a Node<RenderSegment>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.curr_nodes.get(self.idx) {
            let mut child_nodes: Vec<&Node<RenderSegment>> = node
                .children
                .iter()
                .map(|child_idx| &self.tree[*child_idx])
                .filter(|n| n.value.rendered && self.might_have_items(n))
                .collect();

            self.next_nodes.append(&mut child_nodes);
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
    fn new(
        node: &'a Node<RenderSegment>,
        tree: &'a Tree<RenderSegment>,
        relation: TimeRelation,
    ) -> CtxIter<'a> {
        CtxIter {
            tree,
            idx: 0,
            curr_nodes: vec![node],
            next_nodes: vec![],
            time_relation: relation,
        }
    }

    fn might_have_items(&self, node: &Node<RenderSegment>) -> bool {
        match self.time_relation {
            TimeRelation::During(_) => self.time_relation.matches(&node.value.segment),
            TimeRelation::Overlapping(_) => self.time_relation.matches(&node.value.segment),
            TimeRelation::Within((a, b)) => node.value.segment.intersects(&(a, b)),
            TimeRelation::BeginningWithin((a, b)) => node.value.segment.intersects(&(a, b)),
            TimeRelation::EndingWithin((a, b)) => node.value.segment.intersects(&(a, b)),
            TimeRelation::Before((a, _)) => match a {
                Included(v) => node.value.segment.intersects(&(Unbounded, Excluded(v))),
                Excluded(v) => node.value.segment.intersects(&(Unbounded, Included(v))),
                Unbounded => false,
            },
            TimeRelation::After((_, b)) => match b {
                Included(v) => node.value.segment.intersects(&(Excluded(v), Unbounded)),
                Excluded(v) => node.value.segment.intersects(&(Included(v), Unbounded)),
                Unbounded => false,
            },
        }
    }
}
