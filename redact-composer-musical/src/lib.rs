#![deny(missing_docs, missing_debug_implementations)]
//! Musical domain library.

/// Utilities for building or generating rhythms.
pub mod rhythm;

/// Musical timing elements. Namely [`TimeSignature`](timing::TimeSignature).
pub mod timing;

use std::fmt;

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

/// Types implementing [`Element`](redact_composer_core::Element).
#[cfg(feature = "redact-composer")]
pub mod elements {
    pub use super::{timing::*, Chord, Key, Mode, Scale};
}

/// Utility struct used for operating with a set of base notes ([u8] values `0..=11`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notes {
    /// Set of [u8] note values intended -- but not enforced -- to be between `0..=11`.
    /// These correspond to the 12 tones of a chromatic scale, with 0 representing C.
    base_notes: Vec<u8>,
}

impl Notes {
    /// Scales the set of base notes, producing all note values
    /// of the same pitch classes within the given range.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use redact_composer_musical::elements::{Key, Scale, Mode};
    /// # use redact_composer_musical::Notes;
    /// let c_major = Key { tonic: 0, scale: Scale::Major, mode: Mode::Ionian};
    /// let c_major_scale_notes = Notes::from(c_major.scale()).in_range(60..=72);
    /// assert_eq!(c_major_scale_notes, [60, 62, 64, 65, 67, 69, 71, 72]);
    /// ```
    pub fn in_range<R>(&self, range: R) -> Vec<u8>
    where
        R: IntoIterator<Item = u8>,
    {
        range
            .into_iter()
            .filter(|n| self.base_notes.contains(&(n % 12)))
            .collect()
    }

    /// Returns the 0..12 pitch class of a note.
    pub fn base_note(note: &u8) -> u8 {
        note % 12
    }
}

impl<T, K> From<T> for Notes
where
    T: IntoIterator<Item = K>,
    K: Into<u8>,
{
    /// Create a note set from a [`u8`] iterable.
    ///
    /// # Example
    /// ```rust
    /// # use redact_composer_musical::Notes;
    /// let notes = Notes::from([1,2,3]);
    /// ```
    fn from(value: T) -> Self {
        let mut clamped_base_notes: Vec<u8> = value.into_iter().map(|n| n.into() % 12).collect();
        clamped_base_notes.sort_unstable();
        clamped_base_notes.dedup();

        Notes {
            base_notes: clamped_base_notes,
        }
    }
}

/// Musical key signature represented as a tonic ([`u8`] value in `0..=11`), [`Scale`]
/// (e.g. Major/Minor), and [`Mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    /// First note of the scale. (`tonic == 0` represents C)
    pub tonic: u8,
    /// The interval sequence (relative to the `tonic`) defining the base notes this [Key].
    pub scale: Scale,
    /// Offset amount for the scale.
    pub mode: Mode,
}

impl Key {
    /// Returns the scale notes for this [Key], starting from the `tonic` and using relative intervals
    /// as determined by the [Scale].
    pub fn scale(&self) -> Vec<u8> {
        self.scale
            .relative_pitches(&self.mode)
            .iter()
            .map(|p| ((self.tonic % 12) + p) % 12)
            .collect()
    }

    /// Returns the diatonic base notes for a given [Chord] in this [Key].
    ///
    /// # Example
    /// ```rust
    /// # use redact_composer_musical::elements::{Key, Scale, Chord};
    /// let c_major = Key { tonic: 0, scale: Scale::Major, mode: Default::default() };
    /// let c_major_chord_notes = c_major.chord(&Chord::I);
    /// assert_eq!(c_major_chord_notes, [0, 4, 7]); // C, E, G
    /// ```
    pub fn chord(&self, chord: &Chord) -> Vec<u8> {
        let scale = self.scale();
        chord
            .degrees()
            .iter()
            .map(|d| scale[usize::from(*d)])
            .collect()
    }

