use crate::{Interval, Note, NoteIter, NoteIterator, NoteName, PitchClassCollection};
use std::ops::{Add, AddAssign, RangeBounds, Sub, SubAssign};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// An octave-independent note.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
pub struct PitchClass(pub u8);

impl PitchClass {
    /// Returns all valid [`PitchClass`] values (0..=11).
    pub fn values() -> Vec<PitchClass> {
        (0..12).map(PitchClass::from).collect()
    }

    /// Returns all pitch classes contained in the given note range.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{C, F}, PitchClass as PC};
    ///
    /// assert_eq!(
    ///     PC::all_in_range(Note::from((C, 3))..=Note::from((F, 3))),
    ///     vec![PC(0), PC(1), PC(2), PC(3), PC(4), PC(5)]
    /// );
    ///
    /// assert_eq!(
    ///     PC::all_in_range(Note::from((C, 3))..=Note::from((C, 5))),
    ///     PC::values()
    /// );
    /// ```
    pub fn all_in_range<R: RangeBounds<Note>>(range: R) -> Vec<PitchClass> {
        NoteIter::chromatic(range)
            .take(12)
            .map(|note| note.pitch_class())
            .collect()
    }

    /// Returns the [`Note`] with this pitch class in a given octave.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(C).in_octave(4), Note(60));
    /// ```
    pub fn in_octave(&self, octave: i8) -> Note {
        (*self, octave).into()
    }

    /// Returns the next [`Note`] of this pitch class above the given note. If the given note is already of this pitch
    /// class, the note an octave above is returned.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(G).above(&Note::from((C, 3))), Note::from((G, 3)));
    /// assert_eq!(PitchClass::from(G).above(&Note::from((G, 3))), Note::from((G, 4)));
    /// ```
    pub fn above(&self, note: &Note) -> Note {
        let pitch_in_same_octave = Note::from((*self, note.octave()));
        if pitch_in_same_octave > *note {
            pitch_in_same_octave
        } else {
            pitch_in_same_octave + Interval::Octave
        }
    }

    /// Returns the next [`Note`] of this pitch class at or above the given note.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(G).at_or_above(&Note::from((C, 3))), Note::from((G, 3)));
    /// assert_eq!(PitchClass::from(G).at_or_above(&Note::from((G, 3))), Note::from((G, 3)));
    /// ```
    pub fn at_or_above(&self, note: &Note) -> Note {
        if *note == *self {
            *note
        } else {
            self.above(note)
        }
    }

    /// Returns the next [`Note`] of this pitch class below the given note. If the given note is already of this pitch
    /// class, the note an octave below is returned.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(G).below(&Note::from((C, 3))), Note::from((G, 2)));
    /// assert_eq!(PitchClass::from(G).below(&Note::from((G, 3))), Note::from((G, 2)));
    /// ```
    pub fn below(&self, note: &Note) -> Note {
        let pitch_in_same_octave = Note::from((*self, note.octave()));
        if pitch_in_same_octave < *note {
            pitch_in_same_octave
        } else {
            pitch_in_same_octave - Interval::Octave
        }
    }

    /// Returns the next [`Note`] of this pitch class at or below the given note.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(G).at_or_below(&Note::from((C, 3))), Note::from((G, 2)));
    /// assert_eq!(PitchClass::from(G).at_or_below(&Note::from((G, 3))), Note::from((G, 3)));
    /// ```
    pub fn at_or_below(&self, note: &Note) -> Note {
        if *note == *self {
            *note
        } else {
            self.below(note)
        }
    }

    /// Returns the simple interval (ascending) from this pitch class to the nearest `other` pitch class.
    /// ```
    /// use redact_composer_musical::{Interval, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(C).interval_to(&G.into()), Interval::P5);
    /// ```
    pub fn interval_to(&self, other: &PitchClass) -> Interval {
        let (first, second) = if self.0 <= other.0 {
            (self.0, other.0)
        } else {
            (self.0, other.0 + 12)
        };

        Interval(second - first)
    }

    /// Returns the simple interval (ascending) from some `other` pitch class to this one.
    /// ```
    /// use redact_composer_musical::{Interval, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(C).interval_from(&G.into()), Interval::P4);
    /// ```
    pub fn interval_from(&self, other: &PitchClass) -> Interval {
        self.interval_to(other).inversion()
    }
}

impl NoteIterator for PitchClass {
    fn iter_notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> NoteIter<R> {
        NoteIter::from((*self, vec![Interval::P1], note_range))
    }
}

impl PitchClassCollection for PitchClass {
    fn pitch_classes(&self) -> Vec<PitchClass> {
        vec![*self]
    }
}

impl From<NoteName> for PitchClass {
    fn from(value: NoteName) -> Self {
        PitchClass(value.into())
    }
}

impl From<Note> for PitchClass {
    fn from(value: Note) -> Self {
        value.pitch_class()
    }
}

impl From<u8> for PitchClass {
    fn from(value: u8) -> Self {
        Self(value % 12)
    }
}

impl PartialEq<NoteName> for PitchClass {
    /// ```
    /// use redact_composer_musical::{NoteName::C, PitchClass};
    /// assert!(PitchClass(0).eq(&C));
    /// ```
    fn eq(&self, note_name: &NoteName) -> bool {
        *self == PitchClass::from(*note_name)
    }
}

impl Add<Interval> for PitchClass {
    type Output = PitchClass;

    /// Returns the pitch class a given interval above this.
    /// ```
    /// use redact_composer_musical::{Interval as I, NoteName::{C, G}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(C) + I::P5, PitchClass::from(G));
    /// assert_eq!(PitchClass::from(C) + I::Octave, PitchClass::from(C));
    /// ```
    fn add(self, rhs: Interval) -> Self::Output {
        let mut output = self;
        output += rhs;
        output
    }
}

impl AddAssign<Interval> for PitchClass {
    fn add_assign(&mut self, rhs: Interval) {
        self.0 = (self.0 + rhs.to_simple().0) % 12;
    }
}

impl Sub<Interval> for PitchClass {
    type Output = PitchClass;

    /// Returns the pitch class a given interval below this.
    /// ```
    /// use redact_composer_musical::{Interval as I, NoteName::{C, F}, PitchClass};
    ///
    /// assert_eq!(PitchClass::from(C) - I::P5, PitchClass::from(F));
    /// assert_eq!(PitchClass::from(C) - I::Octave, PitchClass::from(C));
    /// ```
    fn sub(self, rhs: Interval) -> Self::Output {
        let mut output = self;
        output -= rhs;
        output
    }
}

impl SubAssign<Interval> for PitchClass {
    fn sub_assign(&mut self, rhs: Interval) {
        self.0 = (self.0 + 12 - rhs.to_simple().0) % 12;
    }
}
