use crate::derive::Element;
use std::collections::Bound;
use std::collections::Bound::{Excluded, Included, Unbounded};
use std::ops::{Range, RangeBounds};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The default beat length which divides evenly for many common factors.
pub const STANDARD_BEAT_LENGTH: i32 = 480;
/// Higher precision beat length if greater divisibility is required.
pub const HIGH_PRECISION_BEAT_LENGTH: i32 = 960;

/// Types implementing [`Element`].
pub mod elements {
    pub use super::Tempo;
}

/// The speed of a (or part of a) composition in beats per minute.
#[derive(Element, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tempo {
    pub(super) bpm: u32,
}

impl Tempo {
    /// Creates a [`Tempo`] from beats per measure.
    pub fn from_bpm(bpm: u32) -> Tempo {
        Tempo { bpm }
    }

    /// Returns this tempo with units of microseconds per beat.
    pub fn microseconds_per_beat(&self) -> u32 {
        60_000_000 / self.bpm
    }

    /// Returns this tempo with units of beats per minute.
    pub fn bpm(&self) -> u32 {
        self.bpm
    }
}

/// A start-inclusive, end-exclusive [`i32`] range (like [`Range<i32>`]) that is copyable,
/// and implements several utility methods.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Timing {
    /// The inclusive start of this interval.
    pub start: i32,
    /// The exclusive end of this interval.
    pub end: i32,
}

impl RangeBounds<i32> for Timing {
    fn start_bound(&self) -> Bound<&i32> {
        Bound::Included(&self.start)
    }

    fn end_bound(&self) -> Bound<&i32> {
        Bound::Excluded(&self.end)
    }
}

impl From<&Timing> for Timing {
    fn from(value: &Timing) -> Self {
        *value
    }
}

impl From<Range<i32>> for Timing {
    fn from(value: Range<i32>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<&Range<i32>> for Timing {
    fn from(value: &Range<i32>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl From<Timing> for Range<i32> {
    fn from(value: Timing) -> Self {
        value.start..value.end
    }
}

impl From<&Timing> for Range<i32> {
    fn from(value: &Timing) -> Self {
        value.start..value.end
    }
}

impl Timing {
    /// Returns the length of this timing (`self.end` - `self.start`).
    pub fn len(&self) -> i32 {
        self.end - self.start
    }
    /// Splits this timing into sequentual pieces of a given `size`.
    /// Returns the resulting `Vec`.
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// let split = Timing::from(0..10).divide_into(5);
    /// assert_eq!(split[0], Timing::from(0..5));
    /// assert_eq!(split[1], Timing::from(5..10));
    /// ```
    #[inline]
    pub fn divide_into(&self, size: i32) -> Vec<Timing> {
        <Range<i32>>::from(self)
            .step_by(size as usize)
            .scan((0, self.start), |s, _| {
                s.0 = s.1;
                s.1 = s.0 + size;

                Some(s.0..s.1)
            })
            .map(Timing::from)
            .collect::<Vec<_>>()
    }

    /// Returns a new [`Timing`] with start/end shifted by the given `amount`.
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..10).shifted_by(10), Timing::from(10..20))
    /// ```
    #[inline]
    pub fn shifted_by(&self, amount: i32) -> Timing {
        Self {
            start: self.start + amount,
            end: self.end + amount,
        }
    }

    /// Returns a new [`Timing`] with the start shifted by the given `amount`.
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..10).start_shifted_by(5), Timing::from(5..10))
    /// ```
    #[inline]
    pub fn start_shifted_by(&self, amount: i32) -> Timing {
        Self {
            start: self.start + amount,
            end: self.end,
        }
    }

    /// Returns a new [`Timing`] with the end shifted by the given `amount`.
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..10).end_shifted_by(-5), Timing::from(0..5))
    /// ```
    #[inline]
    pub fn end_shifted_by(&self, amount: i32) -> Timing {
        Self {
            start: self.start,
            end: self.end + amount,
        }
    }

    // Passthrough impls for RangeBounds<i32> so the trait doesn't need to be `use`ed.
    /// Checks if a particular `&i32` is contained in this [`Timing`].
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(1..10).contains(&1), true);
    /// assert_eq!(Timing::from(1..10).contains(&10), false);
    /// ```
    pub fn contains(&self, item: &i32) -> bool {
        <Self as RangeBounds<i32>>::contains(self, item)
    }

    // Passthrough impls for RangeChecks so the trait doesn't need to be `use`ed.
    /// Checks if this [`Timing`] is empty (including negative).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(1..1).is_empty(), true);
    /// assert_eq!(Timing::from(1..0).is_empty(), true);
    /// assert_eq!(Timing::from(1..2).is_empty(), false);
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        RangeOps::is_empty(self)
    }

    /// Checks if this [`Timing`] is before some other [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..1).is_before(&Timing::from(1..2)), true);
    /// assert_eq!(Timing::from(1..2).is_before(&Timing::from(0..1)), false);
    /// ```
    #[inline]
    pub fn is_before(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::is_before(self, other)
    }

    /// Checks if this [`Timing`] is after some other [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..1).is_after(&Timing::from(1..2)), false);
    /// assert_eq!(Timing::from(1..2).is_after(&Timing::from(0..1)), true);
    /// ```
    #[inline]
    pub fn is_after(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::is_after(self, other)
    }

    /// Checks if this [`Timing`] does not overlap with another [`Timing`] (or other
    /// [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..2).is_disjoint_from(&Timing::from(2..3)), true);
    /// assert_eq!(Timing::from(0..2).is_disjoint_from(&Timing::from(1..3)), false);
    /// ```
    #[inline]
    pub fn is_disjoint_from(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::is_disjoint_from(self, other)
    }

    /// Checks if this [`Timing`] overlaps with another [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..2).intersects(&Timing::from(1..3)), true);
    /// assert_eq!(Timing::from(0..2).intersects(&Timing::from(2..3)), false);
    /// ```
    #[inline]
    pub fn intersects(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::intersects(self, other)
    }

    /// Checks if this [`Timing`] contains another [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..2).contains_range(&Timing::from(0..1)), true);
    /// assert_eq!(Timing::from(0..2).contains_range(&Timing::from(1..3)), false);
    /// ```
    #[inline]
    pub fn contains_range(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::contains_range(self, other)
    }

    /// Checks if this [`Timing`] is contained by another [`Timing`] (or other
    /// [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..2).is_contained_by(&Timing::from(0..3)), true);
    /// assert_eq!(Timing::from(0..2).is_contained_by(&Timing::from(0..1)), false);
    /// ```
    #[inline]
    pub fn is_contained_by(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::is_contained_by(self, other)
    }

    /// Checks if this [`Timing`] begins within another [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..1).begins_within(&Timing::from(0..2)), true);
    /// assert_eq!(Timing::from(0..1).begins_within(&Timing::from(1..2)), false);
    /// ```
    #[inline]
    pub fn begins_within(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::begins_within(self, other)
    }

    /// Checks if this [`Timing`] ends within another [`Timing`] (or other [`RangeBounds<i32>`]).
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// assert_eq!(Timing::from(0..2).ends_within(&Timing::from(0..2)), true);
    /// assert_eq!(Timing::from(0..2).ends_within(&Timing::from(0..1)), false);
    /// ```
    #[inline]
    pub fn ends_within(&self, other: &impl RangeBounds<i32>) -> bool {
        RangeOps::ends_within(self, other)
    }
}

