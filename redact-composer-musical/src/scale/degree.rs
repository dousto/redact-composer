use std::ops::{Add, Sub};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Scale degree, based on a 7-note scale.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[allow(missing_docs)]
pub enum Degree {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
}

impl Degree {
    const VALUES: [Degree; 7] = [
        Degree::I,
        Degree::II,
        Degree::III,
        Degree::IV,
        Degree::V,
        Degree::VI,
        Degree::VII,
    ];

    /// All [`Degree`] variants.
    /// ```
    /// # use redact_composer_musical::{Degree, Degree::*};
    /// assert_eq!(Degree::values(), [I, II, III, IV, V, VI, VII]);
    /// ```
    pub fn values() -> [Degree; 7] {
        Self::VALUES
    }

    /// Returns the next [`Degree`] following this one.
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(Degree::I.next(), Degree::II);
    /// assert_eq!(Degree::VII.next(), Degree::I)
    /// ```
    pub fn next(&self) -> Degree {
        match self {
            Degree::I => Degree::II,
            Degree::II => Degree::III,
            Degree::III => Degree::IV,
            Degree::IV => Degree::V,
            Degree::V => Degree::VI,
            Degree::VI => Degree::VII,
            Degree::VII => Degree::I,
        }
    }

    /// Returns the [`Degree`] previous to this one.
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(Degree::I.prev(), Degree::VII);
    /// assert_eq!(Degree::VII.prev(), Degree::VI);
    /// ```
    pub fn prev(&self) -> Degree {
        match self {
            Degree::I => Degree::VII,
            Degree::II => Degree::I,
            Degree::III => Degree::II,
            Degree::IV => Degree::III,
            Degree::V => Degree::IV,
            Degree::VI => Degree::V,
            Degree::VII => Degree::VI,
        }
    }

    /// Returns the minimum absolute difference from this degree to another.
    /// ```
    /// use redact_composer_musical::Degree;
    /// assert_eq!(Degree::I.diff(&Degree::I), 0);
    /// assert_eq!(Degree::I.diff(&Degree::II), 1);
    /// // Even though VI - II = 4, there is a shorter path traversing the cycle boundary
    /// assert_eq!(Degree::II.diff(&Degree::VI), 3);
    /// ```
    pub fn diff(&self, other: &Degree) -> u8 {
        let (lower, higher) = if *other as u8 >= *self as u8 {
            (*self as u8, *other as u8)
        } else {
            (*other as u8, *self as u8)
        };

        (higher - lower).min(lower + 7 - higher)
    }
}

impl From<Degree> for u8 {
    /// Returns the 0-based value of this degree.
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(0_u8, Degree::I.into());
    /// ```
    fn from(value: Degree) -> Self {
        value as u8
    }
}

impl From<u8> for Degree {
    /// Produces a [`Degree`] using a 0-based [`u8`] index.
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(Degree::I, Degree::from(0_u8));
    /// assert_eq!(Degree::I, Degree::from(7_u8));
    /// ```
    fn from(value: u8) -> Self {
        match value % 7 {
            0 => Degree::I,
            1 => Degree::II,
            2 => Degree::III,
            3 => Degree::IV,
            4 => Degree::V,
            5 => Degree::VI,
            6 => Degree::VII,
            _ => unreachable!(),
        }
    }
}

impl Add<u8> for Degree {
    type Output = Degree;

    /// Adds an offset to a scale degree, wrapping after [`Degree::VII`].
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(Degree::I + 1, Degree::II);
    /// assert_eq!(Degree::I + 7, Degree::I);
    /// ```
    fn add(self, rhs: u8) -> Self::Output {
        Degree::from(self as u8 + rhs)
    }
}

impl Sub<u8> for Degree {
    type Output = Degree;

    /// Subtracts an offset from a scale degree, wrapping at [`Degree::I`].
    /// ```
    /// # use redact_composer_musical::Degree;
    /// assert_eq!(Degree::II - 1, Degree::I);
    /// assert_eq!(Degree::I - 1, Degree::VII);
    /// ```
    fn sub(self, rhs: u8) -> Self::Output {
        Degree::from(7_u8 + self as u8 - rhs)
    }
}
