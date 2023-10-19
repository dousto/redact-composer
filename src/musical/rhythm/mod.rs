use std::{
    borrow::Borrow,
    ops::{Add, Range},
};

use crate::musical::timing::TimeSignature;
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

pub const STANDARD_BEAT_LENGTH: i32 = 480;
pub const HIGH_PRECISION_BEAT_LENGTH: i32 = STANDARD_BEAT_LENGTH * 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subdivision {
    pub timing: Range<i32>,
    pub is_rest: bool,
}

impl From<Vec<Subdivision>> for Rhythm {
    fn from(value: Vec<Subdivision>) -> Self {
        Rhythm(value)
    }
}

impl Add<Rhythm> for Rhythm {
    type Output = Rhythm;

    fn add(self, rhs: Rhythm) -> Self::Output {
        let first_length = self.len();
        let mut new_subdivisions = self.0.into_iter().collect::<Vec<_>>();
        new_subdivisions.append(&mut rhs.offset(first_length).0);

        Rhythm(new_subdivisions)
    }
}

/// Represents a rhythm as a sequence of timing divisions ([`Vec<Subdivision>`]).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rhythm(pub Vec<Subdivision>);
impl Rhythm {
    /// Generates a random rhythm where subdivision lengths are relatively balanced.
    pub fn balanced_timing(
        length: i32,
        subdivisions: i32,
        time_signature: &TimeSignature,
        rng: &mut impl Rng,
    ) -> Rhythm {
        let ts = time_signature;
        let mut rhythm = vec![length];

        let lengths = vec![ts.bar(), ts.beat(), ts.half_beat(), ts.half_beat() / 2];

        while (rhythm.len() as i32) < subdivisions {
            let longest = rhythm
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.cmp(b))
                .unwrap();
            let opt_subdivision = lengths.iter().find(|s| longest.1 / *s > 1);

            if let Some(subdivision) = opt_subdivision {
                rhythm.remove(longest.0);

                let split_length = (longest.1 / subdivision) / 2 * subdivision;
                rhythm.push(split_length);
                rhythm.push(longest.1 - split_length);
            } else {
                break;
            }
        }

        if (rhythm.len() as i32) != subdivisions {
            panic!("Couldn't divide into desired number of subdivisions")
        }

        rhythm.shuffle(rng);

        rhythm
            .into_iter()
            .scan(0, |acc, s| {
                let prev_end = *acc;
                *acc += s;

                Some(Subdivision {
                    timing: (prev_end)..(prev_end + s),
                    is_rest: false,
                })
            })
            .collect::<Vec<Subdivision>>()
            .into()
    }

    /// Generates a random rhythm.
    pub fn random(
        length: i32,
        time_signature: &TimeSignature,
        division_probability: impl Fn(i32) -> f32,
        rest_probability: impl Fn(i32) -> f32,
        rng: &mut impl Rng,
    ) -> Rhythm {
        let ts = time_signature;
        let mut rhythm_build = vec![(length, false)];

        let div_choices = |div: i32| match div {
            div if div > ts.bar() => {
                let mut bar_divisions = vec![ts.bar(); (div / ts.bar()) as usize];
                if div % ts.bar() != 0 {
                    bar_divisions.push(div % ts.bar())
                }

                Some(vec![bar_divisions])
            }
            div if div > ts.beat() * 2 => Some(vec![
                vec![ts.beat() * 2, div - ts.beat() * 2],
                vec![ts.beat(), div - ts.beat()],
            ]),
            div if div == ts.beat() * 2 => Some(vec![
                vec![ts.triplet(); 3],
                vec![ts.beat() + ts.half_beat(), ts.half_beat()],
                vec![ts.beat(), div - ts.beat()],
            ]),
            div if div >= ts.beat() => Some(vec![vec![ts.half_beat(), div - ts.half_beat()]]),
            _ => None,
        };

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
                        if let Some(choices) = div_choices(current_division) {
                            let choice = choices.choose(rng).unwrap();

                            choice.into_iter().map(|b| (*b, false)).collect::<Vec<_>>()
                        } else {
                            vec![(current_division, true)]
                        }
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

    pub fn offset(&self, amount: i32) -> Rhythm {
        Rhythm(
            self.0
                .iter()
                .map(|s| Subdivision {
                    timing: (s.timing.start + amount)..(s.timing.end + amount),
                    is_rest: s.is_rest,
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Returns the length of the rhythm in ticks. Includes leading and trailing rests.
    pub fn len(&self) -> i32 {
        self.0.last().map(|r| r.timing.end).unwrap_or_default()
    }

    pub fn over(&self, range: impl Borrow<Range<i32>>) -> Vec<Subdivision> {
        self.iter_over(range).collect()
    }

    pub fn iter_over<'a>(
        &'a self,
        range: impl Borrow<Range<i32>> + 'a,
    ) -> impl Iterator<Item = Subdivision> + '_ {
        let rhythm_length = self.len();

        self.0
            .iter()
            .cycle()
            .scan(range.borrow().start, move |offset, subdivision| {
                if let Some(last_division) = self.0.last() {
                    let offset_subdivision = Subdivision {
                        timing: (*offset + subdivision.timing.start)
                            ..(*offset + subdivision.timing.end),
                        is_rest: subdivision.is_rest,
                    };

                    if subdivision == last_division {
                        *offset += rhythm_length
                    }

                    if offset_subdivision.timing.end <= range.borrow().end {
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
