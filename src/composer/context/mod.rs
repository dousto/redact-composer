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

pub enum TimeRelation {
    /// Describes a relationship for a target whose time range fully includes the reference time range.
    During((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range fully shares some or all of the reference time range.
    Overlapping((Bound<i32>, Bound<i32>)),
    /// Describes a relationship for a target whose time range is fully enclosed within the reference time range.
    Within((Bound<i32>, Bound<i32>)),
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

    /// Describes a relationship for a target whose time range fully shares some or all of the reference time range.
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

    fn convert_to_inclusive_bounds<T: RangeBounds<i32>>(bounds: T) -> (Bound<i32>, Bound<i32>) {
        (
            match bounds.start_bound() {
                Bound::Included(t) => Bound::Included(*t),
                Bound::Excluded(t) => Bound::Included(t + 1),
                Bound::Unbounded => Bound::Unbounded,
            },
            match bounds.end_bound() {
                Bound::Included(t) => Bound::Included(*t),
                Bound::Excluded(t) => Bound::Included(t - 1),
                Bound::Unbounded => Bound::Unbounded,
            },
        )
    }

    // Determines if a target time range matches this relationship.
    fn matches<T: RangeBounds<i32>>(&self, target_range: T) -> bool {
        let target_range = Self::convert_to_inclusive_bounds((
            target_range.start_bound(),
            target_range.end_bound(),
        ));

        match self {
            TimeRelation::During(ref_range) => {
                let ref_range = Self::convert_to_inclusive_bounds(*ref_range);

                // Emulate `ref_range.start >= target_range.start  && ref_range.end <= target_range.end` accounting for all range bound types (ex-/inclusive/unbounded)
                let start_satisfied = match ref_range.0 {
                    Bound::Included(rin) => {
                        let start_range = (Bound::Unbounded, Bound::Included(rin));
                        match target_range.0 {
                            Bound::Included(tin) => start_range.contains(&tin),
                            Bound::Unbounded => true,
                            _ => panic!("Unexpected range bound: {:?}", target_range.0),
                        }
                    }
                    Bound::Unbounded => {
                        if let Bound::Unbounded = target_range.0 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => panic!("Unexpected range bound: {:?}", ref_range.0),
                };

                let end_satisfied = match ref_range.1 {
                    Bound::Included(rin) => {
                        let end_range = (Bound::Included(rin), Bound::Unbounded);
                        match target_range.1 {
                            Bound::Included(tin) => end_range.contains(&tin),
                            Bound::Unbounded => true,
                            _ => panic!("Unexpected range bound: {:?}", target_range.1),
                        }
                    }
                    Bound::Unbounded => {
                        if let Bound::Unbounded = target_range.1 {
                            true
                        } else {
                            false
                        }
                    }
                    _ => panic!("Unexpected range bound: {:?}", ref_range.1),
                };

                return start_satisfied && end_satisfied;
            }
            TimeRelation::Overlapping(ref_range) => {
                let ref_range = Self::convert_to_inclusive_bounds(*ref_range);
                let start_satisfied = match target_range.1 {
                    Bound::Included(tin) => ref_range.contains(&tin),
                    Bound::Unbounded => false,
                    _ => panic!("Unexpected range bound: {:?}", target_range.1),
                };

                let end_satisfied = match target_range.0 {
                    Bound::Included(tin) => ref_range.contains(&tin),
                    Bound::Unbounded => false,
                    _ => panic!("Unexpected range bound: {:?}", target_range.0),
                };

                start_satisfied || end_satisfied || Self::during(ref_range).matches(target_range)
            }
            TimeRelation::Within(ref_range) => {
                // Inversion of TimeRelation::During
                return TimeRelation::During(target_range).matches(*ref_range);
            }
            TimeRelation::Before(ref_range) => {
                let ref_range = Self::convert_to_inclusive_bounds(*ref_range);
                match ref_range.0 {
                    Bound::Included(rin) => {
                        let before_range = (Bound::Unbounded, Bound::Excluded(rin));
                        match target_range.1 {
                            Bound::Included(tin) => before_range.contains(&tin),
                            Bound::Unbounded => false,
                            _ => panic!("Unexpected range bound: {:?}", target_range.1),
                        }
                    }
                    Bound::Unbounded => false,
                    _ => panic!("Unexpected range bound: {:?}", target_range.0),
                }
            }
            TimeRelation::After(ref_range) => {
                let ref_range = Self::convert_to_inclusive_bounds(*ref_range);
                match ref_range.1 {
                    Bound::Included(rin) => {
                        let after_range = (Bound::Excluded(rin), Bound::Unbounded);
                        match target_range.0 {
                            Bound::Included(tin) => after_range.contains(&tin),
                            Bound::Unbounded => false,
                            _ => panic!("Unexpected range bound: {:?}", target_range.0),
                        }
                    }
                    Bound::Unbounded => false,
                    _ => panic!("Unexpected range bound: {:?}", ref_range.1),
                }
            }
        }
    }
}

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
