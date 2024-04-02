#![deny(missing_docs, missing_debug_implementations)]
//! Musical domain library.

/// Utilities for building or generating rhythms.
pub mod rhythm;

mod timing;
pub use timing::*;

mod chord;
pub use chord::*;

mod pitch_class;
pub use pitch_class::*;

mod note;
pub use note::*;

mod interval;
pub use interval::*;

mod key;
pub use key::*;

mod scale;
pub use scale::*;

/// Types implementing [`Element`](redact_composer_core::Element).
#[cfg(feature = "redact-composer")]
pub mod elements {
    pub use super::{
        rhythm::Rhythm, Chord, ChordShape, Degree, Interval, Key, Mode, Note, NoteName, PitchClass,
        Scale, TimeSignature,
    };
}

/// Provides a sequence of intervals, representing the interval *deltas* from one note to the next.
pub trait IntervalStepSequence {
    /// Provides a sequence of intervals, representing the interval *deltas* from one to the next.
    fn interval_steps(&self) -> Vec<Interval>;
}

/// Provides a collection of intervals, each as an absolute interval from a relative pitch.
pub trait IntervalCollection {
    /// Provides a collection of intervals, each as an absolute interval from a relative pitch.
    fn intervals(&self) -> Vec<Interval>;
}

impl<T> IntervalCollection for T
where
    T: IntervalStepSequence,
{
    fn intervals(&self) -> Vec<Interval> {
        let mut intervals = self
            .interval_steps()
            .into_iter()
            .fold(
                (vec![Interval::P1], Interval::P1),
                |(mut intervals, mut last), step| {
                    last += step;
                    intervals.push(last);

                    (intervals, last)
                },
            )
            .0;

        if intervals.len() > 1 && (intervals[intervals.len() - 1].0 - intervals[0].0) % 12 == 0 {
            intervals.pop();
        }

        intervals
    }
}

/// Trait implemented for types which represent or provide a collection of pitch classes.
pub trait PitchClassCollection {
    /// Returns this type's pitches.
    fn pitch_classes(&self) -> Vec<PitchClass>;
}

impl<P: Into<PitchClass> + Copy, I: IntoIterator<Item = P> + Clone> PitchClassCollection for I {
    fn pitch_classes(&self) -> Vec<PitchClass> {
        self.clone().into_iter().map(|p| p.into()).collect()
    }
}
