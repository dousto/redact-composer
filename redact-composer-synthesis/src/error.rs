use rustysynth::{MidiFileError, SoundFontError, SynthesizerError};
use std::io;
use thiserror::Error;

/// Error types which may occur during synthesis.
#[derive(Debug, Error)]
#[allow(missing_docs, clippy::enum_variant_names)]
pub enum SynthesisError {
    #[error("Error loading the SoundFont file: {:?}", .0)]
    SoundFontFileLoadError(#[from] io::Error),
    #[error("SoundFont error: {:?}", .0)]
    SoundFontError(#[from] SoundFontError),
    #[error("WAV error: {:?}", .0)]
    WavError(#[from] hound::Error),
    #[error("Midi error: {:?}", .0)]
    MidiError(#[from] MidiFileError),
    #[error("Synthesizer error: {:?}", .0)]
    SynthesizerError(#[from] SynthesizerError),
}
