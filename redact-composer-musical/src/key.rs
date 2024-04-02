use crate::{
    Chord, ChordShape, Degree, Interval, IntervalCollection, IntervalStepSequence, Mode, Note,
    NoteIter, NoteIterator, NoteName, PitchClass, PitchClassCollection, Scale,
};
use std::hash::{Hash, Hasher};
use std::ops::RangeBounds;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Musical key signature represented as a root [`PitchClass`], [`Scale`]
/// (e.g. Major/Minor), and [`Mode`].
/// ```
/// use redact_composer_musical::{Key, Mode, NoteName::*, Scale::Major, Note, PitchClassCollection};
/// let c_major = Key::from((C, Major));
/// assert_eq!(c_major.pitch_classes(), vec![C, D, E, F, G, A, B]);
/// ```
#[derive(Debug, Clone, Copy, Eq)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Key {
    /// First pitch of the scale.
    pub(crate) root: PitchClass,
    /// The interval sequence (relative to the `root`) defining the notes this [Key].
    pub(crate) scale: Scale,
    /// Offset amount for the scale.
    pub(crate) mode: Mode,
    /// The preferred [`NoteName`] when naming notes in this key.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub(crate) name_pref: Option<NoteName>,
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root && self.scale == other.scale && self.mode == other.mode
    }
}

impl Hash for Key {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.root.hash(state);
        self.scale.hash(state);
        self.mode.hash(state);
    }
}

impl From<(PitchClass, Scale, Mode)> for Key {
    fn from(value: (PitchClass, Scale, Mode)) -> Self {
        let (root, scale, mode, name_pref) = (value.0, value.1, value.2, None);

        Key {
            root,
            scale,
            mode,
            name_pref,
        }
    }
}

impl From<(PitchClass, Scale)> for Key {
    fn from(value: (PitchClass, Scale)) -> Self {
        Self::from((value.0, value.1, Mode::default()))
    }
}

impl From<(NoteName, Scale, Mode)> for Key {
    fn from(value: (NoteName, Scale, Mode)) -> Self {
        let (tonic, scale, mode, name_pref) = (
            PitchClass::from(value.0),
            value.1,
            value.2,
            Some(Self::simplify_root(value.0)),
        );

        Key {
            root: tonic,
            scale,
            mode,
            name_pref,
        }
    }
}

impl From<(NoteName, Scale)> for Key {
    fn from(value: (NoteName, Scale)) -> Self {
        Self::from((value.0, value.1, Mode::default()))
    }
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
            .map(|i| self.root + i)
            .collect()
    }
}

impl NoteIterator for Key {
    fn iter_notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> NoteIter<R> {
        NoteIter::from((self.root, self.intervals(), note_range))
    }
}

impl Key {
    /// Creates a [`Key`] from a [`PitchClass`], [`Scale`], and [`Mode`].
    /// Alternatively, several [`From`] implementations are supported:
    /// ```
    /// use redact_composer_musical::{Key, Mode::Ionian, NoteName::C, PitchClass, Scale::Major};
    ///
    /// // All the ways to define C Major (Ionian)
    /// let first = Key::new(PitchClass(0), Major, Ionian);
    /// let second = Key::from((PitchClass(0), Major));
    /// let third = Key::from((PitchClass(0), Major, Ionian));
    /// let fourth = Key::from((C, Major));
    /// let fifth = Key::from((C, Major, Ionian));
    ///
    /// assert!([second, third, fourth, fifth].into_iter().all(|k| k == first));
    /// ```
    pub fn new(root: PitchClass, scale: Scale, mode: Mode) -> Key {
        Key {
            root,
            scale,
            mode,
            name_pref: None,
        }
    }

    /// Returns the key's root [`PitchClass`].
    pub fn root(&self) -> PitchClass {
        self.root
    }

    /// Returns the key's [`Scale`].
    pub fn scale(&self) -> Scale {
        self.scale
    }

    /// Returns the key's [`Mode`].
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Returns the chords that use notes exclusively from this key.
    pub fn chords(&self) -> Vec<Chord> {
        self.chords_with_shape(ChordShape::all())
    }

    /// Returns chords of the given shapes which use notes exclusively from this key.
    /// ```
    /// use redact_composer_musical::{Key, NoteName::*, Chord, ChordShape, ChordShape::{dim, maj, min}, Scale, Mode};
    /// use redact_composer_musical::Scale::Major;
    ///
    /// let key = Key::from((C, Major));
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
            .filter(|chord| self.contains(chord))
            .collect()
    }

    /// Checks if all [`PitchClass`]s from a collection (for example, [`Chord`]) belong to this key.
    /// ```
    /// use redact_composer_musical::{Chord, ChordShape::{maj, min}, Key, Mode::Ionian, NoteName::*, Scale::Major};
    ///
    /// assert!(Key::from((C, Major)).contains(&Chord::from((C, maj))));
    /// assert!(!Key::from((C, Major)).contains(&Chord::from((C, min))));
    /// ```
    pub fn contains<P: PitchClassCollection>(&self, pitches: &P) -> bool {
        let scale_pitches = self.pitch_classes();
        pitches
            .pitch_classes()
            .iter()
            .all(|pitch| scale_pitches.contains(pitch))
    }

    /// Returns the pitch class for a given degree of this scale.
    /// ```
    /// use redact_composer_musical::{Degree, Key, Scale::Major, Mode::Locrian, NoteName::{B, D}};
    ///
    /// let key = Key::from((B, Major, Locrian));
    /// assert_eq!(key.relative_pitch(Degree::III), D);
    /// ```
    pub fn relative_pitch<D: Into<Degree>>(&self, degree: D) -> PitchClass {
        self.root + self.intervals()[degree.into() as usize]
    }
}

#[cfg(test)]
mod tests {
    use crate::NoteName::C;
    use crate::{Key, Note, NoteIterator, Scale::*};

    #[test]
    fn middle_c_major_scale() {
        assert_eq!(
            Key::from((C, Major)).notes_in_range(Note(60)..=Note(72)),
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
    fn middle_c_natural_minor_scale() {
        assert_eq!(
            Key::from((C, NaturalMinor)).notes_in_range(Note(60)..=Note(72)),
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
    fn middle_c_melodic_minor_scale() {
        assert_eq!(
            Key::from((C, NaturalMinor)).notes_in_range(Note(60)..=Note(72)),
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
            Key::from((C, HarmonicMinor)).notes_in_range(Note(60)..=Note(72)),
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
