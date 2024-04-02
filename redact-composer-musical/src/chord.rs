#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;
use std::ops::RangeBounds;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    Interval, IntervalCollection, Key, Note, NoteIter, NoteIterator, PitchClass,
    PitchClassCollection,
};

/// Describes a chord using a root [`PitchClass`], and [`ChordShape`].
/// ```
/// use redact_composer_musical::{Chord, ChordShape::maj7, PitchClassCollection, NoteName::*};
///
/// assert_eq!(Chord::from((C, maj7)).pitch_classes(), [C, E, G, B]);
/// ```
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
pub struct Chord {
    /// The chord's root pitch.
    pub(crate) root: PitchClass,
    /// The chord's type (e.g. maj, min, etc..)
    pub(crate) shape: ChordShape,
}

impl NoteIterator for Chord {
    fn iter_notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> NoteIter<R> {
        NoteIter::from((self.root, self.intervals(), note_range))
    }
}

impl<R> From<(R, ChordShape)> for Chord
where
    R: Into<PitchClass>,
{
    fn from(value: (R, ChordShape)) -> Self {
        let (root_pitch_class, shape) = (value.0.into(), value.1);

        Chord {
            root: root_pitch_class,
            shape,
        }
    }
}

impl IntervalCollection for Chord {
    fn intervals(&self) -> Vec<Interval> {
        self.shape.intervals()
    }
}

impl PitchClassCollection for Chord {
    fn pitch_classes(&self) -> Vec<PitchClass> {
        self.intervals()
            .into_iter()
            .map(|i| self.root + i)
            .collect()
    }
}

impl Chord {
    /// Constructs a [`Chord`] from a root and interval collection.
    pub fn new<R: Into<PitchClass>>(root: R, shape: ChordShape) -> Chord {
        Chord::from((root, shape))
    }

    /// Checks if this chord contains notes exclusively from a particular key.
    pub fn is_in_key(&self, key: &Key) -> bool {
        key.contains(self)
    }

    /// Returns the root [`PitchClass`] of this chord.
    pub fn root(&self) -> PitchClass {
        self.root
    }

    /// Returns the [`ChordShape`] of this chord. (e.g. maj, min...)
    pub fn shape(&self) -> ChordShape {
        self.shape
    }

    /// Checks if all [`PitchClass`]s from a collection belong to this [`Chord`].
    /// ```
    /// use redact_composer_musical::{Chord, ChordShape::maj, NoteName::*};
    ///
    /// assert!(Chord::from((C, maj)).contains(&[C, E, G]));
    /// ```
    pub fn contains<P: PitchClassCollection>(&self, pitches: &P) -> bool {
        let chord_pitches = self.pitch_classes();
        pitches
            .pitch_classes()
            .iter()
            .all(|pitch| chord_pitches.contains(pitch))
    }
}

/// Chord types as interval collections.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[allow(non_camel_case_types)]
pub enum ChordShape {
    /// Major
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj.intervals(), vec![I::P1, I::M3, I::P5]);
    /// ```
    maj,
    /// Major 6th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj6.intervals(), vec![I::P1, I::M3, I::P5, I::M6]);
    /// ```
    maj6,
    /// Major 6/9
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj6_9.intervals(), vec![I::P1, I::M3, I::P5, I::M6, I::M9]);
    /// ```
    maj6_9,
    /// Major 7th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj7.intervals(), vec![I::P1, I::M3, I::P5, I::M7]);
    /// ```
    maj7,
    /// Major 9th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj9.intervals(), vec![I::P1, I::M3, I::P5, I::M7, I::M9]);
    /// ```
    maj9,
    /// Major 11th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj11.intervals(), vec![I::P1, I::M3, I::P5, I::M7, I::M9, I::P11]);
    /// ```
    maj11,
    /// Major 13th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(maj13.intervals(), vec![I::P1, I::M3, I::P5, I::M7, I::M9, I::P11, I::M13]);
    /// ```
    maj13,
    /// Minor
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min.intervals(), vec![I::P1, I::m3, I::P5]);
    /// ```
    min,
    /// Minor 6th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min6.intervals(), vec![I::P1, I::m3, I::P5, I::M6]);
    /// ```
    min6,
    /// Minor 7th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min7.intervals(), vec![I::P1, I::m3, I::P5, I::m7]);
    /// ```
    min7,
    /// Minor Major 7th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min_M7.intervals(), vec![I::P1, I::m3, I::P5, I::M7]);
    /// ```
    min_M7,
    /// Minor 9th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min9.intervals(), vec![I::P1, I::m3, I::P5, I::m7, I::M9]);
    /// ```
    min9,
    /// Minor 11th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min11.intervals(), vec![I::P1, I::m3, I::P5, I::m7, I::M9, I::P11]);
    /// ```
    min11,
    /// Minor 13th
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min13.intervals(), vec![I::P1, I::m3, I::P5, I::m7, I::M9, I::P11, I::M13]);
    /// ```
    min13,
    /// Dominant 7
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dom7.intervals(), vec![I::P1, I::M3, I::P5, I::m7]);
    /// ```
    dom7,
    /// Dominant 9
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dom9.intervals(), vec![I::P1, I::M3, I::P5, I::m7, I::M9]);
    /// ```
    dom9,
    /// Dominant 11
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dom11.intervals(), vec![I::P1, I::M3, I::P5, I::m7, I::M9, I::P11]);
    /// ```
    dom11,
    /// Dominant 13
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dom13.intervals(), vec![I::P1, I::M3, I::P5, I::m7, I::M9, I::P11, I::M13]);
    /// ```
    dom13,
    /// Diminished
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dim.intervals(), vec![I::P1, I::m3, I::d5]);
    /// ```
    dim,
    /// Diminished 7
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(dim7.intervals(), vec![I::P1, I::m3, I::d5, I::d7]);
    /// ```
    dim7,
    /// Half Diminished (min7 with flat 5)
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(min7_b5.intervals(), vec![I::P1, I::m3, I::d5, I::m7]);
    /// ```
    min7_b5,
    /// Augmented
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(aug.intervals(), vec![I::P1, I::M3, I::A5]);
    /// ```
    aug,
    /// Augmented 7
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(aug7.intervals(), vec![I::P1, I::M3, I::A5, I::m7]);
    /// ```
    aug7,
    /// Suspended 2
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(sus2.intervals(), vec![I::P1, I::M2, I::P5]);
    /// ```
    sus2,
    /// Suspended 4
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(sus4.intervals(), vec![I::P1, I::P4, I::P5]);
    /// ```
    sus4,
    /// Suspended 4 w/ m7
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(sus4_7.intervals(), vec![I::P1, I::P4, I::P5, I::m7]);
    /// ```
    sus4_7,
    /// Add 9
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(add9.intervals(), vec![I::P1, I::M3, I::P5, I::M9]);
    /// ```
    add9,
    /// Add 11
    /// ```
    /// # use redact_composer_musical::ChordShape::*;
    /// # use redact_composer_musical::IntervalCollection;
    /// use redact_composer_musical::Interval as I;
    /// assert_eq!(add11.intervals(), vec![I::P1, I::M3, I::P5, I::P11]);
    /// ```
    add11,
}

