use thiserror::Error;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Error type which may be returned from [`Renderer::render`](crate::render::Renderer::render).
pub enum RendererError {
    /// May be returned while rendering a composition element indicating the required dependencies
    /// are not yet available.
    #[error("Missing required context: {:?}" , .0)]
    MissingContext(String),
    /// Error indicating a type conversion failure.
    #[error("Invalid conversion attempt during render.")]
    BadConversion(#[from] ConversionError),
}

#[derive(Debug, Error)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Error indicating a type conversion failure.
pub enum ConversionError {
    /// Error type when attempting a conversion where the type does not match.
    #[error("The contained type does not match its target.")]
    TypeMismatch,
}
