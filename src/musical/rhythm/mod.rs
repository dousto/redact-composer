use std::ops::Range;

use rand::{seq::SliceRandom, Rng};

#[cfg(test)]
mod test;

pub const STANDARD_BEAT_LENGTH: i32 = 480;
pub const HIGH_PRECISION_BEAT_LENGTH: i32 = STANDARD_BEAT_LENGTH * 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subdivision {
    pub timing: Range<i32>,
    pub is_rest: bool,
}

impl From<Vec<Subdivision>> for Rhythm {
    fn from(value: Vec<Subdivision>) -> Self {
        Rhythm(value)
    }
}

/// Represents a rhythm as a sequence of timing divisions ([`Vec<Subdivision>`]).
#[derive(Debug, Clone)]
pub struct Rhythm(pub Vec<Subdivision>);
impl Rhythm {
    /// Generates a random rhythm where subdivision lengths are balanced to be either `n` or `2n` for some `n >= min_subdivision`.
    pub fn balanced_timing(
        length: i32,
        subdivisions: i32,
        min_subdivision: i32,
        rng: &mut impl Rng,
    ) -> Rhythm {
        if min_subdivision * subdivisions > length {
            panic!("Rhythm of length {:?} with {:?} subdivisions cannot be achieved with min_subdivision={:?}.",
            length, subdivisions, min_subdivision)
        }

        let mut rhythm = vec![min_subdivision; subdivisions as usize];

        // Double one of the smallest subdivisions, until rhythm length fits the target length
        let mut rhythm_length: i32 = rhythm.iter().sum();
        while length - rhythm_length >= min_subdivision {
            let min_rhythm = rhythm.iter().min().unwrap();
            let mut min_indices: Vec<usize> = rhythm
                .iter()
                .enumerate()
                .filter(|(_, r)| r == &min_rhythm)
                .map(|(i, _)| i)
                .collect();
            min_indices.shuffle(rng);
            let selected_index = min_indices[rng.gen_range(0..min_indices.len())];
            rhythm[selected_index] *= 2;

            rhythm_length = rhythm.iter().sum()
        }

        Self(
            rhythm
                .into_iter()
                .scan(0, |start, x| {
                    let subdivision = (*start, *start + x);
                    *start = subdivision.1;

                    Some(Subdivision {
                        timing: subdivision.0..subdivision.1,
                        is_rest: false,
                    })
                })
                .collect(),
        )
    }

    pub fn random(
        length: i32,
        division_probability: impl Fn(i32) -> f32,
        rest_probability: impl Fn(i32) -> f32,
        rng: &mut impl Rng,
    ) -> Rhythm {
        let mut rhythm_build = vec![(length, false)];

        while !rhythm_build.iter().all(|(_, stop_dividing)| *stop_dividing) {
            rhythm_build = rhythm_build
                .into_iter()
                .flat_map(|(current_division, stop_dividing)| {
                    if !stop_dividing
                        && rng.gen_bool(
                            division_probability(current_division)
                                .clamp(0.0, 1.0)
                                .into(),
                        )
                    {
                        // if current_division % 2 == 0 {
                        if division_probability(current_division / 4) == 0.0
                            && division_probability(current_division / 2) != 0.0
                        {
                            let quarter_division = current_division / 4;
                            let first_division = rng.gen_range(1..=3);
                            let second_division = 4 - first_division;

                            vec![
                                (first_division * quarter_division, first_division == 3),
                                (second_division * quarter_division, second_division == 3),
                            ]
                        } else {
                            vec![(current_division / 2, false); 2]
                        }
                        // } else if current_division % 3 == 0 {
                        //     let thirds_division = current_division / 3;
                        //     let first_division = rng.gen_range(1..=2);
                        //     let second_division = 3 - first_division;

                        //     vec![(first_division * thirds_division, false), (second_division * thirds_division, false)]
                        // }  else {
                        //     vec![(current_division, true)]
                        // }
                    } else {
                        vec![(current_division, true)]
                    }
                })
                .collect();
        }

        Self(
            rhythm_build
                .into_iter()
                .scan(0, |start, (x, _)| {
                    let subdivision = (*start, *start + x);
                    *start = subdivision.1;

                    Some(Subdivision {
                        timing: subdivision.0..subdivision.1,
                        is_rest: rng.gen_bool(rest_probability(x).clamp(0.0, 1.0).into()),
                    })
                })
                .collect(),
        )
    }

    /// Returns the length of the rhythm in ticks. Includes leading and trailing rests.
    pub fn len(&self) -> i32 {
        self.0.last().map(|r| r.timing.end).unwrap_or_default()
    }

    pub fn over(&self, range: Range<i32>) -> Vec<Subdivision> {
        self.iter_over(range).collect()
    }

    pub fn iter_over(&self, range: Range<i32>) -> impl Iterator<Item = Subdivision> + '_ {
        let rhythm_length = self.len();

        self.0
            .iter()
            .cycle()
            .scan(range.start, move |offset, subdivision| {
                if let Some(last_division) = self.0.last() {
                    let offset_subdivision = Subdivision {
                        timing: (*offset + subdivision.timing.start)
                            ..(*offset + subdivision.timing.end),
                        is_rest: subdivision.is_rest,
                    };

                    if subdivision == last_division {
                        *offset += rhythm_length
                    }

                    if offset_subdivision.timing.end <= range.end {
                        Some(offset_subdivision)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
    }
}
