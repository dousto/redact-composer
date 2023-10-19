use crate::composer::{
    render::{Node, Tree},
    Instrument, Part, PartType, PlayNote, RenderSegment, TypedSegment,
};
use crate::musical::rhythm::STANDARD_BEAT_LENGTH;
use crate::musical::timing::Tempo;
use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use std::{cmp::Ordering, collections::HashSet};

#[cfg(test)]
mod test;

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

                let initial_events = if channel == 0 {
                    Some(Self::extract_tempo_events(tree))
                } else {
                    None
                };

                let mut track = Self::convert_subtree(subtree_root, tree, channel, initial_events);

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
                timing: Metrical((STANDARD_BEAT_LENGTH as u16).into()),
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
        sorted_part_times.sort_by_key(|(p, _)| p.value.segment.time_range.start);

        let mut temp_channels: Vec<(&Node<RenderSegment>, u8)> = vec![];
        for (next_part, opt_ch) in sorted_part_times {
            // release temp channels for reuse if they're past the next part's start time
            let mut i: usize = 0;
            while i < temp_channels.len() {
                if temp_channels[i].0.value.segment.time_range.end
                    <= next_part.value.segment.time_range.start
                {
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

    fn extract_tempo_events<'a>(tree: &'a Tree<RenderSegment>) -> Vec<(i32, TrackEvent<'a>)> {
        let time_range = if let Some(root) = tree.root() {
            root.value.segment.time_range.clone()
        } else {
            return vec![];
        };

        let default_tempo = Tempo::from_bpm(120);
        let spanning_tempos = tree
            .iter()
            .flat_map(|n| (&n.value.segment).try_into().ok())
            .fold(
                vec![TypedSegment {
                    value: &default_tempo,
                    time_range,
                }],
                |mut tempos, tempo: TypedSegment<Tempo>| {
                    // Find the position of the first existing tempo starting after/at the new tempo
                    let start_overlap =
                        tempos.partition_point(|x| &x.time_range.start < &tempo.time_range.start);
                    // Find the position of the first existing tempo ending before the new tempo
                    let end_overlap =
                        tempos.partition_point(|x| &tempo.time_range.end >= &x.time_range.end);

                    if start_overlap > end_overlap {
                        // This is the case if the new tempo is within an existing tempo segment
                        // In this case the new tempo needs to be spliced within an existing tempo segment
                        let splice_tempo = tempos.remove(end_overlap);

                        let first_split = TypedSegment {
                            value: splice_tempo.value,
                            time_range: splice_tempo.time_range.start..tempo.time_range.start,
                        };
                        let last_split = TypedSegment {
                            value: splice_tempo.value,
                            time_range: tempo.time_range.end..splice_tempo.time_range.end,
                        };

                        tempos.insert(end_overlap, first_split);
                        tempos.insert(end_overlap + 1, tempo);
                        tempos.insert(end_overlap + 2, last_split);
                    } else {
                        // Cut out the existing tempos during the overlapping range
                        tempos.drain(start_overlap..end_overlap);

                        // Update the existing tempo segment (before the cut region) and update its
                        // timing to end at the inserted tempo's start time
                        if let Some(ele) = if start_overlap == 0 {
                            None
                        } else {
                            tempos.get_mut(start_overlap - 1)
                        } {
                            ele.time_range.end = ele.time_range.end.min(tempo.time_range.start)
                        }

                        // Update the existing tempo segment (after the cut region) and update its
                        // timing to start at the inserted tempo's end time
                        if let Some(ele) = tempos.get_mut(start_overlap) {
                            ele.time_range.start = ele.time_range.start.max(tempo.time_range.end)
                        }

                        tempos.insert(start_overlap, tempo);
                    }

                    tempos
                },
            );

        // Convert each tempo segment into a midi event
        spanning_tempos
            .into_iter()
            .map(|t| {
                (
                    t.time_range.start,
                    TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Meta(MetaMessage::Tempo(t.value.µspb().into())),
                    },
                )
            })
            .collect::<Vec<_>>()
    }

    fn convert_subtree<'a>(
        subtree_root: &Node<RenderSegment>,
        tree: &'a Tree<RenderSegment>,
        channel: u8,
        initial_abs_time_events: Option<Vec<(i32, TrackEvent<'a>)>>,
    ) -> Vec<TrackEvent<'a>> {
        let mut abs_time_events: Vec<(i32, TrackEvent)> = tree
            .node_iter(subtree_root)
            .flat_map(|n| {
                if let Some(instrument) = n.value.segment.segment_type_as::<Instrument>() {
                    Some(vec![(
                        n.value.segment.time_range.start,
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
                            n.value.segment.time_range.start,
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
                            n.value.segment.time_range.end,
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

        if let Some(mut initial_events) = initial_abs_time_events {
            abs_time_events.append(&mut initial_events);
        }

        abs_time_events.sort_by(|a, b| {
            let time_comparison = a.0.cmp(&b.0);
            match time_comparison {
                Ordering::Equal => {
                    // Tempo and ProgramChange messages should come before others, assuming equal timing
                    match (a.1.kind, b.1.kind) {
                        (
                            TrackEventKind::Meta(MetaMessage::Tempo(..)),
                            TrackEventKind::Meta(MetaMessage::Tempo(..)),
                        ) => Ordering::Equal,
                        (TrackEventKind::Meta(MetaMessage::Tempo(..)), _) => Ordering::Less,
                        (_, TrackEventKind::Meta(MetaMessage::Tempo(..))) => Ordering::Greater,
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
