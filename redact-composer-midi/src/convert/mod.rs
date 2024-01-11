use crate::elements::Program;
use log::{debug, info, log_enabled, warn, Level};
use midly::num::u4;
use midly::{
    Format::Parallel, Header, MetaMessage, MidiMessage, Smf, Timing::Metrical, TrackEvent,
    TrackEventKind,
};
use redact_composer_core::timing::Timing;
use redact_composer_core::{
    elements::{Part, PlayNote},
    render::{
        tree::{Node, Tree},
        RenderSegment,
    },
    timing::elements::Tempo,
    Composition, PartType, SegmentRef,
};
use std::{cmp::Ordering, collections::HashSet};

#[cfg(test)]
mod test;

/// Converter for [`Composition`] -> MIDI format.
#[allow(missing_debug_implementations)]
pub struct MidiConverter;

impl MidiConverter {
    /// Converts [`Composition`]s into MIDI format using the [`midly`] crate.
    pub fn convert(composition: &Composition) -> Smf {
        info!("Converting to MIDI.");
        let track_subtrees: Vec<&Node<RenderSegment>> = composition
            .tree
            .iter()
            .filter(|n| n.value.segment.element_as::<Part>().is_some())
            .collect();

        let channel_assignments = Self::assign_channels(&track_subtrees);

        if channel_assignments.iter().any(Option::is_none) {
            warn!("Warning: Some parts could not be assigned a channel due to too many concurrent parts.");
            warn!(
                "Maximum allowed concurrent Parts: (Instrument: {:?}, Percussion: {:?})",
                Self::instrument_channels().len(),
                Self::drum_channels().len()
            );
        }

        let tracks: Vec<Vec<TrackEvent>> = track_subtrees
            .into_iter()
            .zip(channel_assignments.iter())
            .filter_map(|(node, opt_ch)| opt_ch.map(|ch| (node, ch)))
            .map(|(subtree_root, channel)| {
                let initial_events = if channel == 0 {
                    Some(Self::extract_tempo_events(&composition.tree))
                } else {
                    None
                };

                let mut track =
                    Self::convert_subtree(subtree_root, &composition.tree, channel, initial_events);

                track.append(&mut vec![TrackEvent {
                    delta: 0.into(),
                    kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
                }]);
                track
            })
            .collect();

        if log_enabled!(Level::Info) {
            let used_channels = channel_assignments
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let drum_channels = Self::drum_channels().into_iter().collect::<Vec<_>>();
            let instrument_channels = Self::instrument_channels().into_iter().collect::<Vec<_>>();
            let used_drum_channels = (0..u4::max_value().into())
                .filter(|ch| drum_channels.contains(ch) && used_channels.contains(ch))
                .collect::<Vec<_>>();
            let used_instrument_channels = (0..u4::max_value().into())
                .filter(|ch| instrument_channels.contains(ch) && used_channels.contains(ch))
                .collect::<Vec<_>>();

            info!("MIDI conversion complete. Total events: {:?}. Channels used: Instrument: {:?}, Percussion: {:?}.",
                tracks.iter().map(Vec::len).sum::<usize>(), used_instrument_channels, used_drum_channels);
        }

        Smf {
            header: Header {
                format: Parallel,
                timing: Metrical((composition.options.ticks_per_beat as u16).into()),
            },
            tracks,
        }
    }

    fn drum_channels() -> HashSet<u8> {
        HashSet::from_iter([9])
    }

    fn instrument_channels() -> HashSet<u8> {
        let drum_channels = Self::drum_channels();
        (0..=u4::max_value().into())
            .filter(|ch| !drum_channels.contains(ch))
            .collect()
    }