/// Convenient interval comparisons.
pub trait RangeOps<T>: RangeBounds<T> {
    /// Checks if an interval is empty.
    fn is_empty(&self) -> bool;
    /// Checks if an interval has no overlap with another.
    fn is_disjoint_from(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval has some overlap with another.
    fn intersects(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval ends before the start of another.
    fn is_before(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval starts after the end of another.
    fn is_after(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval starts within another.
    fn begins_within(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval ends within another.
    fn ends_within(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval contains another.
    fn contains_range(&self, other: &impl RangeBounds<T>) -> bool;
    /// Checks if this interval is contained by another.
    fn is_contained_by(&self, other: &impl RangeBounds<T>) -> bool;
}

impl<T, R> RangeOps<T> for R
where
    T: PartialOrd,
    R: RangeBounds<T>,
{
    #[inline]
    fn is_empty(&self) -> bool {
        match (self.start_bound(), self.end_bound()) {
            (Included(s), Excluded(e))
            | (Excluded(s), Included(e))
            | (Excluded(s), Excluded(e)) => e <= s,
            (Included(s), Included(e)) => e < s,
            (Included(_), Unbounded)
            | (Excluded(_), Unbounded)
            | (Unbounded, Included(_))
            | (Unbounded, Excluded(_))
            | (Unbounded, Unbounded) => false,
        }
    }

    #[inline]
    fn is_before(&self, other: &impl RangeBounds<T>) -> bool {
        <(Bound<&T>, Bound<&T>) as RangeOps<T>>::is_empty(&(other.start_bound(), self.end_bound()))
    }

    #[inline]
    fn is_after(&self, other: &impl RangeBounds<T>) -> bool {
        <(Bound<&T>, Bound<&T>) as RangeOps<T>>::is_empty(&(self.start_bound(), other.end_bound()))
    }

    #[inline]
    fn is_disjoint_from(&self, other: &impl RangeBounds<T>) -> bool {
        self.is_before(other) || self.is_after(other)
    }

    #[inline]
    fn intersects(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_disjoint_from(other)
    }

    #[inline]
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

    #[inline]
    fn is_contained_by(&self, other: &impl RangeBounds<T>) -> bool {
        other.contains_range(self)
    }

    #[inline]
    fn begins_within(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_after(other) && other.contains_range(&(self.start_bound(), other.end_bound()))
    }

    #[inline]
    fn ends_within(&self, other: &impl RangeBounds<T>) -> bool {
        !self.is_before(other) && other.contains_range(&(other.start_bound(), self.end_bound()))
    }
}

/// Convenience methods for `[Vec<Timing>]`.
pub trait TimingSequenceUtil {
    /// Joins the sequence of `Timing`s, merging overlapping/continuous regions.
    fn join(&self) -> Vec<Timing>;
}

impl TimingSequenceUtil for Vec<Timing> {
    /// Joins a sequence of [`Timing`]s. Overlapping or continuously sequential
    /// (i.e. end == start) are merged and the resulting sequence is returned.
    /// ```
    /// # use redact_composer_core::timing::Timing;
    /// # use redact_composer_core::timing::TimingSequenceUtil;
    /// assert_eq!(
    ///     vec![Timing::from(0..4), Timing::from(2..5), Timing::from(6..10)].join(),
    ///     vec![Timing::from(0..5), Timing::from(6..10)]
    /// );
    /// ```
    fn join(&self) -> Vec<Timing> {
        if let Some(first) = self.first().copied() {
            let mut joined = vec![first];

            for next in (0..self.len()).skip(1) {
                let joined_len = joined.len();
                if joined[joined_len - 1].contains(&self[next].start)
                    || joined[joined_len - 1].end == self[next].start
                {
                    joined[joined_len - 1].end = self[next].end;
                } else {
                    joined.push(Timing::from(self[next].start..self[next].end));
                }
            }

            joined
        } else {
            vec![]
        }
    }
}