    /// Returns the note value for a scale degree.
    pub fn note(&self, degree: u8) -> u8 {
        let scale = self.scale();

        scale[usize::from(degree) % scale.len()]
    }
}

/// Musical chords defined relative to scale degrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Chord {
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::I.degrees(), vec![0, 2, 4])
    /// ```
    I,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::II.degrees(), vec![1, 3, 5])
    /// ```
    II,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::III.degrees(), vec![2, 4, 6])
    /// ```
    III,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::IV.degrees(), vec![3, 5, 0])
    /// ```
    IV,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::V.degrees(), vec![4, 6, 1])
    /// ```
    V,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::VI.degrees(), vec![5, 0, 2])
    /// ```
    VI,
    /// ```rust
    /// # use redact_composer_musical::elements::Chord;
    /// assert_eq!(Chord::VII.degrees(), vec![6, 1, 3])
    /// ```
    VII,
}

impl Chord {
    const I_STR: &'static str = "I";
    const II_STR: &'static str = "II";
    const III_STR: &'static str = "III";
    const IV_STR: &'static str = "IV";
    const V_STR: &'static str = "V";
    const VI_STR: &'static str = "VI";
    const VII_STR: &'static str = "VII";

    /// Returns a [Vec]<[Chord]> of all types.
    pub fn values() -> Vec<Chord> {
        vec![
            Self::I,
            Self::II,
            Self::III,
            Self::IV,
            Self::V,
            Self::VI,
            Self::VII,
        ]
    }

    /// Returns the diatonic degrees (scale notes) represented by this [Chord].
    pub fn degrees(&self) -> Vec<u8> {
        vec![self.root(), self.third(), self.fifth()]
    }

    /// Returns the scale degree of chord's root note.
    pub fn root(&self) -> u8 {
        match self {
            Chord::I => 0,
            Chord::II => 1,
            Chord::III => 2,
            Chord::IV => 3,
            Chord::V => 4,
            Chord::VI => 5,
            Chord::VII => 6,
        }
    }

    /// Returns the scale degree of chord's third interval.
    pub fn third(&self) -> u8 {
        match self {
            Chord::I => 2,
            Chord::II => 3,
            Chord::III => 4,
            Chord::IV => 5,
            Chord::V => 6,
            Chord::VI => 0,
            Chord::VII => 1,
        }
    }

    /// Returns the scale degree of chord's fifth interval.
    pub fn fifth(&self) -> u8 {
        match self {
            Chord::I => 4,
            Chord::II => 5,
            Chord::III => 6,
            Chord::IV => 0,
            Chord::V => 1,
            Chord::VI => 2,
            Chord::VII => 3,
        }
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(*self))
    }
}

impl From<Chord> for String {
    fn from(value: Chord) -> Self {
        match value {
            Chord::I => Chord::I_STR,
            Chord::II => Chord::II_STR,
            Chord::III => Chord::III_STR,
            Chord::IV => Chord::IV_STR,
            Chord::V => Chord::V_STR,
            Chord::VI => Chord::VI_STR,
            Chord::VII => Chord::VII_STR,
        }
        .into()
    }
}

impl From<&Chord> for String {
    fn from(value: &Chord) -> Self {
        Self::from(*value)
    }
}

impl From<&str> for Chord {
    fn from(value: &str) -> Self {
        match value {
            Self::I_STR => Self::I,
            Self::II_STR => Self::II,
            Self::III_STR => Self::III,
            Self::IV_STR => Self::IV,
            Self::V_STR => Self::V,
            Self::VI_STR => Self::VI,
            Self::VII_STR => Self::VII,
            _ => panic!(),
        }
    }
}

