use crate::{Note, PitchClass};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Musical note name.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[allow(missing_docs)]
pub enum NoteName {
    Ab,
    A,
    As,
    Bb,
    B,
    Bs,
    Cb,
    C,
    Cs,
    Db,
    D,
    Ds,
    Eb,
    E,
    Es,
    Fb,
    F,
    Fs,
    Gb,
    G,
    Gs,
}

impl NoteName {
    /// Returns this note name as a [`Note`] in a given octave.
    /// ```
    /// use redact_composer_musical::{Note, NoteName::C};
    /// assert_eq!(C.in_octave(4), Note(60));
    /// ```
    pub fn in_octave(&self, octave: i8) -> Note {
        (*self, octave).into()
    }
}

impl From<NoteName> for u8 {
    fn from(note: NoteName) -> Self {
        match note {
            NoteName::Cb => 11,
            NoteName::C => 0,
            NoteName::Cs => 1,
            NoteName::Db => 1,
            NoteName::D => 2,
            NoteName::Ds => 3,
            NoteName::Eb => 3,
            NoteName::E => 4,
            NoteName::Es => 5,
            NoteName::Fb => 4,
            NoteName::F => 5,
            NoteName::Fs => 6,
            NoteName::Gb => 6,
            NoteName::G => 7,
            NoteName::Gs => 8,
            NoteName::Ab => 8,
            NoteName::A => 9,
            NoteName::As => 10,
            NoteName::Bb => 10,
            NoteName::B => 11,
            NoteName::Bs => 0,
        }
    }
}

impl PartialEq<PitchClass> for NoteName {
    /// ```
    /// use redact_composer_musical::{NoteName, PitchClass};
    /// assert!(NoteName::C.eq(&PitchClass(0)));
    /// ```
    fn eq(&self, pitch_class: &PitchClass) -> bool {
        PitchClass::from(*self) == *pitch_class
    }
}
