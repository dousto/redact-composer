use rand::Rng;
use redact_composer::{
    elements::Part,
    midi::convert::MidiConverter,
    musical::{
        elements::Chord,
        rhythm::Rhythm,
        ChordShape::maj,
        Note, NoteIterator,
        NoteName::{C, F, G},
    },
    render::context::{CompositionContext, TimingRelation::Within},
    render::{RenderEngine, Result},
    synthesis::SF2Synthesizer,
    util::IntoSegment,
    Composer, Renderer, Segment, SegmentRef,
};
use serde::{Deserialize, Serialize};
use std::fs;

fn main() {
    let composer = Composer::from(RenderEngine::new() + CompositionRenderer + PlayChordsRenderer);

    // Create a 16-beat length composition
    let composition_length = composer.options.ticks_per_beat * 16;
    let composition = composer.compose(CompositionRoot.over(0..composition_length));

    // Convert it to a MIDI file and save it
    MidiConverter::convert(&composition)
        .save("./composition.mid")
        .unwrap();

    // And/or synthesize it to audio with a SoundFont
    let synth = SF2Synthesizer::new("./sounds/sound_font.sf2").unwrap();
    synth
        .synthesize(&composition)
        .to_file("./composition.wav")
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
        composition: SegmentRef<CompositionRoot>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        let chords: [Chord; 4] = [
            (C, maj).into(),
            (F, maj).into(),
            (G, maj).into(),
            (C, maj).into(),
        ];

        Ok(
            // Repeat the four chords over the composition -- one every two beats
            Rhythm::from([2 * context.beat_length()])
                .iter_over(composition)
                .zip(chords.into_iter().cycle())
                .map(|(subdivision, chord)| chord.over(subdivision))
                .chain([
                    // Also include the new component, spanning the whole composition
                    Part::instrument(PlayChords).over(composition),
                ])
                .collect(),
        )
    }
}

struct PlayChordsRenderer;
impl Renderer for PlayChordsRenderer {
    type Element = PlayChords;

    fn render(
        &self,
        play_chords: SegmentRef<PlayChords>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        // `CompositionContext` enables finding previously rendered elements
        let chord_segments = context
            .find::<Chord>()
            .with_timing(Within, play_chords)
            .require_all()?;
        // As well as random number generation
        let mut rng = context.rng();

        // Map Chord notes to PlayNote elements, forming a triad
        let notes = chord_segments
            .iter()
            .flat_map(|chord| {
                chord
                    .element
                    .iter_notes_in_range(Note::from((C, 4))..Note::from((C, 5)))
                    .map(|note|
                        // Add subtle nuance striking the notes with different velocities
                        note.play(rng.gen_range(80..110) /* velocity */)
                            .over(chord))
                    .collect::<Vec<_>>()
            })
            .collect();

        Ok(notes)
    }
}
