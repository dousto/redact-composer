use std::cmp::Ordering;
use std::ops::{Add, Range};

use crate::rhythm::DivType::Div;
use crate::timing::TimeSignature;
use rand::distributions::WeightedIndex;
use rand::{seq::SliceRandom, Rng};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "redact-composer")]
use redact_composer_core::derive::Element;

#[cfg(test)]
mod test;

/// A rhythm subdivision.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Subdivision {
    /// The starting time of the subdivision relative to the rhythm's start.
    pub start: i32,
    /// The end time of the subdivision relative to the rhythm's start.
    pub end: i32,
    /// `true` if the subdivision is a rest.
    pub is_rest: bool,
}

impl Subdivision {
    /// Returns the subdivision's timing as a [`Range<i32>`].
    pub fn timing(&self) -> Range<i32> {
        self.start..self.end
    }
}

impl From<Subdivision> for Range<i32> {
    fn from(value: Subdivision) -> Self {
        value.timing()
    }
}

impl From<&Subdivision> for Range<i32> {
    fn from(value: &Subdivision) -> Self {
        value.timing()
    }
}

impl From<Range<i32>> for Subdivision {
    fn from(value: Range<i32>) -> Self {
        Subdivision {
            start: value.start,
            end: value.end,
            is_rest: false,
        }
    }
}

impl From<&Range<i32>> for Subdivision {
    fn from(value: &Range<i32>) -> Self {
        Subdivision {
            start: value.start,
            end: value.end,
            is_rest: false,
        }
    }
}

impl From<i32> for Subdivision {
    fn from(value: i32) -> Self {
        Subdivision {
            start: 0,
            end: value,
            is_rest: false,
        }
    }
}

impl From<&i32> for Subdivision {
    fn from(value: &i32) -> Self {
        Subdivision {
            start: 0,
            end: *value,
            is_rest: false,
        }
    }
}

impl<I: IntoIterator<Item = T>, T: Into<Subdivision>> From<I> for Rhythm {
    fn from(value: I) -> Self {
        let mut stuff = value
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Subdivision>>();
        stuff.sort_by_key(|div| div.start);

        let result = stuff
            .into_iter()
            .scan(None::<Subdivision>, |opt_prev, item| {
                let next = match opt_prev {
                    None => {
                        vec![item]
                    }
                    Some(prev) => {
                        if item.start == 0 {
                            vec![Subdivision {
                                start: prev.end,
                                end: item.end + prev.end,
                                is_rest: false,
                            }]
                        } else if item.end <= prev.end {
                            vec![]
                        } else if item.start > prev.end {
                            vec![
                                Subdivision {
                                    start: prev.end,
                                    end: item.start,
                                    is_rest: true,
                                },
                                Subdivision {
                                    start: item.start,
                                    end: item.end,
                                    is_rest: item.is_rest,
                                },
                            ]
                        } else {
                            vec![Subdivision {
                                start: prev.end,
                                end: item.end,
                                is_rest: item.is_rest,
                            }]
                        }
                    }
                };

                if let Some(next) = next.last() {
                    opt_prev.replace(*next);
                }

                Some(next)
            })
            .flatten()
            .collect::<Vec<_>>();

        Rhythm(result)
    }
}

/// Rhythm division type used during construction of [`Rhythm`]s.
#[derive(Debug)]
pub enum DivType {
    /// Non-rest subdivision length.
    Div(i32),
    /// Rest subdivision length.
    Rest(i32),
}

impl From<DivType> for Subdivision {
    fn from(value: DivType) -> Self {
        match value {
            Div(len) => Subdivision {
                start: 0,
                end: len,
                is_rest: false,
            },
            DivType::Rest(len) => Subdivision {
                start: 0,
                end: len,
                is_rest: true,
            },
        }
    }
}

impl Add<Rhythm> for Rhythm {
    type Output = Rhythm;

    fn add(self, mut rhs: Rhythm) -> Self::Output {
        let first_length = self.len();
        let mut new_subdivisions = self.0.into_iter().collect::<Vec<_>>();
        new_subdivisions.append(&mut rhs.offset(first_length).0);

        Rhythm(new_subdivisions)
    }
}

/// Represents a rhythm as a sequence of timing divisions ([`Vec<Subdivision>`]).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "redact-composer", derive(Element))]
pub struct Rhythm(pub Vec<Subdivision>);
impl Rhythm {
    /// Creates a new empty rhythm.
    pub fn new() -> Rhythm {
        Rhythm(vec![])
    }

    /// Returns an iterator over the rhythm's non-rest subdivisions.
    pub fn iter(&self) -> impl Iterator<Item = &Subdivision> {
        self.0.iter().filter(|div| !div.is_rest)
    }

    /// Returns an iterator over the rhythm's subdivisions (both rests and non-rests).
    pub fn iter_including_rests(&self) -> impl Iterator<Item = &Subdivision> {
        self.0.iter()
    }

    /// Generates a random rhythm where subdivision lengths are relatively balanced.
    pub fn balanced_timing(
        length: i32,
        subdivisions: i32,
        time_signature: &TimeSignature,
        rng: &mut impl Rng,
    ) -> Rhythm {
        let ts = time_signature;
        let mut rhythm = vec![length];

        let lengths = [ts.bar(), ts.beat(), ts.half_beat(), ts.quarter_beat()];

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

        Rhythm::from(rhythm)
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
                    bar_divisions.push(div % ts.bar());
                }

