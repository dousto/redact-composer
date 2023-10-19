use thiserror::Error;

#[derive(Debug, Error)]
pub enum RendererError {
    /// May be returned from [`Renderer::render`] indicating the required dependencies
    /// are not yet available.
    #[error("Missing required context.")]
    MissingContext,
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("The contained type does not match its target.")]
    TypeMismatch,
}
