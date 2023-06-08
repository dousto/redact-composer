use std::any::TypeId;
use std::ops::{Bound, RangeBounds};

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;

use super::{
    render::{Node, Tree},
    RenderSegment, SegmentType,
};
use super::{CompositionSegment, Part};

#[cfg(test)]
mod test;

/// Type used during the render of abstract CompositionSegments which allows lookup
/// of data from other composition tree nodes.
///
/// ## Fields
/// * `tree: &Tree<RenderSegment>` A reference to the (in-progress) composition tree
/// * `start: &Node<RenderSegment>` The node being rendered. Lookups are relative to this node.
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

    /// Provides a reproducible source of randomness while rendering [`SegmentType`]s. This function creates and returns
    /// an [`Rng`] (currently implemented with [`ChaCha12Rng`]) seeded from a parent Rng of the [`super::Composer`].
    ///
    /// Note: Call this once and reuse the returned [`Rng`] if multiple random generations are required. Since this creates
    /// a new [`Rng`] every time it is called, calling it multiple times (from the same [`CompositionContext`]) will
    /// result in multiple [`Rng`]s seeded identically, resulting in the same random sequences.
    pub fn rng(&self) -> impl Rng {
        ChaCha12Rng::seed_from_u64(self.start.value.seed)
    }

    /// Get a single typed implementation of [`SegmentType`] from the render tree within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get<F: SegmentType>(&self, relation: TimeRelation, scope: SearchScope) -> Option<&F> {
        self.get_segment::<F>(relation, scope)
            .and_then(|s| s.segment_type_as::<F>())
    }

    /// Get a single typed implementation of [`SegmentType`] from the render tree within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria that matches the passed [`Fn(&SegmentType) -> bool`] closure.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_where<F: SegmentType>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<&F> {
        self.get_segment_where::<F>(where_clause, relation, scope)
            .and_then(|s| s.segment_type_as::<F>())
    }

    /// Get all typed implementations of [`SegmentType`] from the render tree within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_all<F: SegmentType>(
        &self,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<Vec<&F>> {
        self.get_all_segments::<F>(relation, scope)
            .and_then(|v| Some(v.iter().flat_map(|s| s.segment_type_as::<F>()).collect()))
    }

    /// Get all typed implementations of [`SegmentType`] from the render tree within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria that matches the passed [`Fn(&SegmentType) -> bool`] closure.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_all_where<F: SegmentType>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<Vec<&F>> {
        self.get_all_segments_where::<F>(where_clause, relation, scope)
            .and_then(|v| Some(v.iter().flat_map(|s| s.segment_type_as::<F>()).collect()))
    }

    /// Get the containing [CompositionSegment] from the render tree matching the given [SegmentType] where within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_segment<F: SegmentType>(
        &self,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<&CompositionSegment> {
        self.get_all_segments::<F>(relation, scope)
            .and_then(|v| v.first().map(|s| *s))
    }

    /// Get the containing [CompositionSegment] from the render tree matching the given [SegmentType] within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria that matches the passed [`Fn(&SegmentType) -> bool`] closure.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_segment_where<F: SegmentType>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<&CompositionSegment> {
        self.get_all_segments_where::<F>(where_clause, relation, scope)
            .and_then(|v| v.first().map(|s| *s))
    }

    /// Get the containing [CompositionSegment]s from the render tree matching the given [SegmentType] where within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_all_segments<F: SegmentType>(
        &self,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<Vec<&CompositionSegment>> {
        self.get_all_segments_where::<F>(|_| true, relation, scope)
    }

    /// Get the containing [CompositionSegment]s from the render tree matching the given [SegmentType] within the given
    /// [`TimeRelation`] and [`SearchScope`] criteria that matches the passed [`Fn(&SegmentType) -> bool`] closure.
    ///
    /// See [`TimeRelation`] and [`SearchScope`] for specifying the match criteria.
    pub fn get_all_segments_where<F: SegmentType>(
        &self,
        where_clause: impl Fn(&F) -> bool,
        relation: TimeRelation,
        scope: SearchScope,
    ) -> Option<Vec<&CompositionSegment>> {
        let mut matching_segments: Vec<&CompositionSegment> = vec![];
        let mut opt_ancestor = self.start.parent.and_then(|p_idx| self.tree.get(p_idx));
        let mut skip = vec![self.start.idx];

        while let Some(ancestor) = opt_ancestor {
            if !matching_segments.is_empty() {
                break;
            }

            let iter = self.tree.node_iter_with_skip(ancestor, skip).filter(|n| {
                n.value.rendered
                    && relation.matches(n.value.segment.begin..n.value.segment.end)
                    && self.is_in_scope(&scope, n)
            });

            for node in iter {
                if node
                    .value
                    .segment
                    .segment_type_as::<F>()
                    .and_then(|f| Some(where_clause(f)))
                    .unwrap_or(false)
                {
                    matching_segments.insert(matching_segments.len(), &node.value.segment)
                }
            }

            skip = vec![ancestor.idx];
            opt_ancestor = ancestor.parent.and_then(|p_idx| self.tree.get(p_idx));
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
                    if *search_type
                        == (&*cursor_node.value.segment.segment_type)
                            .as_any()
                            .type_id()
                        || cursor_node
                            .value
                            .segment
                            .segment_type_as::<Part>()
                            .map(|p| *search_type == (&*p.0).as_any().type_id())
                            .unwrap_or(false)
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
                    if (&*ancestor.value.segment.segment_type).as_any().type_id() == *search_type
                        || ancestor
                            .value
                            .segment
                            .segment_type_as::<Part>()
                            .map(|p| *search_type == (&*p.0).as_any().type_id())
                            .unwrap_or(false)
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

#[derive(Debug)]
pub enum SearchScope {
    /// Describes the relationship for a target that is a descendent of a particular ancestor of the reference node type.
    WithinAncestor(TypeId),
    /// Describes the relationship for a target that is a descendent of a particular reference node type.
    Within(TypeId),
    /// Describes a scope that has no restrictions.
    Anywhere,
}

impl SearchScope {
    /// Describes the relationship for a target that is a descendent of a particular reference node type.
    pub fn within_any<T: SegmentType>() -> SearchScope {
        SearchScope::Within(TypeId::of::<T>())
    }

    /// Describes the relationship for a target that is a descendent of a particular ancestor of the reference node type.
    pub fn within_ancestor<T: SegmentType>() -> SearchScope {
        SearchScope::WithinAncestor(TypeId::of::<T>())
    }

    /// Describes a scope that has no restrictions.
    pub fn anywhere() -> SearchScope {
        SearchScope::Anywhere
    }
}

pub enum TimeRelation {
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
    pub fn during<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::During((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range shares any part of the reference time range.
    pub fn overlapping<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::Overlapping((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range is fully enclosed within the reference time range.
    pub fn within<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::Within((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends within the reference time range.
    pub fn beginning_within<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::BeginningWithin((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends within the reference time range.
    pub fn ending_within<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::EndingWithin((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range ends before/at the reference time range begin.
    pub fn before<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::Before((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    /// Describes a relationship for a target whose time range starts after/at the reference time range end.
    pub fn after<T: RangeBounds<i32>>(ref_range: T) -> TimeRelation {
        Self::After((
            ref_range.start_bound().cloned(),
            ref_range.end_bound().cloned(),
        ))
    }

    // Determines if a target time range matches this relationship.
    fn matches<T: RangeBounds<i32>>(&self, target_range: T) -> bool {
        let (tar_begin, tar_end) = BoundType::bounds_from(&target_range);
        let (ref_begin, ref_end) = self.bounds();

        match self {
            TimeRelation::During(_) => {
                tar_begin.is_before(&ref_begin) && tar_end.is_after(&ref_end)
            }
            TimeRelation::Overlapping(_) => {
                !tar_end.is_before(&ref_begin) && !tar_begin.is_after(&ref_end)
            }
            TimeRelation::Within(_) => {
                ref_begin.is_before(&tar_begin) && ref_end.is_after(&tar_end)
            }
            TimeRelation::BeginningWithin(_) => {
                !tar_begin.is_after(&ref_end) && tar_begin.is_after(&ref_begin)
            }
            TimeRelation::EndingWithin(_) => {
                !tar_end.is_before(&ref_begin) && tar_end.is_before(&ref_end)
            }
            TimeRelation::Before(_) => tar_end.is_before(&ref_begin),
            TimeRelation::After(_) => tar_begin.is_after(&ref_end),
        }
    }

    fn bounds(&self) -> (BoundType, BoundType) {
        match self {
            TimeRelation::During(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::Overlapping(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::Within(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::BeginningWithin(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::EndingWithin(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::Before(bounds) => BoundType::bounds_from(bounds),
            TimeRelation::After(bounds) => BoundType::bounds_from(bounds),
        }
    }
}

#[derive(Debug)]
enum BoundType {
    Start(Bound<i32>),
    End(Bound<i32>),
}

impl BoundType {
    fn bounds_from(range: &impl RangeBounds<i32>) -> (BoundType, BoundType) {
        (
            BoundType::Start(range.start_bound().cloned()),
            BoundType::End(range.end_bound().cloned()),
        )
    }

    fn is_after(&self, other: &BoundType) -> bool {
        other.is_before(self)
    }

    // Here lies all the nastiness..
    fn is_before(&self, other: &BoundType) -> bool {
        match (self, other) {
            // Bounded cases
            (BoundType::Start(Bound::Excluded(start)), BoundType::Start(Bound::Included(end))) => {
                start <= &(end - 1)
            }
            (BoundType::Start(Bound::Excluded(start)), BoundType::End(Bound::Excluded(end))) => {
                start <= &(end - 1)
            }
            (BoundType::End(Bound::Included(start)), BoundType::Start(Bound::Included(end))) => {
                start <= &(end - 1)
            }
            (BoundType::End(Bound::Included(start)), BoundType::End(Bound::Excluded(end))) => {
                start <= &(end - 1)
            }
            (BoundType::Start(Bound::Included(start)), BoundType::Start(Bound::Included(end))) => {
                start <= end
            }
            (BoundType::Start(Bound::Included(start)), BoundType::End(Bound::Excluded(end))) => {
                start <= end
            }
            (BoundType::Start(Bound::Excluded(start)), BoundType::Start(Bound::Excluded(end))) => {
                start <= end
            }
            (BoundType::Start(Bound::Excluded(start)), BoundType::End(Bound::Included(end))) => {
                start <= end
            }
            (BoundType::End(Bound::Included(start)), BoundType::Start(Bound::Excluded(end))) => {
                start <= end
            }
            (BoundType::End(Bound::Included(start)), BoundType::End(Bound::Included(end))) => {
                start <= end
            }
            (BoundType::End(Bound::Excluded(start)), BoundType::Start(Bound::Included(end))) => {
                start <= end
            }
            (BoundType::End(Bound::Excluded(start)), BoundType::End(Bound::Excluded(end))) => {
                start <= end
            }
            (BoundType::Start(Bound::Included(start)), BoundType::Start(Bound::Excluded(end))) => {
                start <= &(end + 1)
            }
            (BoundType::Start(Bound::Included(start)), BoundType::End(Bound::Included(end))) => {
                start <= &(end + 1)
            }
            (BoundType::End(Bound::Excluded(start)), BoundType::Start(Bound::Excluded(end))) => {
                start <= &(end + 1)
            }
            (BoundType::End(Bound::Excluded(start)), BoundType::End(Bound::Included(end))) => {
                start <= &(end + 1)
            }

            // Unbounded cases
            (BoundType::Start(Bound::Included(_)), BoundType::Start(Bound::Unbounded)) => false,
            (BoundType::Start(Bound::Excluded(_)), BoundType::Start(Bound::Unbounded)) => false,
            (BoundType::End(Bound::Included(_)), BoundType::Start(Bound::Unbounded)) => false,
            (BoundType::End(Bound::Excluded(_)), BoundType::Start(Bound::Unbounded)) => false,
            (BoundType::End(Bound::Unbounded), BoundType::Start(Bound::Included(_))) => false,
            (BoundType::End(Bound::Unbounded), BoundType::Start(Bound::Excluded(_))) => false,
            (BoundType::End(Bound::Unbounded), BoundType::Start(Bound::Unbounded)) => false,
            (BoundType::End(Bound::Unbounded), BoundType::End(Bound::Included(_))) => false,
            (BoundType::End(Bound::Unbounded), BoundType::End(Bound::Excluded(_))) => false,
            (BoundType::Start(Bound::Included(_)), BoundType::End(Bound::Unbounded)) => true,
            (BoundType::Start(Bound::Excluded(_)), BoundType::End(Bound::Unbounded)) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::Start(Bound::Included(_))) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::Start(Bound::Excluded(_))) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::Start(Bound::Unbounded)) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::End(Bound::Included(_))) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::End(Bound::Excluded(_))) => true,
            (BoundType::Start(Bound::Unbounded), BoundType::End(Bound::Unbounded)) => true,
            (BoundType::End(Bound::Included(_)), BoundType::End(Bound::Unbounded)) => true,
            (BoundType::End(Bound::Excluded(_)), BoundType::End(Bound::Unbounded)) => true,
            (BoundType::End(Bound::Unbounded), BoundType::End(Bound::Unbounded)) => true,
        }
    }
}
