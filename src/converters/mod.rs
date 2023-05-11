use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use std::{convert::identity, marker::PhantomData};

use crate::composer::{Node, RenderSegment, SegmentType, Tree};

pub struct MidiConverter<T> {
    marker: PhantomData<T>,
}

impl<T> MidiConverter<T> {
    pub fn convert(tree: &Tree<RenderSegment<T>>) -> Smf {
        let track_subtrees: Vec<&Node<RenderSegment<T>>> = tree
            .iter()
            .filter(|n| match n.value.segment.segment_type {
                SegmentType::Part(_) => true,
                _ => false,
            })
            .collect();

        let tracks: Vec<Vec<TrackEvent>> = track_subtrees
            .iter()
            .enumerate()
            .map(|(idx, subtree_root)| {
                let u8idx: u8 = idx.try_into().unwrap();
                let mut track = Self::convert_subtree(subtree_root, tree, u8idx);
                if u8idx == 0 {
                    track.insert(
                        0,
                        TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000.into())),
                        },
                    )
                }
                track
            })
            .collect();

        Smf {
            header: Header {
                format: Parallel,
                timing: Metrical(480.into()),
            },
            tracks: tracks,
        }
    }

    fn convert_subtree<'a>(
        subtree_root: &Node<RenderSegment<T>>,
        tree: &'a Tree<RenderSegment<T>>,
        channel: u8,
    ) -> Vec<TrackEvent<'a>> {
        let mut abs_time_events: Vec<(u32, TrackEvent)> = tree
            .node_iter(subtree_root)
            .flat_map(|n| match n.value.segment.segment_type {
                SegmentType::PlayNote { note, velocity } => Some(vec![
                    (
                        n.value.segment.begin,
                        TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::NoteOn {
                                    key: note.into(),
                                    vel: velocity.into(),
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
                                    key: note.into(),
                                    vel: velocity.into(),
                                },
                            },
                        },
                    ),
                ]),
                SegmentType::Instrument { program } => Some(vec![(
                    n.value.segment.begin,
                    TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Midi {
                            channel: channel.into(),
                            message: MidiMessage::ProgramChange {
                                program: program.into(),
                            },
                        },
                    },
                )]),
                _ => None,
            })
            .flat_map(identity)
            .collect();

        abs_time_events.sort_by_key(|k| k.0);

        let mut curr_time: u32 = 0;
        for (timing, track_event) in &mut abs_time_events {
            track_event.delta = (*timing - curr_time).into();
            curr_time = *timing;
        }

        abs_time_events.iter().map(|t| t.1).collect()
    }
}