                Some(vec![bar_divisions])
            }
            div if div > ts.beats(2) => Some(vec![
                vec![ts.beats(2), div - ts.beats(2)],
                vec![ts.beat(), div - ts.beat()],
            ]),
            div if div == ts.beats(2) => Some(vec![
                vec![ts.triplet(); 3],
                vec![ts.beat() + ts.half_beat(), ts.half_beat()],
                vec![ts.beat(), div - ts.beat()],
            ]),
            div if div >= ts.beat() => Some(vec![vec![ts.half_beat(), div - ts.half_beat()]]),
            div if div >= ts.half_beat() => {
                Some(vec![vec![ts.quarter_beat(), div - ts.quarter_beat()]])
            }
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

                            choice.iter().map(|b| (*b, false)).collect::<Vec<_>>()
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
                .scan((0, 0), |acc, (x, _)| {
                    acc.0 = acc.1;
                    acc.1 = acc.0 + x;

                    Some(Subdivision {
                        start: acc.0,
                        end: acc.1,
                        is_rest: rng.gen_bool(rest_probability(x).clamp(0.0, 1.0).into()),
                    })
                })
                .collect(),
        )
    }

    /// Generates a random rhythm using a set of sub-sequences of subdivision lengths. Random sub-sequences are chosen
    /// until their combined length reaches the target length.
    pub fn random_with_subdivisions_weights(
        length: i32,
        subdivision_weights: &[(Vec<i32>, i32)],
        rng: &mut impl Rng,
    ) -> Rhythm {
        let mut timings = vec![];
        let mut sum = 0;

        if subdivision_weights
            .iter()
            .all(|(t, _)| t.iter().sum::<i32>() > length)
        {
            return Rhythm::new();
        }

        while sum < length {
            let remaining = length - sum;
            let sizeable_choices = subdivision_weights
                .iter()
                .filter(|(t, _)| t.iter().sum::<i32>() <= remaining)
                .collect::<Vec<_>>();
            let dist = WeightedIndex::new(sizeable_choices.iter().map(|(_, i)| i)).unwrap();

            let choice = &sizeable_choices[rng.sample(dist)].0;

            timings.push(choice.clone());
            sum += choice.iter().sum::<i32>();
        }

        timings.shuffle(rng);

        Rhythm(
            timings
                .into_iter()
                .flatten()
                .scan((0, 0), |acc, x| {
                    acc.0 = acc.1;
                    acc.1 = acc.0 + x;
                    Some(*acc)
                })
                .map(|t| Subdivision {
                    start: t.0,
                    end: t.1,
                    is_rest: false,
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Returns a new [`Rhythm`], based on the input [`Rhythm`] offset by a given `amount`.
    pub fn offset(&mut self, amount: i32) -> Rhythm {
        Rhythm(
            self.0
                .iter()
                .map(|s| Subdivision {
                    start: s.start + amount,
                    end: s.end + amount,
                    is_rest: s.is_rest,
                })
                .collect::<Vec<_>>(),
        )
    }

    /// Creates a new [`Rhythm`] from this one sized according to the `frame_size`. If the rhythm is
    /// smaller than `frame_size` it it will be extended with rest. If the rhythm is larger
    /// than `frame_size` it will be truncated to fit `frame_size` exactly.
    pub fn frame(&self, frame_size: i32) -> Rhythm {
        let mut divs = self
            .0
            .iter()
            .take_while(|div| div.start < frame_size)
            .copied()
            .collect::<Vec<_>>();

        if let Some(last) = divs.last_mut() {
            let copied_end = last.end;
            match last.end.cmp(&frame_size) {
                Ordering::Less => {
                    if last.is_rest {
                        last.end = frame_size;
                    } else {
                        divs.push(Subdivision {
                            start: copied_end,
                            end: frame_size,
                            is_rest: true,
                        });
                    }
                }
                Ordering::Greater => {
                    last.end = frame_size;
                }
                Ordering::Equal => {}
            }

            Rhythm(divs)
        } else {
            Rhythm(Vec::new())
        }
    }

    /// Convenience alias of [`Self::frame`].
    pub fn repeat_every(&self, length: i32) -> Rhythm {
        self.frame(length)
    }

    /// Returns the length of the rhythm in ticks. Includes leading and trailing rests.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> i32 {
        self.0.last().map(|r| r.end).unwrap_or_default()
    }

    /// Repeats the [`Rhythm`] over the given time range. If the range is smaller than the rhythm
    /// however, it will be truncated to fit.
    pub fn over(&self, range: impl Into<Range<i32>>) -> Vec<Subdivision> {
        self.iter_over(range.into()).collect()
    }

    /// Iterates over rhythm [`Subdivision`]s over the time range, following the rhythmic pattern
    /// of this [`Rhythm`]. The rhythm will repeat if the time range is longer than the rhythm,
    /// or be truncated if shorter.
    pub fn iter_over<'a>(
        &'a self,
        range: impl Into<Range<i32>> + 'a,
    ) -> impl Iterator<Item = Subdivision> + '_ {
        let rhythm_length = self.len();
        let range = range.into();

        self.0
            .iter()
            .cycle()
            .scan(range.start, move |offset, subdivision| {
                if let Some(last_division) = self.0.last() {
                    let offset_subdivision = Subdivision {
                        start: *offset + subdivision.start,
                        end: *offset + subdivision.end,
                        is_rest: subdivision.is_rest,
                    };

                    if subdivision == last_division {
                        *offset += rhythm_length;
                    }

                    if offset_subdivision.end <= range.end {
                        Some(offset_subdivision)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter(|div| !div.is_rest)
    }
}

impl Default for Rhythm {
    fn default() -> Self {
        Rhythm::new()
    }
}
