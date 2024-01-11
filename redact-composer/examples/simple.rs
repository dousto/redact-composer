use rand::Rng;
use redact_composer::{
    elements::{Part, PlayNote},
    midi::convert::MidiConverter,
    musical::{
        elements::{Chord, Key, Scale},
        rhythm::Rhythm,
        Notes,
    },
    render::context::{
        CompositionContext,
        TimingRelation::{During, Within},
    },
    render::{RenderEngine, Result},
    util::IntoCompositionSegment,
    Composer, Renderer, Segment,
};
use redact_composer_core::SegmentRef;
use serde::{Deserialize, Serialize};
use std::fs;

fn main() {
    let composer = Composer::from(RenderEngine::new() + CompositionRenderer + PlayChordsRenderer);

    // Create a 16-beat length composition
    let composition_length = composer.options.ticks_per_beat * 16;
    let composition = composer.compose(CompositionRoot.into_segment(0..composition_length));

    // Convert it to a MIDI file and save it
    MidiConverter::convert(&composition)
        .save("./composition.mid")
        .unwrap();

    // Write the composition output in json format
    fs::write(
        "./composition.json",
        serde_json::to_string_pretty(&composition).unwrap(),
    )
    .unwrap();
}

#[derive(redact_composer::Element, Serialize, Deserialize, Debug)]
pub struct CompositionRoot;

#[derive(redact_composer::Element, Serialize, Deserialize, Debug)]
struct PlayChords;

struct CompositionRenderer;
impl Renderer for CompositionRenderer {
    type Element = CompositionRoot;

    fn render(
        &self,
        composition: SegmentRef<Self::Element>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        Ok(
            // Repeat four chords over the composition -- one every two beats
            Rhythm::from([2 * context.beat_length()])
                .iter_over(composition.timing)
                .zip(
                    [Chord::I, Chord::IV, Chord::V, Chord::I]
                        .into_iter()
                        .cycle(),
                )
                .map(|(subdivision, chord)| chord.into_segment(subdivision.timing()))
                .chain([
                    // Also include the new component, spanning the whole composition
                    Part::instrument(PlayChords).into_segment(composition.timing),
                    // And a Key for the composition -- used later
                    Key {
                        tonic: 0, /* C */
                        scale: Scale::Major,
                        mode: Default::default(),
                    }
                    .into_segment(composition.timing),
                ])
                .collect::<Vec<_>>(),
        )
    }
}

struct PlayChordsRenderer;
impl Renderer for PlayChordsRenderer {
    type Element = PlayChords;

    fn render(
        &self,
        play_chords: SegmentRef<Self::Element>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        // `CompositionContext` enables finding previously rendered elements
        let chord_segments = context
            .find::<Chord>()
            .with_timing(Within, play_chords.timing)
            .require_all()?;
        let key = context
            .find::<Key>()
            .with_timing(During, play_chords.timing)
            .require()?
            .element;
        // As well as random number generation
        let mut rng = context.rng();

        // Map Chord notes to PlayNote elements, forming a triad
        let notes = chord_segments
            .iter()
            .flat_map(|chord| {
                Notes::from(key.chord(chord.element))
                    .in_range(60..72)
                    .into_iter()
                    .map(|note|
                        // Add subtle nuance striking the notes with different velocities
                        PlayNote { note, velocity: rng.gen_range(80..110) }
                            .into_segment(chord.timing))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Ok(notes)
    }
}
