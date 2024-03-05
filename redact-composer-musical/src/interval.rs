use std::iter::Sum;
use std::ops::Add;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// A pitch difference, measured in half-steps/semitones.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
pub struct Interval(pub u8);

#[allow(non_upper_case_globals)]
impl Interval {
    /// Perfect Unison (0 semitones)
    pub const P1: Interval = Interval(0);
    /// Minor 2nd (1 semitone)
    pub const m2: Interval = Interval(1);
    /// Major 2nd (2 semitones)
    pub const M2: Interval = Interval(2);
    /// Minor 3rd (3 semitones)
    pub const m3: Interval = Interval(3);
    /// Major 3rd (4 semitones)
    pub const M3: Interval = Interval(4);
    /// Perfect 4th (5 semitones)
    pub const P4: Interval = Interval(5);
    /// Tritone (6 semitones)
    pub const TT: Interval = Interval(6);
    /// Augmented 4th (6 semitones)
    pub const A4: Interval = Interval(6);
    /// Diminished 5th (6 semitones)
    pub const d5: Interval = Interval(6);
    /// Perfect 5th (7 semitones)
    pub const P5: Interval = Interval(7);
    /// Minor 6th (8 semitones)
    pub const m6: Interval = Interval(8);
    /// Augmented 5th (8 semitones)
    pub const A5: Interval = Interval(8);
    /// Major 6th (9 semitones)
    pub const M6: Interval = Interval(9);
    /// Diminished 7th (9 semitones)
    pub const d7: Interval = Interval(9);
    /// Minor 7th (10 semitones)
    pub const m7: Interval = Interval(10);
    /// Major 7th (11 semitones)
    pub const M7: Interval = Interval(11);
    /// Perfect 8th (12 semitones)
    pub const P8: Interval = Interval(12);
    /// Octave (12 semitones)
    pub const Octave: Interval = Self::P8;
    /// Minor 9th (13 semitones)
    pub const m9: Interval = Interval(13);
    /// Major 9th (13 semitones)
    pub const M9: Interval = Interval(14);
    /// Minor 10th (15 semitones)
    pub const m10: Interval = Interval(15);
    /// Major 10th (16 semitones)
    pub const M10: Interval = Interval(16);
    /// Perfect 11th (17 semitones)
    pub const P11: Interval = Interval(17);
    /// Perfect 12th (19 semitones)
    pub const P12: Interval = Interval(19);
    /// Minor 13th (20 semitones)
    pub const m13: Interval = Interval(20);
    /// Major 13th (21 semitones)
    pub const M13: Interval = Interval(21);

    /// Returns `true` if this is a simple interval (up to one octave).
    /// ```
    /// # use redact_composer_musical::Interval;
    /// assert!(Interval::P5.is_simple());
    /// ```
    pub fn is_simple(&self) -> bool {
        self.0 <= 12
    }

    /// Return the simple interval counterpart.
    /// Note: This function will reduce [`Interval::Octave`] to [`Interval::P1`].
    /// ```
    /// # use redact_composer_musical::Interval;
    /// assert_eq!(Interval::m9.to_simple(), Interval::m2);
    /// ```
    pub fn to_simple(self) -> Interval {
        Interval(self.0 % 12)
    }

    /// Returns `true` if this is a compound interval (larger than one octave).
    /// ```
    /// # use redact_composer_musical::Interval;
    /// assert!(Interval::m9.is_compound());
    /// ```
    pub fn is_compound(&self) -> bool {
        !self.is_simple()
    }

    /// Return the compound interval counterpart (added octave). Does nothing if the interval is already compound.
    /// ```
    /// # use redact_composer_musical::Interval;
    /// assert_eq!(Interval::m2.to_compound(), Interval::m9);
    /// ```
    pub fn to_compound(self) -> Interval {
        if self.is_simple() {
            Interval(self.0 + 12)
        } else {
            self
        }
    }

    /// Returns the interval's inversion.
    /// ```
    /// # use redact_composer_musical::Interval;
    /// assert_eq!(Interval::P5.inversion(), Interval::P4);
    /// ```
    pub fn inversion(&self) -> Interval {
        if self.is_simple() {
            Interval(12 - self.0)
        } else {
            let octaves = self.0 / 12 + 1;

            Interval(12 * octaves - self.0)
        }
    }
}

impl Add for Interval {
    type Output = Interval;

    fn add(self, rhs: Self) -> Self::Output {
        Interval(self.0 + rhs.0)
    }
}

impl Sum for Interval {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Interval::default(), |i1, i2| i1 + i2)
    }
}
