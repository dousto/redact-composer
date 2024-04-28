# ![icon] RedACT Composer

[![docs-badge]][docs-link]
[![crates.io-badge]][crates.io-link]
[![ci-badge]][ci-link]
![license-badge]

**A Rust library for building modular musical composers.**

Composers are built by creating a set of composition elements, and defining how each of these elements will generate
further sub-elements. In this library's domain, these correspond to the
[`Element`](crate::Element) and [`Renderer`](crate::Renderer) traits respectively.

> This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Most importantly at this time
> would be [spec item #4](https://semver.org/spec/v2.0.0.html#spec-item-4).

<hr />
<div align="center">

Jump to: [
[Setup](#setup)
| [Example](#example)
| [Bigger Example](#much-bigger-example)
| [Inspector](#inspector)
| [Feature Flags](#feature-flags)
]
</div>
<hr />

<details open>
<summary>

## Setup
</summary>

```shell
cargo add redact-composer
```

If using the `serde` [feature](#feature-flags), `typetag` is also required:
```shell
cargo add typetag
```
</details>

<details open>
<summary> 

## Example
</summary>

The basic capabilities can be demonstrated by creating a simple I-IV-V-I chord composer. The full code example is
located at
[`redact-composer/examples/simple.rs`](https://github.com/dousto/redact-composer/blob/main/redact-composer/examples/simple.rs).

### Building Blocks
This example composer will use some library-provided elements ([`Chord`](crate::musical::elements::Chord),
[`Part`](crate::elements::Part), [`PlayNote`](crate::elements::PlayNote)) and two new elements:
```rust
#[derive(Element, Serialize, Deserialize, Debug)]
pub struct CompositionRoot;

#[derive(Element, Serialize, Deserialize, Debug)]
struct PlayChords;
```

Before moving ahead, some background: A composition is an n-ary tree structure and is "composed" by starting with a
root [`Element`](crate::Element), and calling its associated [`Renderer`](crate::Renderer) which
generates additional [`Element`](crate::Element)s as children. These children then have their
[`Renderer`](crate::Renderer)s called, and this process continues until tree leaves are reached (i.e. elements that do
not generate further children).

This composer will use the `CompositionRoot` element as a root. Defining a [`Renderer`](crate::Renderer) for this
then looks like:

```rust
struct CompositionRenderer;
impl Renderer for CompositionRenderer {
    type Element = CompositionRoot;

    fn render(
        &self, composition: SegmentRef<CompositionRoot>, context: CompositionContext,
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
```

> Note: [`Part::instrument(...)`](crate::elements::Part::instrument) is just a wrapper for another element, indicating
> that notes generated within the wrapped element are to be played by a single instrument at a time.

This [`Renderer`](crate::Renderer) takes a `CompositionRoot` element (via a [`SegmentRef`](crate::SegmentRef)) and generates several
children including [`Chord`](crate::musical::elements::Chord) elements (with a [`Rhythm`](crate::musical::rhythm::Rhythm) of one every two beats over the composition), and
newly defined `PlayChords` element. These children are returned as [`Segment`](crate::Segment)s, which defines where they
are located in the composition's timeline.

At this stage, the [`Chord`](crate::musical::elements::Chord) and `PlayChords` elements are just abstract concepts
however, and need to produce something concrete. This is done with another [`Renderer`](crate::Renderer) for
`PlayChords`:

```rust
struct PlayChordsRenderer;
impl Renderer for PlayChordsRenderer {
    type Element = PlayChords;

    fn render(
        &self, play_chords: SegmentRef<PlayChords>, context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        // `CompositionContext` enables finding previously rendered elements
        let chord_segments = context.find::<Chord>()
            .with_timing(Within, play_chords)
            .require_all()?;
        // As well as random number generation
        let mut rng = context.rng();

        // Map Chord notes to PlayNote elements, forming a triad
        let notes = chord_segments
            .iter()
            .flat_map(|chord| {
                chord.element
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
```

Here, [`CompositionContext`](crate::render::context::CompositionContext) is used to reference the previously created
[`Chord`](crate::musical::elements::Chord) segments. Then the [`Note`](crate::musical::elements::Note)s from each
[`Chord`](crate::musical::elements::Chord) within an octave range are [`play`](crate::musical::Note::play)ed over the
[`Chord`](crate::musical::elements::Chord) segment's timing.

### Creating the Composer
In essence, a [`Composer`](crate::Composer) is just a set of [`Renderer`](crate::Renderer)s, and can be constructed with
just a little bit of glue:

```rust
let composer = Composer::from(
    RenderEngine::new() + CompositionRenderer + PlayChordsRenderer,
);
```

And finally the magic unfolds by passing a root [`Segment`](crate::Segment) to its
[`compose()`](crate::Composer::compose) method.

```rust
// Create a 16-beat length composition
let composition_length = composer.options.ticks_per_beat * 16;
let composition = composer.compose(CompositionRoot.over(0..composition_length));

// Convert it to a MIDI file and save it
MidiConverter::convert(&composition).save("./composition.mid").unwrap();

// And/or synthesize it to audio with a SoundFont
let synth = SF2Synthesizer::new("./sounds/sound_font.sf2").unwrap();
synth.synthesize(&composition).to_file("./composition.wav").unwrap();
```
> Note: [`SF2Synthesizer`](crate::synthesis::SF2Synthesizer) does not have any default/embedded SoundFont so you'll have
> to supply your own. (FluidR3, created by Frank Wen, is a great general-purpose, high-quality, MIT licensed option)


The output should sound somewhat like this:

<https://github.com/dousto/redact-composer/assets/5882189/aeed4e7a-5543-4cf1-839d-d5f62c55fea9>

Additionally, composition outputs support serialization/deserialization (with `serde` feature, enabled by default).

```rust
// Write the composition output in json format
fs::write("./composition.json", serde_json::to_string_pretty(&composition).unwrap()).unwrap();
```
</details>
<details open>
<summary>

## Much bigger example
</summary>

Check out [this repo](https://github.com/dousto/redact-renderer-example) for a more in depth example which utilizes
additional features to create a full length composition.
</details>
<details open>
<summary>

## Inspector
</summary>

Debugging composition outputs can quickly get unwieldy with larger compositions.
[redact-composer-inspector](https://dousto.github.io/redact-composer-inspector/) is a simple web tool that helps to
visualize and navigate the structure of [`Composition`](crate::Composition) outputs (currently only compatible with
json output).

For example, here is the [simple example loaded in the inspector](https://dousto.github.io/redact-composer-inspector/inspect?composition=examples/simple).
</details>
<details open>
<summary>

## Feature Flags
</summary>

### `default`
`derive`, `musical`, `midi`, `synthesis`, `serde`

### `derive` <sub>default</sub>
Enable derive macro for [`Element`](crate::Element).

### `musical` <sub>default</sub>
Include [`musical`](crate::musical) domain module. ([`Key`](crate::musical::Key), [`Chord`](crate::musical::Chord),
[`Rhythm`](crate::musical::rhythm::Rhythm), etc..).

### `midi` <sub>default</sub>
Include [`midi`](crate::midi) module containing MIDI-related [`Element`](crate::Element)s and MIDI converter for
[`Composition`](crate::Composition)s.

### `synthesis` <sub>default</sub>
Include [`synthesis`](crate::synthesis) module to synthesize [`Composition`](crate::Composition)s into audio.

### `serde` <sub>default</sub>
Enables serialization and deserialization of [`Composition`](crate::Composition) outputs via (as you may have guessed)
[`serde`](https://docs.rs/serde/latest/serde/).
</details>

[icon]: https://dousto.github.io/redact-composer-inspector-dev/favicon-32.png ""
[docs-badge]: https://docs.rs/redact-composer/badge.svg
[docs-link]: https://docs.rs/redact-composer
[crates.io-badge]: https://img.shields.io/crates/v/redact-composer.svg
[crates.io-link]: https://crates.io/crates/redact-composer
[ci-badge]: https://github.com/dousto/redact-composer/workflows/CI/badge.svg
[ci-link]: https://github.com/dousto/redact-composer/actions
[license-badge]: https://img.shields.io/crates/l/redact-composer.svg