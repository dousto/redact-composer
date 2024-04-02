use crate::{Interval, PitchClass};
use std::ops::{Add, AddAssign, Sub, SubAssign};

mod note_name;
pub use note_name::*;

mod iter;
pub use iter::*;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::{derive::Element, elements::PlayNote};

/// A musical note, corresponding to a specific frequency in 12 tone equal temperament space.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
pub struct Note(pub u8);

impl Note {
    /// Returns the note's [`PitchClass`].
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C, PitchClass};
    ///
    /// assert_eq!(Note::from((C, 4)).pitch_class(), PitchClass(0));
    /// ```
    pub fn pitch_class(&self) -> PitchClass {
        self.0.into()
    }

    /// Returns the octave number of this note:
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{B, C}};
    ///
    /// assert_eq!(Note::from((C, 0)).octave(), 0);
    /// assert_eq!(Note::from((B, 5)).octave(), 5);
    /// ```
    pub fn octave(&self) -> i8 {
        (self.0 / 12) as i8 - 1
    }

    /// Returns the interval between this note and another.
    /// ```
    /// use redact_composer_musical::{Interval, Note, NoteName::C};
    ///
    /// assert_eq!(Note::from((C, 3)).interval_with(&Note::from((C, 4))), Interval::Octave);
    /// ```
    pub fn interval_with(&self, other_note: &Note) -> Interval {
        let (lower, higher) = if self.0 <= other_note.0 {
            (self.0, other_note.0)
        } else {
            (other_note.0, self.0)
        };

        Interval(higher - lower)
    }
}

impl From<(NoteName, i8)> for Note {
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C};
    ///
    /// assert_eq!(Note::from((C, 4)), Note(60));
    /// ```
    fn from(value: (NoteName, i8)) -> Self {
        Note::from((PitchClass::from(value.0), value.1))
    }
}

impl From<(PitchClass, i8)> for Note {
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C, PitchClass};
    ///
    /// let c3 = Note::from((PitchClass(0), 4));
    /// assert_eq!(c3, Note(60));
    /// ```
    fn from(value: (PitchClass, i8)) -> Self {
        let (note, octave) = (value.0 .0, value.1 + 1);
        Note(note + 12 * octave as u8)
    }
}

impl Add<Interval> for Note {
    type Output = Note;
    /// Adds an interval to this note, resulting in another note.
    /// ```
    /// use redact_composer_musical::{Interval as I, Note, NoteName::{C, G}};
    ///
    /// let c3 = Note::from((C, 4));
    /// assert_eq!(c3, Note(60));
    /// assert_eq!(c3 + I::P5, Note::from((G, 4)));
    /// ```
    fn add(self, rhs: Interval) -> Self::Output {
        Note(self.0 + rhs.0)
    }
}

impl AddAssign<Interval> for Note {
    fn add_assign(&mut self, rhs: Interval) {
        self.0 = self.0 + rhs.0;
    }
}

impl Sub<Interval> for Note {
    type Output = Note;
    /// Subtracts an interval from this note, resulting in another note.
    /// ```
    /// use redact_composer_musical::{Interval as I, Note, NoteName::{C, G}};
    ///
    /// let c3 = Note::from((C, 3));
    /// assert_eq!(c3 - I::P4, Note::from((G, 2)));
    /// ```
    fn sub(self, rhs: Interval) -> Self::Output {
        Note(self.0 - rhs.0)
    }
}

impl SubAssign<Interval> for Note {
    fn sub_assign(&mut self, rhs: Interval) {
        self.0 = self.0 - rhs.0;
    }
}

impl PartialEq<PitchClass> for Note {
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C, PitchClass};
    ///
    /// assert!(Note::from((C, 3)) == PitchClass::from(C));
    /// ```
    fn eq(&self, other: &PitchClass) -> bool {
        &self.pitch_class() == other
    }
}

impl PartialEq<Note> for (NoteName, i8) {
    /// ```
    /// use redact_composer_musical::{Note, NoteName::*};
    /// assert_eq!((C, 4), Note(60));
    /// ```
    fn eq(&self, other: &Note) -> bool {
        Note::from(*self).eq(other)
    }
}

impl PartialEq<(NoteName, i8)> for Note {
    /// ```
    /// use redact_composer_musical::{Note, NoteName::*};
    /// assert_eq!(Note(60), (C, 4));
    /// ```
    fn eq(&self, other: &(NoteName, i8)) -> bool {
        self.eq(&Note::from(*other))
    }
}

#[cfg(feature = "redact-composer")]
impl Note {
    /// Creates a [`PlayNote`] element from this note, which can then be used as a [`Segment`](redact_composer_core::Segment).
    /// ```
    /// use redact_composer_core::{elements::PlayNote, timing::Timing, IntoSegment};
    /// use redact_composer_musical::{Note, NoteName::C};
    ///
    /// let expected_play_note = PlayNote { note: 60, velocity: 100};
    /// let play_note = Note::from((C, 4)).play(100);
    /// assert_eq!(play_note, expected_play_note);
    ///
    ///let segment = play_note.into_segment(0..480);
    /// assert_eq!(segment.element_as::<PlayNote>(), Some(&expected_play_note));
    /// assert_eq!(segment.timing, Timing::from(0..480));
    /// ```
    pub fn play(&self, velocity: u8) -> PlayNote {
        PlayNote {
            note: self.0,
            velocity,
        }
    }
}
