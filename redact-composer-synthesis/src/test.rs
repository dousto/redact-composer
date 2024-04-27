use crate::error::SynthesisError;
use crate::{SF2Synthesizer, SoundFontSynthesizerOptions};
use redact_composer_core::derive::Element;
use redact_composer_core::elements::Part;
use redact_composer_core::render::{AdhocRenderer, RenderEngine};
use redact_composer_core::timing::Tempo;
use redact_composer_core::{Composer, IntoSegment};
use redact_composer_musical::Note;
use redact_composer_musical::NoteName::{A, B, C, E, F};
use serde::{Deserialize, Serialize};
use std::fs;
use std::iter::once;

#[derive(Debug, Element, Serialize, Deserialize)]
struct SynthComp;

pub fn test_synth_composer() -> Composer {
    let notes: [Note; 5] = [
        (F, 3).into(),
        (A, 3).into(),
        (C, 4).into(),
        (E, 4).into(),
        (B, 4).into(),
    ];
    let renderers = RenderEngine::new()
        + AdhocRenderer::<SynthComp>::new(move |segment, _| {
            Ok(notes
                .into_iter()
                .map(|n| n.play(100).over(segment))
                .chain(once(Tempo::from_bpm(60).over(segment)))
                .collect())
        });

    Composer::from(renderers)
}

const SF2_TEST_FILE: &str = "./test-resources/tiny_sine.sf2";
const TEST_OUTPUT_DIR: &str = "./test-result-output";

#[test]
pub fn test_soundfont_synthesis() {
    let composer = test_synth_composer();
    let composition =
        composer.compose(Part::instrument(SynthComp).over(0..composer.options.ticks_per_beat));

    let synth = SF2Synthesizer::new(SF2_TEST_FILE).expect("Error creating SF2Synthesizer");
    let synth_result = synth.synthesize(&composition).to_raw_stereo_waveforms();

    match synth_result {
        Ok((left, right)) => {
            assert_eq!(
                left.len(),
                right.len(),
                "Left and right channel samples should be equal"
            );

            // Default sample rate is 44.1kHz, Tempo is 60 BPM (1 beat per second)
            // This 1 beat, 1 second composition should generate at least 44100 samples
            // Usually more accounting for trailoff, but should never be more than 10 extra seconds worth
            assert!(
                left.len() > 44100,
                "Expected > 41000 samples but only got {:?}",
                left.len()
            );
            assert!(
                left.len() < 44100 * 11,
                "Expected < 485100 samples but got {:?}",
                left.len()
            );

            // Smoke test ensuring that it wasn't just silence produced
            let non_zero_samples = [&left, &right]
                .iter()
                .flat_map(|ch| ch.iter())
                .filter(|s| **s != 0.0)
                .count();
            let non_zero_ratio = non_zero_samples as f32 / (left.len() * 2) as f32;
            assert!(
                non_zero_ratio > 0.97,
                "Expected > 97% of samples to be non-zero but only {:?} were",
                non_zero_ratio
            );
        }
        Err(err) => {
            panic!("Synthesis failed!: {:?}", err);
        }
    }
}

#[test]
pub fn test_soundfont_synthesis_to_file() {
    let composer = test_synth_composer();
    let composition =
        composer.compose(Part::instrument(SynthComp).over(0..composer.options.ticks_per_beat));

    let synth = SF2Synthesizer::new(SF2_TEST_FILE).expect("Error creating SF2Synthesizer");
    let output_file = format!("{}/sine_chord.wav", TEST_OUTPUT_DIR);
    synth
        .synthesize(&composition)
        .to_file(&output_file)
        .expect("Error during synthesis");
    let file_bytes = fs::read(&output_file).expect("Error reading the synthesized file");
    assert!(
        file_bytes.len() > 500000,
        "WAV file size is less than expected {:?}",
        file_bytes.len()
    );
}

#[test]
pub fn test_soundfont_synthesis_to_file_with_custom_options() {
    let composer = test_synth_composer();
    let composition =
        composer.compose(Part::instrument(SynthComp).over(0..composer.options.ticks_per_beat));

    let synth = SF2Synthesizer::new_with_options(
        SF2_TEST_FILE,
        SoundFontSynthesizerOptions {
            sample_rate: 96000,
            bit_depth: 32,
        },
    )
    .expect("Error creating SF2Synthesizer");
    let output_file = format!("{}/high_quality_sine_chord.wav", TEST_OUTPUT_DIR);
    synth
        .synthesize(&composition)
        .to_file(&output_file)
        .expect("Error during synthesis");
    let file_bytes = fs::read(&output_file).expect("Error reading the synthesized file");
    assert!(
        file_bytes.len() > 2200000,
        "WAV file size is less than expected {:?}",
        file_bytes.len()
    );
}

#[test]
pub fn test_non_existent_soundfont() {
    let synth = SF2Synthesizer::new("./test-resources/no_its_not_here.sf2");
    assert!(
        synth.is_err(),
        "Synth found the SoundFont that's not supposed to exist!"
    );
    assert!(matches!(
        synth.unwrap_err(),
        SynthesisError::SoundFontFileLoadError(_)
    ))
}

#[test]
pub fn debug_display() {
    let synth = SF2Synthesizer::new(SF2_TEST_FILE).expect("Error creating SF2Synthesizer");
    assert_eq!(format!("{:?}", synth), "SF2Synthesizer { sound_font: \"Tiny Sine\", options: SoundFontSynthesizerOptions { sample_rate: 44100, bit_depth: 16 } }");
}