impl From<String> for Chord {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for Chord {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

/// Sequence of intervals spanning 12 semitones or one octave.
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Scale {
    /// ```rust
    /// # use redact_composer_musical::elements::Scale;
    /// assert_eq!(Scale::Major.relative_pitches(&Default::default()), vec![0, 2, 4, 5, 7, 9, 11])
    /// ```
    Major,
    /// ```rust
    /// # use redact_composer_musical::elements::Scale;
    /// assert_eq!(Scale::Minor.relative_pitches(&Default::default()), vec![0, 2, 3, 5, 7, 9, 10])
    /// ```
    Minor,
    /// ```rust
    /// # use redact_composer_musical::elements::Scale;
    /// assert_eq!(Scale::NaturalMinor.relative_pitches(&Default::default()), vec![0, 2, 3, 5, 7, 8, 10])
    /// ```
    NaturalMinor,
    /// ```rust
    /// # use redact_composer_musical::elements::Scale;
    /// assert_eq!(Scale::HarmonicMinor.relative_pitches(&Default::default()), vec![0, 2, 3, 5, 7, 8, 11])
    /// ```
    HarmonicMinor,
}

impl Scale {
    const MAJOR_STR: &'static str = "Major";
    const MINOR_STR: &'static str = "Minor";
    const NATURAL_MINOR_STR: &'static str = "NaturalMinor";
    const HARMONIC_MINOR_STR: &'static str = "HarmonicMinor";

    /// Returns a [Vec]<[Scale]> of all types.
    pub fn values() -> Vec<Scale> {
        vec![
            Self::Major,
            Self::Minor,
            Self::NaturalMinor,
            Self::HarmonicMinor,
        ]
    }

    /// Returns the pitches of this [Scale] (note offset relative to tonic).
    pub fn relative_pitches(&self, mode: &Mode) -> Vec<u8> {
        match self {
            Scale::Major => vec![0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => vec![0, 2, 3, 5, 7, 9, 10],
            Scale::NaturalMinor => vec![0, 2, 3, 5, 7, 8, 10],
            Scale::HarmonicMinor => vec![0, 2, 3, 5, 7, 8, 11],
        }
        .into_iter()
        .cycle()
        .skip(*mode as usize)
        .take(7)
        .collect::<Vec<_>>()
    }
}

impl fmt::Display for Scale {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

impl From<Scale> for String {
    fn from(value: Scale) -> Self {
        match value {
            Scale::Major => Scale::MAJOR_STR,
            Scale::Minor => Scale::MINOR_STR,
            Scale::NaturalMinor => Scale::NATURAL_MINOR_STR,
            Scale::HarmonicMinor => Scale::HARMONIC_MINOR_STR,
        }
        .into()
    }
}

impl From<&Scale> for String {
    fn from(value: &Scale) -> Self {
        Self::from(*value)
    }
}

impl From<&str> for Scale {
    fn from(value: &str) -> Self {
        match value {
            Self::MAJOR_STR => Self::Major,
            Self::MINOR_STR => Self::Minor,
            Self::NATURAL_MINOR_STR => Self::NaturalMinor,
            Self::HARMONIC_MINOR_STR => Self::HarmonicMinor,
            _ => panic!(),
        }
    }
}

impl From<String> for Scale {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&String> for Scale {
    fn from(value: &String) -> Self {
        Self::from(value.as_str())
    }
}

/// Offset applied to a [`Scale`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Mode {
    /// No offset
    #[default]
    Ionian,
    /// Offset of 1, starting a scale on the second pitch.
    Dorian,
    /// Offset of 2, starting a scale on the third pitch.
    Phrygian,
    /// Offset of 3, starting a scale on the fourth pitch.
    Lydian,
    /// Offset of 4, starting a scale on the fifth pitch.
    Mixolydian,
    /// Offset of 5, starting a scale on the sixth pitch.
    Aeolian,
    /// Offset of 6, starting a scale on the seventh pitch.
    Locrian,
}

impl Mode {
    /// Returns a [Vec]<[Mode]> of all types.
    pub fn values() -> Vec<Mode> {
        vec![
            Self::Ionian,
            Self::Dorian,
            Self::Phrygian,
            Self::Lydian,
            Self::Mixolydian,
            Self::Aeolian,
            Self::Locrian,
        ]
    }
}
