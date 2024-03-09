use crate::{
    Chord, ChordShape, Degree, Interval, IntervalCollection, IntervalStepSequence, Mode, Note,
    NoteIter, NoteIterator, PitchClass, PitchClassCollection, Scale,
};
use std::ops::RangeBounds;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Musical key signature represented as a tonic ([`PitchClass`]), [`Scale`]
/// (e.g. Major/Minor), and [`Mode`].
/// ```
/// use redact_composer_musical::{Key, Mode, NoteName::*, Scale::Major, Note, PitchClassCollection};
/// let c_major = Key { tonic: C.into(), scale: Major, mode: Mode::default() };
/// assert_eq!(c_major.pitch_classes(), vec![C, D, E, F, G, A, B]);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    /// First note of the scale.
    pub tonic: PitchClass,
    /// The interval sequence (relative to the `tonic`) defining the notes this [Key].
    pub scale: Scale,
    /// Offset amount for the scale.
    pub mode: Mode,
}

impl IntervalStepSequence for Key {
    fn interval_steps(&self) -> Vec<Interval> {
        let steps = self.scale.interval_steps();
        let num_steps = steps.len();
        steps
            .into_iter()
            .cycle()
            .skip(self.mode as usize)
            .take(num_steps)
            .collect()
    }
}

impl PitchClassCollection for Key {
    fn pitch_classes(&self) -> Vec<PitchClass> {
        self.intervals()
            .into_iter()
            .map(|i| self.tonic + i)
            .collect()
    }
}

impl NoteIterator for Key {
    fn iter_notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> NoteIter<R> {
        NoteIter::from((self.tonic, self.intervals(), note_range))
    }
}

impl Key {
    /// Returns the chords that use notes exclusively from this key.
    pub fn chords(&self) -> Vec<Chord> {
        self.chords_with_shape(ChordShape::all())
    }

    /// Returns chords of the given shapes which use notes exclusively from this key.
    /// ```
    /// use redact_composer_musical::{Key, NoteName::*, Chord, ChordShape, ChordShape::{dim, maj, min}, Scale, Mode};
    ///
    /// let key = Key { tonic: C.into(), scale: Scale::Major, mode: Mode::default() };
    /// assert_eq!(
    ///     key.chords_with_shape(ChordShape::triad()),
    ///     vec![
    ///         Chord::from((C, maj)),
    ///         Chord::from((D, min)),
    ///         Chord::from((E, min)),
    ///         Chord::from((F, maj)),
    ///         Chord::from((G, maj)),
    ///         Chord::from((A, min)),
    ///         Chord::from((B, dim)),
    ///     ]
    /// )
    /// ```
    pub fn chords_with_shape(&self, shape: Vec<ChordShape>) -> Vec<Chord> {
        Degree::values()
            .into_iter()
            .map(|d| self.relative_pitch(d))
            .flat_map(|root| shape.iter().map(move |chord_shape| (root, *chord_shape)))
            .map(Chord::from)
            .filter(|chord| self.contains_chord(chord))
            .collect()
    }

    /// Checks if a chord uses notes exclusively from this key.
    /// ```
    /// use redact_composer_musical::{ChordShape::maj, Key, Mode, NoteName::*, Scale};
    ///
    /// assert!(
    ///     Key { tonic: B.into(), scale: Scale::Major, mode: Mode::Locrian }
    ///         .contains_chord(&(C, maj).into())
    /// );
    /// ```
    pub fn contains_chord(&self, chord: &Chord) -> bool {
        let scale_pitches = self.pitch_classes();
        chord
            .pitch_classes()
            .iter()
            .all(|chord_pitch| scale_pitches.contains(chord_pitch))
    }

    /// Returns the pitch class for a given degree of this scale.
    /// ```
    /// use redact_composer_musical::{Degree, Key, Scale, Mode::Locrian, NoteName::{B, D}};
    ///
    /// let key = Key { tonic: B.into(), scale: Scale::Major, mode: Locrian };
    /// assert_eq!(key.relative_pitch(Degree::III), D);
    /// ```
    pub fn relative_pitch<D: Into<Degree>>(&self, degree: D) -> PitchClass {
        self.tonic + self.intervals()[degree.into() as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::NoteName::C;
    use crate::{Key, Mode, Note, NoteIterator, Scale};

    #[test]
    fn middle_c_major_scale() {
        assert_eq!(
            Key {
                tonic: C.into(),
                scale: Scale::Major,
                mode: Mode::default()
            }
            .notes_in_range(Note(60)..=Note(72)),
            [
                Note(60),
                Note(62),
                Note(64),
                Note(65),
                Note(67),
                Note(69),
                Note(71),
                Note(72)
            ]
        )
    }

    #[test]
    fn middle_c_minor_scale() {
        assert_eq!(
            Key {
                tonic: C.into(),
                scale: Scale::Minor,
                mode: Mode::default()
            }
            .notes_in_range(Note(60)..=Note(72)),
            [
                Note(60),
                Note(62),
                Note(63),
                Note(65),
                Note(67),
                Note(69),
                Note(70),
                Note(72)
            ]
        )
    }

    #[test]
    fn middle_c_natural_minor_scale() {
        assert_eq!(
            Key {
                tonic: C.into(),
                scale: Scale::NaturalMinor,
                mode: Mode::default()
            }
            .notes_in_range(Note(60)..=Note(72)),
            [
                Note(60),
                Note(62),
                Note(63),
                Note(65),
                Note(67),
                Note(68),
                Note(70),
                Note(72)
            ]
        )
    }

    #[test]
    fn middle_c_harmonic_minor_scale() {
        assert_eq!(
            Key {
                tonic: C.into(),
                scale: Scale::HarmonicMinor,
                mode: Mode::default()
            }
            .notes_in_range(Note(60)..=Note(72)),
            [
                Note(60),
                Note(62),
                Note(63),
                Note(65),
                Note(67),
                Note(68),
                Note(71),
                Note(72)
            ]
        )
    }
}
