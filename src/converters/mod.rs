use crate::{
    composer::{
        render::{Node, Tree},
        Part, PlayNote, RenderSegment,
    },
    musical::midi::Instrument,
};
use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use std::cmp::Ordering;

pub struct MidiConverter;

impl MidiConverter {
    /// Converts the given render tree ([`Tree<RenderSegment>`]) into MIDI format using the [midly] crate (special thanks to its creators!).
    pub fn convert(tree: &Tree<RenderSegment>) -> Smf {
        let track_subtrees: Vec<&Node<RenderSegment>> = tree
            .iter()
            .filter(|n| n.value.segment.segment_type_as::<Part>().is_some())
            .collect();

        let tracks: Vec<Vec<TrackEvent>> = track_subtrees
            .iter()
            .enumerate()
            .map(|(idx, subtree_root)| {
                let u8idx: u8 = idx.try_into().unwrap();
                let channel = {
                    match subtree_root
                        .value
                        .segment
                        .segment_type_as::<Part>()
                        .unwrap()
                        .1
                    {
                        crate::composer::PartType::Instrument => u8idx + (u8idx / 9_u8).min(1), // Skips 9, since it is reserved for percussion
                        crate::composer::PartType::Percussion => 9_u8,
                    }
                };

                let mut track = Self::convert_subtree(subtree_root, tree, channel);
                if u8idx == 0 {
                    track.insert(
                        0,
                        TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000.into())),
                        },
                    )
                }
                track.append(&mut vec![TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
                }]);
                track
            })
            .collect();

        Smf {
            header: Header {
                format: Parallel,
                timing: Metrical(480.into()),
            },
            tracks,
        }
    }

    fn convert_subtree<'a>(
        subtree_root: &Node<RenderSegment>,
        tree: &'a Tree<RenderSegment>,
        channel: u8,
    ) -> Vec<TrackEvent<'a>> {
        let mut abs_time_events: Vec<(i32, TrackEvent)> = tree
            .node_iter(subtree_root)
            .flat_map(|n| {
                if let Some(instrument) = n.value.segment.segment_type_as::<Instrument>() {
                    Some(vec![(
                        n.value.segment.begin,
                        TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::ProgramChange {
                                    program: u8::from(*instrument).into(),
                                },
                            },
                        },
                    )])
                } else if let Some(play_note) = n.value.segment.segment_type_as::<PlayNote>() {
                    Some(vec![
                        (
                            n.value.segment.begin,
                            TrackEvent {
                                delta: 0.into(),
                                kind: TrackEventKind::Midi {
                                    channel: channel.into(),
                                    message: MidiMessage::NoteOn {
                                        key: play_note.note.into(),
                                        vel: play_note.velocity.into(),
                                    },
                                },
                            },
                        ),
                        (
                            n.value.segment.end,
                            TrackEvent {
                                delta: 0.into(),
                                kind: TrackEventKind::Midi {
                                    channel: channel.into(),
                                    message: MidiMessage::NoteOff {
                                        key: play_note.note.into(),
                                        vel: play_note.velocity.into(),
                                    },
                                },
                            },
                        ),
                    ])
                } else {
                    None
                }
            })
            .flatten()
            .collect();

        abs_time_events.sort_by(|a, b| {
            let time_comparison = a.0.cmp(&b.0);
            match time_comparison {
                Ordering::Equal => {
                    // ProgramChange messages should come before others, assuming equal timing
                    match (a.1.kind, b.1.kind) {
                        (
                            TrackEventKind::Midi {
                                message: MidiMessage::ProgramChange { .. },
                                ..
                            },
                            TrackEventKind::Midi {
                                message: MidiMessage::ProgramChange { .. },
                                ..
                            },
                        ) => Ordering::Equal,
                        (
                            TrackEventKind::Midi {
                                message: MidiMessage::ProgramChange { .. },
                                ..
                            },
                            _,
                        ) => Ordering::Less,
                        (
                            _,
                            TrackEventKind::Midi {
                                message: MidiMessage::ProgramChange { .. },
                                ..
                            },
                        ) => Ordering::Greater,
                        _ => Ordering::Equal,
                    }
                }
                _ => time_comparison,
            }
        });

        let mut curr_time: i32 = 0;
        for (timing, track_event) in &mut abs_time_events {
            track_event.delta = ((*timing - curr_time) as u32).into();
            curr_time = *timing;
        }

        abs_time_events.iter().map(|t| t.1).collect()
    }
}
