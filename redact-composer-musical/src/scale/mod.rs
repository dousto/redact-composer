use crate::{Interval, IntervalStepSequence};

mod mode;
pub use mode::*;

mod degree;
pub use degree::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Sequence of intervals spanning 12 semitones or one octave.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scale {
    /// ```
    /// # use redact_composer_musical::Scale;
    /// # use redact_composer_musical::{Interval, IntervalStepSequence};
    /// let (w, h) = (Interval(2), Interval(1)); // w = Whole-step, h = Half-step
    /// assert_eq!(Scale::Major.interval_steps(), vec![w, w, h, w, w, w, h]);
    /// assert_eq!(Scale::Major.interval_steps().into_iter().sum::<Interval>(), Interval::Octave);
    /// ```
    Major,
    /// ```
    /// # use redact_composer_musical::Scale;
    /// # use redact_composer_musical::{Interval, IntervalStepSequence};
    /// let (w, h) = (Interval(2), Interval(1)); // w = Whole-step, h = Half-step
    /// assert_eq!(Scale::Minor.interval_steps(), vec![w, h, w, w, w, h, w]);
    /// assert_eq!(Scale::Minor.interval_steps().into_iter().sum::<Interval>(), Interval::Octave);
    /// ```
    Minor,
    /// ```
    /// # use redact_composer_musical::Scale;
    /// # use redact_composer_musical::{Interval, IntervalStepSequence};
    /// let (w, h) = (Interval(2), Interval(1)); // w = Whole-step, h = Half-step
    /// assert_eq!(Scale::NaturalMinor.interval_steps(), vec![w, h, w, w, h, w, w]);
    /// assert_eq!(Scale::NaturalMinor.interval_steps().into_iter().sum::<Interval>(), Interval::Octave);
    /// ```
    NaturalMinor,
    /// ```
    /// # use redact_composer_musical::Scale;
    /// # use redact_composer_musical::{Interval, IntervalStepSequence};
    /// let (w, h) = (Interval(2), Interval(1)); // w = Whole-step, h = Half-step
    /// assert_eq!(Scale::HarmonicMinor.interval_steps(), vec![w, h, w, w, h, w + h, h]);
    /// assert_eq!(Scale::HarmonicMinor.interval_steps().into_iter().sum::<Interval>(), Interval::Octave);
    /// ```
    HarmonicMinor,
}

impl IntervalStepSequence for Scale {
    fn interval_steps(&self) -> Vec<Interval> {
        let (w, h) = (Interval(2), Interval(1)); // w = Whole-step, h = Half-step

        match self {
            Scale::Major => vec![w, w, h, w, w, w, h],
            Scale::Minor => vec![w, h, w, w, w, h, w],
            Scale::NaturalMinor => vec![w, h, w, w, h, w, w],
            Scale::HarmonicMinor => vec![w, h, w, w, h, w + h, h],
        }
    }
}

impl Scale {
    /// Returns a [Vec]<[Scale]> of all types.
    pub fn values() -> Vec<Scale> {
        vec![
            Self::Major,
            Self::Minor,
            Self::NaturalMinor,
            Self::HarmonicMinor,
        ]
    }
}
