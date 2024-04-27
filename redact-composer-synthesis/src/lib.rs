#![deny(missing_docs, missing_debug_implementations)]
//! Audio synthesis utilities for [`redact-composer`].
//!
//! ## Example
//! Synthesize a [`Composition`] to WAV format using [`SF2Synthesizer`].
//! ```no_run
//! # use redact_composer_core::Composition;
//! # use redact_composer_synthesis::{SF2Synthesizer, SF2Synthesizable};
//! let composition: Composition = todo!();
//! let synth = SF2Synthesizer::new("./path/to/sound_font.sf2")
//!     .expect("The SoundFont file should exist and be SF2 format");
//! synth.synthesize(&composition)
//!     .to_file("./path/to/output.wav")
//!     .unwrap();
//!
//! // Alternatively
//! composition.synthesize_with(&synth)
//!     .to_file("./path/to/output.wav")
//! .unwrap();
//! ```
//!
//! ## Options
//! [`SF2Synthesizer`] defaults to 44.1kHz sample rate with a bit-depth of 16, but can be customized
//! if desired.
//! ```
//! # use redact_composer_synthesis::{SF2Synthesizer, SoundFontSynthesizerOptions};
//! let synth = SF2Synthesizer::new_with_options(
//!     "./path/to/sound_font.sf2",
//!     SoundFontSynthesizerOptions {
//!         sample_rate: 96000,
//!         bit_depth: 32, // This should be one of [8, 16, 24, 32].
//!     }
//! ).expect("Custom settings should be applied!");
//! ```

mod error;
#[cfg(test)]
mod test;

use crate::error::SynthesisError;
use hound::{SampleFormat, WavSpec, WavWriter};
use log::{debug, info};
use midly::Smf;
use redact_composer_core::Composition;
use redact_composer_midi::convert::MidiConverter;
pub use rustysynth::SoundFont;
use rustysynth::{MidiFile, MidiFileSequencer, Synthesizer, SynthesizerSettings};
use std::cmp::Ordering::Less;
use std::fmt::{Debug, Formatter};
use std::fs;
use std::fs::File;
use std::io::{Seek, Write};
use std::ops::RangeFrom;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

/// Result type which may produce [`SynthesisError`].
pub type Result<T, E = SynthesisError> = std::result::Result<T, E>;

/// A SoundFont [`Composition`] synthesizer (`.sf2` specifically). Outputs as WAV format.
///
/// Made possible by [`rustysynth`] and [`hound`] -- special thanks to their authors/contributors.
pub struct SF2Synthesizer {
    pub(crate) sound_font: Arc<SoundFont>,
    pub(crate) options: SoundFontSynthesizerOptions,
}

impl Debug for SF2Synthesizer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SF2Synthesizer")
            .field("sound_font", &self.sound_font.get_info().get_bank_name())
            .field("options", &self.options)
            .finish()
    }
}

impl SF2Synthesizer {
    /// Creates a new SoundFont Synthesizer from a SoundFont (.sf2) file with default options
    /// (sample_rate = 44.1kHz, bit-depth = 16).
    pub fn new<P: AsRef<Path>>(sf2_file: P) -> Result<SF2Synthesizer> {
        Self::new_with_options(sf2_file, SoundFontSynthesizerOptions::default())
    }

    /// Create a new SoundFont Synthesizer with custom options.
    pub fn new_with_options<P: AsRef<Path>>(
        sf2_file: P,
        options: SoundFontSynthesizerOptions,
    ) -> Result<SF2Synthesizer> {
        let mut sound_font_file = File::open(sf2_file)?;
        let sound_font = SoundFont::new(&mut sound_font_file)?;

        Ok(SF2Synthesizer {
            sound_font: Arc::new(sound_font),
            options,
        })
    }

    /// Prepares a synthesis request for the given content. Use further chained calls to initiate
    /// synthesis -- such as [`to_file`](SF2SynthesisRequest::to_file), [`write`](SF2SynthesisRequest::write)
    /// or [`to_raw_stereo_waveforms`](SF2SynthesisRequest::to_raw_stereo_waveforms).
    pub fn synthesize<'a, S: MidiBytesProvider>(
        &'a self,
        content: &'a S,
    ) -> SF2SynthesisRequest<'_, S> {
        content.synthesize_with(self)
    }
}

impl MidiBytesProvider for Composition {
    fn midi_bytes(&self) -> Vec<u8> {
        let smf = MidiConverter::convert(self);
        smf.midi_bytes()
    }
}

impl MidiBytesProvider for Smf<'_> {
    fn midi_bytes(&self) -> Vec<u8> {
        let mut smf_bytes = Vec::new();
        self.write(&mut smf_bytes).unwrap();

        smf_bytes
    }
}

impl<M: MidiBytesProvider> SF2Synthesizable<M> for M {
    fn synthesize_with<'a>(&'a self, synth: &'a SF2Synthesizer) -> SF2SynthesisRequest<'_, M> {
        SF2SynthesisRequest {
            synth,
            midi_reader: self,
        }
    }
}

/// A trait implemented by types which can be synthesized.
pub trait SF2Synthesizable<M: MidiBytesProvider> {
    /// Prepare to synthesize with a [`SF2Synthesizer`].
    fn synthesize_with<'a>(&'a self, synth: &'a SF2Synthesizer) -> SF2SynthesisRequest<'_, M>;
}

