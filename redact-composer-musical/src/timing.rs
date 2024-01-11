#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// A musical time signature as a combination of beats per bar, and beat length.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "redact-composer", derive(Element))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TimeSignature {
    /// The number of beats per bar.
    pub beats_per_bar: i32,
    /// How many ticks a beat represents.
    pub beat_length: i32,
}

impl TimeSignature {
    /// Length of a bar in ticks.
    pub fn bar(&self) -> i32 {
        self.beats_per_bar * self.beat_length
    }

    /// The length of `n` bars in ticks.
    pub fn bars(&self, n: i32) -> i32 {
        self.bar() * n
    }

    /// Length of a beat in ticks.
    pub fn beat(&self) -> i32 {
        self.beat_length
    }

    /// The length of `n` beats in ticks.
    pub fn beats(&self, n: i32) -> i32 {
        self.beat() * n
    }

    /// Length of a half-beat in ticks.
    pub fn half_beat(&self) -> i32 {
        self.beat_length / 2
    }

    /// The length of `n` half-beats in ticks.
    pub fn half_beats(&self, n: i32) -> i32 {
        self.half_beat() * n
    }

    /// Length of a triplet in ticks. (2/3 of a beat)
    pub fn triplet(&self) -> i32 {
        self.beat_length * 2 / 3
    }

    /// The length of `n` triplets in ticks.
    pub fn triplets(&self, n: i32) -> i32 {
        self.triplet() * n
    }

    /// Length of a half-triplet in ticks. (1/3 of a beat)
    pub fn half_triplet(&self) -> i32 {
        self.beat_length / 3
    }

    /// The length of `n` half-triplets in ticks.
    pub fn half_triplets(&self, n: i32) -> i32 {
        self.half_triplet() * n
    }

    /// Length of a quarter-beat in ticks.
    pub fn quarter_beat(&self) -> i32 {
        self.beat_length / 4
    }

    /// The length of `n` quarter-beats in ticks.
    pub fn quarter_beats(&self, n: i32) -> i32 {
        self.quarter_beat() * n
    }

    /// Length of an eighth-beat in ticks.
    pub fn eighth_beat(&self) -> i32 {
        self.beat_length / 8
    }

    /// The length of `n` eighth-beats in ticks.
    pub fn eighth_beats(&self, n: i32) -> i32 {
        self.eighth_beat() * n
    }
}