impl IntervalCollection for ChordShape {
    fn intervals(&self) -> Vec<Interval> {
        use Interval as I;

        match self {
            ChordShape::maj => vec![I::P1, I::M3, I::P5],
            ChordShape::maj6 => vec![I::P1, I::M3, I::P5, I::M6],
            ChordShape::maj6_9 => vec![I::P1, I::M3, I::P5, I::M6, I::M9],
            ChordShape::maj7 => vec![I::P1, I::M3, I::P5, I::M7],
            ChordShape::maj9 => vec![I::P1, I::M3, I::P5, I::M7, I::M9],
            ChordShape::maj11 => vec![I::P1, I::M3, I::P5, I::M7, I::M9, I::P11],
            ChordShape::maj13 => vec![I::P1, I::M3, I::P5, I::M7, I::M9, I::P11, I::M13],
            ChordShape::min => vec![I::P1, I::m3, I::P5],
            ChordShape::min6 => vec![I::P1, I::m3, I::P5, I::M6],
            ChordShape::min7 => vec![I::P1, I::m3, I::P5, I::m7],
            ChordShape::min_M7 => vec![I::P1, I::m3, I::P5, I::M7],
            ChordShape::min9 => vec![I::P1, I::m3, I::P5, I::m7, I::M9],
            ChordShape::min11 => vec![I::P1, I::m3, I::P5, I::m7, I::M9, I::P11],
            ChordShape::min13 => vec![I::P1, I::m3, I::P5, I::m7, I::M9, I::P11, I::M13],
            ChordShape::dom7 => vec![I::P1, I::M3, I::P5, I::m7],
            ChordShape::dom9 => vec![I::P1, I::M3, I::P5, I::m7, I::M9],
            ChordShape::dom11 => vec![I::P1, I::M3, I::P5, I::m7, I::M9, I::P11],
            ChordShape::dom13 => vec![I::P1, I::M3, I::P5, I::m7, I::M9, I::P11, I::M13],
            ChordShape::dim => vec![I::P1, I::m3, I::d5],
            ChordShape::dim7 => vec![I::P1, I::m3, I::d5, I::d7],
            ChordShape::min7_b5 => vec![I::P1, I::m3, I::d5, I::m7],
            ChordShape::aug => vec![I::P1, I::M3, I::A5],
            ChordShape::aug7 => vec![I::P1, I::M3, I::A5, I::m7],
            ChordShape::sus2 => vec![I::P1, I::M2, I::P5],
            ChordShape::sus4 => vec![I::P1, I::P4, I::P5],
            ChordShape::sus4_7 => vec![I::P1, I::P4, I::P5, I::m7],
            ChordShape::add9 => vec![I::P1, I::M3, I::P5, I::M9],
            ChordShape::add11 => vec![I::P1, I::M3, I::P5, I::P11],
        }
    }
}

impl ChordShape {
    /// All chord shapes.
    pub fn all() -> Vec<ChordShape> {
        Self::major()
            .into_iter()
            .chain(Self::minor())
            .chain(Self::diminished())
            .chain(Self::augmented())
            .chain(Self::suspended())
            .collect()
    }

    /// Basic triad shapes.
    pub fn triad() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![maj, min, dim, aug]
    }

    /// Simple chord shapes which fit into a single octave.
    pub fn simple() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![
            maj, maj6, maj7, min, min6, min7, min_M7, dim, dim7, min7_b5, sus2, sus4, sus4_7,
        ]
    }

    /// Chord shapes spanning two octaves.
    pub fn jazz() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![maj6_9, maj9, maj11, maj13, min9, min11, min13, add9, add11]
    }

    /// Major chord shapes.
    pub fn major() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![maj, maj6, maj7, maj6_9, maj9, maj11, maj13]
    }

    /// Minor chord shapes.
    pub fn minor() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![min, min6, min7, min_M7, min9, min11, min13]
    }

    /// Diminished chord shapes.
    pub fn diminished() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![dim, dim7, min7_b5]
    }

    /// Augmented chord shapes.
    pub fn augmented() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![aug, aug7]
    }

    /// Suspended chord shapes.
    pub fn suspended() -> Vec<ChordShape> {
        use ChordShape::*;
        vec![sus2, sus4, sus4_7, add9, add11]
    }
}
