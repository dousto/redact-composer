use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum RendererError {
    /// May be returned from [`Renderer::render`] indicating the required dependencies
    /// are not yet available.
    #[error("Missing required context: {:?}" , .0)]
    MissingContext(String),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("The contained type does not match its target.")]
    TypeMismatch,
}
