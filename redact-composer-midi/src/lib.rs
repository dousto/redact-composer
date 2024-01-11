#![deny(missing_docs, missing_debug_implementations)]
//! MIDI-related types, [`Element`]s and
//! [`Composition`](redact_composer_core::Composition) output converter.

/// Midi converter for [`Composition`](redact_composer_core::Composition) output.
pub mod convert;

/// General Midi Level 1 types and elements.
pub mod gm;

use redact_composer_core::derive::Element;
use redact_composer_core::render::{AdhocRenderer, RenderEngine, Renderer};
use redact_composer_core::IntoCompositionSegment;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Elements implementing [`Element`].
pub mod elements {
    pub use super::{DrumKit, Program};
}

/// The renderers for [`Element`]s of this
/// module.
pub fn renderers() -> RenderEngine {
    RenderEngine::new() + DrumKit::renderer() + gm::renderers()
}

/// A program number (instrument) that should play during a
/// [`Part`](redact_composer_core::elements::Part).
#[derive(Element, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Program(pub u8);

/// A semantic wrapper for program number indicating a drum instrument.
#[derive(Element, Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DrumKit(pub u8);

impl DrumKit {
    /// Default [`DrumKit`] renderer. Simply converts it into a [`Program`].
    pub fn renderer() -> impl Renderer<Element = Self> {
        AdhocRenderer::<Self>::new(|segment, _| {
            Ok(vec![
                Program::from(segment.element).into_segment(segment.timing)
            ])
        })
    }
}

impl From<DrumKit> for Program {
    fn from(value: DrumKit) -> Self {
        Program(value.0)
    }
}

impl From<&DrumKit> for Program {
    fn from(value: &DrumKit) -> Self {
        Program(value.0)
    }
}

impl From<u8> for DrumKit {
    fn from(value: u8) -> Self {
        DrumKit(value)
    }
}

impl From<&u8> for DrumKit {
    fn from(value: &u8) -> Self {
        DrumKit(*value)
    }
}