    fn assign_channels(parts: &[&Node<RenderSegment>]) -> Vec<Option<u8>> {
        let mut drum_channels: HashSet<u8> = Self::drum_channels();
        let mut inst_channels: HashSet<u8> = Self::instrument_channels();

        let mut part_times: Vec<(&Node<RenderSegment>, Option<u8>)> =
            parts.iter().map(|p| (*p, None)).collect();

        let mut sorted_part_times: Vec<&mut (&Node<RenderSegment>, Option<u8>)> =
            part_times.iter_mut().collect();
        sorted_part_times.sort_by_key(|(p, _)| p.value.segment.timing.start);

        let mut temp_channels: Vec<(&Node<RenderSegment>, u8)> = vec![];
        for (next_part, opt_ch) in sorted_part_times {
            // release temp channels for reuse if they're past the next part's start time
            let mut i: usize = 0;
            while i < temp_channels.len() {
                if temp_channels[i].0.value.segment.timing.end
                    <= next_part.value.segment.timing.start
                {
                    match temp_channels[i]
                        .0
                        .value
                        .segment
                        .element_as::<Part>()
                        .unwrap()
                        .part_type()
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
            let channel_pool = match next_part
                .value
                .segment
                .element_as::<Part>()
                .unwrap()
                .part_type()
            {
                PartType::Instrument => &mut inst_channels,
                PartType::Percussion => &mut drum_channels,
            };

            let available_channel: Option<u8> = channel_pool.iter().min().copied();

            if let Some(ch) = available_channel {
                *opt_ch = Some(ch);
                channel_pool.remove(&ch);
                temp_channels.insert(temp_channels.len(), (next_part, ch));
            } else {
                debug!(
                    "Could not assign channel for {:?} (idx: {:?}). \
                All available channels are occupied during this time.",
                    next_part.value.segment, next_part.idx
                );
            };
        }

        part_times.into_iter().map(|(_, ch)| ch).collect()
    }

    fn extract_tempo_events(tree: &Tree<RenderSegment>) -> Vec<(i32, TrackEvent<'_>)> {
        let timing = if let Some(root) = tree.root() {
            root.value.segment.timing
        } else {
            return vec![];
        };

        let default_tempo = Tempo::from_bpm(120);
        let spanning_tempos = tree
            .iter()
            .filter_map(|n| (&n.value.segment).try_into().ok())
            .fold(
                vec![(&default_tempo, timing)],
                |mut tempos, tempo: SegmentRef<Tempo>| {
                    // Find the position of the first existing tempo starting after/at the new tempo
                    let start_overlap =
                        tempos.partition_point(|(_, timing)| timing.start < tempo.timing.start);
                    // Find the position of the first existing tempo ending before the new tempo
                    let end_overlap =
                        tempos.partition_point(|(_, timing)| tempo.timing.end >= timing.end);

                    if start_overlap > end_overlap {
                        // This is the case if the new tempo is within an existing tempo segment
                        // In this case the new tempo needs to be spliced within an existing tempo segment
                        let splice_tempo = tempos.remove(end_overlap);

                        let first_split = (
                            splice_tempo.0,
                            Timing::from(splice_tempo.1.start..tempo.timing.start),
                        );
                        let last_split = (
                            splice_tempo.0,
                            Timing::from(tempo.timing.end..splice_tempo.1.end),
                        );

                        tempos.insert(end_overlap, first_split);
                        tempos.insert(end_overlap + 1, (tempo.element, *tempo.timing));
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
                            ele.1.end = ele.1.end.min(tempo.timing.start);
                        }

                        // Update the existing tempo segment (after the cut region) and update its
                        // timing to start at the inserted tempo's end time
                        if let Some(ele) = tempos.get_mut(start_overlap) {
                            ele.1.start = ele.1.start.max(tempo.timing.end);
                        }

                        tempos.insert(start_overlap, (tempo.element, *tempo.timing));
                    }

                    tempos
                },
            );

        // Convert each tempo segment into a midi event
        spanning_tempos
            .into_iter()
            .map(|(tempo, timing)| {
                (
                    timing.start,
                    TrackEvent {
                        delta: 0.into(),
                        kind: TrackEventKind::Meta(MetaMessage::Tempo(
                            tempo.microseconds_per_beat().into(),
                        )),
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
            .filter_map(|n| {
                if let Some(instrument) = n.value.segment.element_as::<Program>() {
                    Some(vec![(
                        n.value.segment.timing.start,
                        TrackEvent {
                            delta: 0.into(),
                            kind: TrackEventKind::Midi {
                                channel: channel.into(),
                                message: MidiMessage::ProgramChange {
                                    program: instrument.0.into(),
                                },
                            },
                        },
                    )])
                } else {
                    n.value.segment.element_as::<PlayNote>().map(|play_note| {
                        vec![
                            (
                                n.value.segment.timing.start,
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
                                n.value.segment.timing.end,
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
                        ]
                    })
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
