use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use std::{convert::identity, marker::PhantomData};

use crate::composer::{Node, RenderSegment, SegmentType};

pub struct MidiConverter<T> {
    marker: PhantomData<T>,
}

impl<T> MidiConverter<T> {
    pub fn convert(tree: &Vec<Node<RenderSegment<T>>>) -> Smf {
        let track_subtrees = vec![tree];

        let meta_track = vec![TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Meta(MetaMessage::Tempo(500_000.into())),
        }];

        let mut tracks: Vec<Vec<TrackEvent>> = track_subtrees
            .iter()
            .enumerate()
            .map(|(idx, tree)| {
                let u8idx: u8 = idx.try_into().unwrap();
                let mut track = Self::convert_subtree(tree, u8idx + 1);
                track.insert(
                    0,
                    TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Midi {
                            channel: u8idx.into(),
                            message: MidiMessage::ProgramChange { program: 0.into() },
                        },
                    },
                );
                track
            })
            .collect();

        tracks.insert(0, meta_track);

        Smf {
            header: Header {
                format: Parallel,
                timing: Metrical(480.into()),
            },
            tracks: tracks,
        }
    }

    fn convert_subtree(tree: &Vec<Node<RenderSegment<T>>>, channel: u8) -> Vec<TrackEvent> {
        let mut abs_time_events: Vec<(u32, TrackEvent)> = tree
            .iter()
            .flat_map(|n| match n.value.segment.segment_type {
                SegmentType::PlayNote { note, velocity } => Some([
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
                _ => None,
            })
            .flat_map(identity)
            .collect();

        abs_time_events.sort_by_key(|k| k.0);

        let mut curr_time: u32 = 0;
        for abs_event in &mut abs_time_events {
            abs_event.1.delta = (abs_event.0 - curr_time).into();
            curr_time = abs_event.0;
        }

        abs_time_events.iter().map(|t| t.1).collect()
    }
}
