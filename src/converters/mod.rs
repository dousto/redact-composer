use crate::composer::{
    render::{Node, Tree},
    Instrument, Part, PartType, PlayNote, RenderSegment,
};
use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use std::{cmp::Ordering, collections::HashSet};

pub struct MidiConverter;

impl MidiConverter {
    /// Converts the given render tree ([`Tree<RenderSegment>`]) into MIDI format using the [midly] crate (special thanks to its creators!).
    pub fn convert(tree: &Tree<RenderSegment>) -> Smf {
        let track_subtrees: Vec<&Node<RenderSegment>> = tree
            .iter()
            .filter(|n| n.value.segment.segment_type_as::<Part>().is_some())
            .collect();

        let channel_assignments = Self::assign_channels(&track_subtrees);

        if channel_assignments.iter().any(|opt_ch| opt_ch.is_none()) {
            println!("Warning: Some parts could not be assigned a channel due to too many simultaneous parts.");
        }

        let tracks: Vec<Vec<TrackEvent>> = track_subtrees
            .into_iter()
            .zip(channel_assignments)
            .filter(|(_, opt_ch)| opt_ch.is_some())
            .map(|(subtree_root, opt_ch)| {
                let channel = opt_ch.unwrap();

                let mut track = Self::convert_subtree(subtree_root, tree, channel);
                if channel == 0 {
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

    fn assign_channels(parts: &[&Node<RenderSegment>]) -> Vec<Option<u8>> {
        let mut drum_channels: HashSet<u8> = HashSet::from_iter([9].into_iter());
        let mut inst_channels: HashSet<u8> =
            HashSet::from_iter((0..=16).filter(|ch| !drum_channels.contains(ch)));

        let mut part_times: Vec<(&Node<RenderSegment>, Option<u8>)> =
            parts.iter().map(|p| (*p, None)).collect();

        let mut sorted_part_times: Vec<&mut (&Node<RenderSegment>, Option<u8>)> =
            part_times.iter_mut().collect();
        sorted_part_times.sort_by_key(|(p, _)| p.value.segment.begin);

        let mut temp_channels: Vec<(&Node<RenderSegment>, u8)> = vec![];
        for (next_part, opt_ch) in sorted_part_times {
            // release temp channels for reuse if they're past the next part's start time
            let mut i: usize = 0;
            while i < temp_channels.len() {
                if temp_channels[i].0.value.segment.end <= next_part.value.segment.begin {
                    match temp_channels[i]
                        .0
                        .value
                        .segment
                        .segment_type_as::<Part>()
                        .unwrap()
                        .1
                    {
                        PartType::Instrument => inst_channels.insert(temp_channels[i].1),
                        PartType::Percussion => drum_channels.insert(temp_channels[i].1),
                    };

                    temp_channels.remove(i);
                    // No increment here because the removal shifts elements past `i` left by 1
                } else {
                    i += 1;
                }
            }

            // Assign a channel from available channels
            let channel_pool = match next_part.value.segment.segment_type_as::<Part>().unwrap().1 {
                PartType::Instrument => &mut inst_channels,
                PartType::Percussion => &mut drum_channels,
            };

            let available_channel: Option<u8> = channel_pool.iter().min().copied();

            if let Some(ch) = available_channel {
                *opt_ch = Some(ch);
                channel_pool.remove(&ch);
                temp_channels.insert(temp_channels.len(), (next_part, ch));
            };
        }

        part_times.into_iter().map(|(_, ch)| ch).collect()
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
                                    program: instrument.program.into(),
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
