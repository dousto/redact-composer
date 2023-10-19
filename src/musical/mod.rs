use std::fmt;

use crate::composer::render::RenderEngine;
use serde::{Deserialize, Serialize};

use crate::composer::SegmentType;

pub mod midi;
pub mod rhythm;
pub mod timing;

#[cfg(test)]
mod test;

pub fn renderers() -> RenderEngine {
    midi::renderers() + timing::renderers()
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
    /// # use redact_composer::musical::{Key, Scale, Notes, Mode};
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
    /// # use redact_composer::musical::{Notes};
    /// let notes = Notes::from([1,2,3]);
    /// ```
    fn from(value: T) -> Self {
        let mut clamped_base_notes: Vec<u8> = value.into_iter().map(|n| n.into() % 12).collect();
        clamped_base_notes.sort();
        clamped_base_notes.dedup();

        Notes {
            base_notes: clamped_base_notes,
        }
    }
}

/// Represents a key signature via a tonic ([u8] value in `0..=11`), [Scale] (e.g. Major/Minor),
/// and [`Mode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Key {
    /// First note of the scale. (`tonic == 0` represents C)
    pub tonic: u8,
    /// The interval sequence (relative to the `tonic`) defining the base notes this [Key].
    pub scale: Scale,
    /// Offset amount for the scale.
    pub mode: Mode,
}

#[typetag::serde]
impl SegmentType for Key {}

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
    /// # use redact_composer::musical::{Key, Scale, Chord};
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
}

/// A type representing the diatonic harmony chord variations based on [heptatonic (7-note) scales](https://en.wikipedia.org/wiki/Heptatonic_scale).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Chord {
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::I.degrees(), vec![0, 2, 4])
    /// ```
    I,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::II.degrees(), vec![1, 3, 5])
    /// ```
    II,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::III.degrees(), vec![2, 4, 6])
    /// ```
    III,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::IV.degrees(), vec![3, 5, 0])
    /// ```
    IV,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::V.degrees(), vec![4, 6, 1])
    /// ```
    V,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::VI.degrees(), vec![5, 0, 2])
    /// ```
    VI,
    /// ```rust
    /// # use redact_composer::musical::{Chord};
    /// assert_eq!(Chord::VII.degrees(), vec![6, 1, 3])
    /// ```
    VII,
}

#[typetag::serde]
impl SegmentType for Chord {}

impl Chord {
    const I_STR: &str = "I";
    const II_STR: &str = "II";
    const III_STR: &str = "III";
    const IV_STR: &str = "IV";
    const V_STR: &str = "V";
    const VI_STR: &str = "VI";
    const VII_STR: &str = "VII";

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
        match self {
            Chord::I => vec![0, 2, 4],
            Chord::II => vec![1, 3, 5],
            Chord::III => vec![2, 4, 6],
            Chord::IV => vec![3, 5, 0],
            Chord::V => vec![4, 6, 1],
            Chord::VI => vec![5, 0, 2],
            Chord::VII => vec![6, 1, 3],
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

/// A type representing the sequence of intervals defining the notes in a [Key].
#[derive(Debug, Hash, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    /// ```rust
    /// # use redact_composer::musical::{Mode, Scale};
    /// assert_eq!(Scale::Major.relative_pitches(&Mode::Ionian), vec![0, 2, 4, 5, 7, 9, 11])
    /// ```
    Major,
    /// ```rust
    /// # use redact_composer::musical::{Mode, Scale};
    /// assert_eq!(Scale::Minor.relative_pitches(&Mode::Ionian), vec![0, 2, 3, 5, 7, 9, 10])
    /// ```
    Minor,
    /// ```rust
    /// # use redact_composer::musical::{Mode, Scale};
    /// assert_eq!(Scale::NaturalMinor.relative_pitches(&Mode::Ionian), vec![0, 2, 3, 5, 7, 8, 10])
    /// ```
    NaturalMinor,
    /// ```rust
    /// # use redact_composer::musical::{Mode, Scale};
    /// assert_eq!(Scale::HarmonicMinor.relative_pitches(&Mode::Ionian), vec![0, 2, 3, 5, 7, 8, 11])
    /// ```
    HarmonicMinor,
}

#[typetag::serde]
impl SegmentType for Scale {}

impl Scale {
    const MAJOR_STR: &str = "Major";
    const MINOR_STR: &str = "Minor";
    const NATURAL_MINOR_STR: &str = "NaturalMinor";
    const HARMONIC_MINOR_STR: &str = "HarmonicMinor";

    /// Returns a [Vec]<[Scale]> of all types.
    pub fn values() -> Vec<Scale> {
        vec![
            Self::Major,
            Self::Minor,
            Self::NaturalMinor,
            Self::HarmonicMinor,
        ]
    }

    // Returns the pitches of this [Scale] (via interval offsest from tonic).
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[typetag::serde]
impl SegmentType for Mode {}

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
