#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

/// Offset applied to a [`Scale`](super::Scale)'s interval sequence.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Mode {
    /// No offset
    #[default]
    Ionian,
    /// Offset of 1, starting a scale from the second interval step.
    Dorian,
    /// Offset of 2, starting a scale from the third interval step.
    Phrygian,
    /// Offset of 3, starting a scale from the fourth interval step.
    Lydian,
    /// Offset of 4, starting a scale from the fifth interval step.
    Mixolydian,
    /// Offset of 5, starting a scale from the sixth interval step.
    Aeolian,
    /// Offset of 6, starting a scale from the seventh interval step.
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