/// Trait implemented for types which can provide midi file bytes. ([`Composition`], [`Smf`]..)
pub trait MidiBytesProvider {
    /// Return the midi file bytes for this type.
    fn midi_bytes(&self) -> Vec<u8>;
}

/// A synthesis request, which can be processed to multiple output types.
#[allow(missing_debug_implementations)]
pub struct SF2SynthesisRequest<'a, M: MidiBytesProvider> {
    synth: &'a SF2Synthesizer,
    midi_reader: &'a M,
}

impl<M: MidiBytesProvider> SF2SynthesisRequest<'_, M> {
    /// Synthesizes and writes the WAV output to the given `writer`.
    pub fn write<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let (mut left, mut right) = self.to_raw_stereo_waveforms()?;

        info!("Writing WAV output.");
        let wav_spec = WavSpec {
            channels: 2,
            sample_rate: self.synth.options.sample_rate,
            bits_per_sample: self.synth.options.bit_depth as u16,
            sample_format: SampleFormat::Int,
        };

        normalize(&mut left, &mut right);

        let bit_depth_max_val = 2_i64.pow((wav_spec.bits_per_sample - 1).into()) - 1;
        let mut writer = WavWriter::new(writer, wav_spec)?;
        for (ls, rs) in left.into_iter().zip(right.into_iter()) {
            writer.write_sample((ls * bit_depth_max_val as f32) as i32)?;
            writer.write_sample((rs * bit_depth_max_val as f32) as i32)?;
        }

        Ok(writer.finalize()?)
    }
    /// Synthesizes and writes the WAV output to the given file -- overwriting if already present.
    pub fn to_file<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        let path = filename.as_ref();
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?
        }
        let file = File::create(path)?;
        let buf_writer = std::io::BufWriter::new(file);
        self.write(buf_writer)?;

        info!("Output written to '{}'", path.display());

        Ok(())
    }

    /// Synthesizes and returns the raw stereo waveforms as `(Vec<f32>, Vec<f32>)` (left and right channels).
    pub fn to_raw_stereo_waveforms(&self) -> Result<(Vec<f32>, Vec<f32>)> {
        info!("Synthesizing...");
        debug!("{:?}", self.synth.options);
        let start_instant = std::time::Instant::now();
        let midi_bytes = self.midi_reader.midi_bytes();
        let midi_file = Arc::new(MidiFile::new(&mut &midi_bytes[..])?);

        // Create a RustySynth MIDI file sequencer.
        let settings = SynthesizerSettings::new(self.synth.options.sample_rate as i32);
        let synthesizer = Synthesizer::new(&self.synth.sound_font, &settings)?;
        let mut sequencer = MidiFileSequencer::new(synthesizer);

        // Play our midi file through the sequencer
        sequencer.play(&midi_file, false);

        // Create two sample buffers for left and right stereo channels
        // Adds an additional 10 seconds at the end to account for trailoff
        let sample_count = (settings.sample_rate as f64 * (midi_file.get_length() + 10.0)) as usize;
        let mut left: Vec<f32> = vec![0_f32; sample_count];
        let mut right: Vec<f32> = vec![0_f32; sample_count];

        // Render the waveforms into the sample buffers.
        sequencer.render(&mut left[..], &mut right[..]);

        // Trim the final period of silence at the end of the buffers
        let end_trim_range = get_end_trim_range(&left, &right);
        [&mut left, &mut right].into_iter().for_each(|ch| {
            ch.drain(end_trim_range.clone());
        });

        let audio_duration =
            Duration::from_secs_f32(left.len() as f32 / settings.sample_rate as f32);
        let duration = std::time::Instant::now().duration_since(start_instant);
        info!(
            "Synthesis complete ({:?}). Synthesized {:?} of audio.",
            duration, audio_duration
        );

        Ok((left, right))
    }
}

// Scales the left/right sample buffers so their samples fit snuggly in the range [-1.0, 1.0].
fn normalize(left: &mut [f32], right: &mut [f32]) {
    let abs_max = left
        .iter()
        .chain(right.iter())
        .map(|s| s.abs())
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Less));

    if let Some(max) = abs_max {
        for s in left.iter_mut().chain(right.iter_mut()) {
            *s /= max;
        }
    }
}

// Finds the tail range of silence in the stereo channel samples
fn get_end_trim_range(left: &[f32], right: &[f32]) -> RangeFrom<usize> {
    let end = left
        .iter()
        .zip(right.iter())
        .enumerate()
        .fold(
            0,
            |end, (idx, (ls, rs))| {
                if ls != &0.0 && rs != &0.0 {
                    idx
                } else {
                    end
                }
            },
        );

    end..
}

/// Options to configure a [`SF2Synthesizer`].
#[derive(Debug, Copy, Clone)]
pub struct SoundFontSynthesizerOptions {
    /// Sample rate in Hz. Default: 44100
    pub sample_rate: u32,
    /// Bit depth of the WAV output, must be one of [8, 16, 24, 32]. Default: 16.
    pub bit_depth: u8,
}

impl Default for SoundFontSynthesizerOptions {
    fn default() -> Self {
        SoundFontSynthesizerOptions {
            sample_rate: 44100,
            bit_depth: 16,
        }
    }
}
