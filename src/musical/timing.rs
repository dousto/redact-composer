use crate::composer::context::CompositionContext;
use crate::composer::context::TimingRelation::During;
use crate::composer::render::{AdhocRenderer, RenderEngine, Renderer, RendererGroup};
use crate::composer::CompositionElement;
use crate::composer::{CompositionSegment, Part, PlayNote};
use crate::musical::midi::Instrument;
use crate::musical::rhythm::{Rhythm, Subdivision};
use core::ops::Range;
use serde::{Deserialize, Serialize};

pub fn renderers() -> RenderEngine {
    RenderEngine::new() + Metronome::renderer()
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Tempo {
    bpm: u32,
}

#[typetag::serde]
impl CompositionElement for Tempo {}

impl Tempo {
    pub fn from_bpm(bpm: u32) -> Tempo {
        Tempo { bpm }
    }

    pub fn µspb(&self) -> u32 {
        60_000_000 / self.bpm
    }

    pub fn bpm(&self) -> u32 {
        self.bpm
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TimeSignature {
    pub beats_per_bar: i32,
    pub beat_length: i32,
}

#[typetag::serde]
impl CompositionElement for TimeSignature {}

impl TimeSignature {
    pub fn bar(&self) -> i32 {
        self.beats_per_bar * self.beat_length
    }

    pub fn beat(&self) -> i32 {
        self.beat_length
    }

    pub fn half_beat(&self) -> i32 {
        self.beat_length / 2
    }

    pub fn triplet(&self) -> i32 {
        self.beat_length * 2 / 3
    }

    pub fn eighth_triplet(&self) -> i32 {
        self.beat_length / 3
    }

    pub fn sixteenth(&self) -> i32 {
        self.half_beat() / 2
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metronome;

#[typetag::serde]
impl CompositionElement for Metronome {}

impl Metronome {
    pub fn new<S: CompositionElement>() -> impl Renderer<Item = S> {
        AdhocRenderer::from(
            |_segment: &_, time_range: &Range<i32>, _context: &CompositionContext| {
                Ok(vec![CompositionSegment::new(
                    Part::instrument(Metronome),
                    time_range,
                )])
            },
        )
    }

    pub fn renderer() -> impl Renderer<Item = Self> {
        RendererGroup::new()
            + AdhocRenderer::from(
                |_segment: &Self, time_range: &Range<i32>, _context: &CompositionContext| {
                    Ok(vec![CompositionSegment::new(
                        Instrument::Woodblock,
                        time_range,
                    )])
                },
            )
            + AdhocRenderer::from(
                |_segment: &Self, time_range: &Range<i32>, context: &CompositionContext| {
                    let time_signatures = context
                        .find::<TimeSignature>()
                        .with_timing(During, time_range)
                        .require_all()?;

                    Ok(time_signatures
                        .iter()
                        .flat_map(|ts_segment| {
                            let ts = ts_segment.value;
                            let tick = Rhythm(vec![
                                Subdivision {
                                    timing: 0..(ts.sixteenth() / 2),
                                    is_rest: false,
                                },
                                Subdivision {
                                    timing: (ts.sixteenth() / 2)..ts.beat(),
                                    is_rest: true,
                                },
                            ]);

                            tick.iter_over(&ts_segment.time_range)
                                .filter(|div| !div.is_rest)
                                .enumerate()
                                .map(|(idx, s)| {
                                    CompositionSegment::new(
                                        PlayNote {
                                            note: if (idx as i32) % ts.beats_per_bar == 0 {
                                                88
                                            } else {
                                                100
                                            },
                                            velocity: 100,
                                        },
                                        s.timing,
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>())
                },
            )
    }
}
