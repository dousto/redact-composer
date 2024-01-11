#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(not(doctest), doc = include_str!("../../README.md"))]

/// Utility traits and types.
pub mod util;

// Re-export core components
pub use redact_composer_core::{
    elements, error, render::Renderer, timing, timing::Timing, Composer, ComposerOptions,
    Composition, CompositionOptions, Element, Segment, SegmentRef,
};

/// Types and traits used for and during composition rendering.
pub mod render {
    pub use redact_composer_core::render::{
        context, AdhocRenderer, RenderEngine, RenderSegment, Renderer, RendererGroup, Result,
    };
}

#[cfg(feature = "derive")]
#[doc(inline)]
/// `feature = derive (default)`
pub use redact_composer_derive::Element;

#[cfg(feature = "midi")]
#[doc(inline)]
/// `feature = midi (default)`
pub use redact_composer_midi as midi;

#[cfg(feature = "musical")]
#[doc(inline)]
/// `feature = musical (default)`
pub use redact_composer_musical as musical;

/// Default renderers for [`midi`] elements if `midi` feature is enabled (default).
/// Otherwise, just an empty [`RenderEngine`](crate::render::RenderEngine).
pub fn renderers() -> crate::render::RenderEngine {
    let mut engine = crate::render::RenderEngine::new();

    #[cfg(feature = "midi")]
    {
        engine = engine + midi::renderers();
    }

    engine
}
