use super::MidiConverter;
use midly::TrackEventKind::Meta;
use midly::{MetaMessage, TrackEvent};
use redact_composer_core::derive::Element;
use redact_composer_core::render::tree::Tree;
use redact_composer_core::timing::elements::Tempo;
use redact_composer_core::{render::RenderSegment, Segment};
use serde::{Deserialize, Serialize};

#[derive(Element, Serialize, Deserialize, Debug)]
struct Composition;

#[test]
fn tempo_splice_beginning() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..10),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                10,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(500000.into()))
                }
            )
        ]
    );
}

#[test]
fn tempo_splice_end() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 20..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(500000.into()))
                }
            ),
            (
                20,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            )
        ]
    );
}

#[test]
fn tempo_splice_middle() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(500000.into()))
                }
            ),
            (
                10,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                20,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(500000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_into_multiple() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..15),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 15..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                15,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_spanning() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..15),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 15..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(80), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                10,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(750000.into()))
                }
            ),
            (
                20,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_spanning2() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..10),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 20..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(80), 5..25),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                5,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(750000.into()))
                }
            ),
            (
                25,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_spanning3() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..10),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 20..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(80), 0..25),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(750000.into()))
                }
            ),
            (
                25,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_spanning4() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..30),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..10),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 20..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(80), 5..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                5,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(750000.into()))
                }
            ),
        ]
    );
}

#[test]
fn tempo_splice_multi_spanning() {
    let mut render_tree: Tree<RenderSegment> = Tree::new();
    render_tree.insert(
        RenderSegment {
            rendered: false,
            seed: 0,
            segment: Segment::new(Composition, 0..40),
            error: None,
        },
        None,
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 0..10),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 10..20),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 20..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(100), 30..40),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    render_tree.insert(
        RenderSegment {
            segment: Segment::new(Tempo::from_bpm(80), 10..30),
            seed: 0,
            rendered: true,
            error: None,
        },
        Some(0),
    );

    let tempo_events = MidiConverter::extract_tempo_events(&render_tree);

    assert_eq!(
        tempo_events,
        vec![
            (
                0,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
            (
                10,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(750000.into()))
                }
            ),
            (
                30,
                TrackEvent {
                    delta: 0.into(),
                    kind: Meta(MetaMessage::Tempo(600000.into()))
                }
            ),
        ]
    );
}
