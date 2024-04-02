use crate::{Interval, Note, PitchClass};
use std::collections::Bound;
use std::ops::RangeBounds;

/// Provides iteration over notes of specific patterns (e.g. scale/chord) within a given note range.
///
/// When implementing this trait, only [`Self::iter_notes_in_range`] is required. The default implementation of
/// [`Self::notes_in_range`] simply collects these into a [`Vec`].
pub trait NoteIterator {
    /// Returns a note iterator ([`NoteIter`]) for notes of an interval pattern within the given note range.
    fn iter_notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> NoteIter<R>;
    /// Returns all notes matching an interval pattern within the given note range.
    fn notes_in_range<R: RangeBounds<Note>>(&self, note_range: R) -> Vec<Note> {
        self.iter_notes_in_range(note_range).collect()
    }
}

/// Iteration state for [`Note`]s according to an [`Interval`] pattern within a [`RangeBounds<Note>`].
/// ```
/// use redact_composer_musical::{Note, NoteIter};
///
/// let mut iter = NoteIter::from(Note(60)..Note(63));
/// assert_eq!(iter.next(), Some(Note(60)));
/// assert_eq!(iter.next(), Some(Note(61)));
/// assert_eq!(iter.next(), Some(Note(62)));
/// assert_eq!(iter.next(), None);
/// ```
#[derive(Debug, Clone)]
pub struct NoteIter<R: RangeBounds<Note>> {
    curr: Option<Note>,
    intervals: Vec<Interval>,
    interval_idx: usize,
    bounds: R,
}

impl<R: RangeBounds<Note>> NoteIter<R> {
    /// A chromatic note iterator over notes within a given note range.
    pub fn chromatic(range: R) -> NoteIter<R> {
        NoteIter {
            curr: match range.start_bound() {
                Bound::Included(n) => Some(*n),
                Bound::Excluded(n) => Some(Note(n.0 + 1)),
                Bound::Unbounded => Some(Note(0)),
            },
            intervals: vec![Interval(1)],
            interval_idx: 0,
            bounds: range,
        }
    }

    fn next_interval(&mut self) -> Option<Interval> {
        let returned = self
            .intervals
            .get(self.interval_idx)
            .copied()
            .and_then(|i| {
                // Don't let the next interval push it past the maximum
                if self.curr.map(|n| u8::MAX - n.0 >= i.0).unwrap_or(true) {
                    Some(i)
                } else {
                    None
                }
            });

        self.interval_idx = (self.interval_idx + 1) % self.intervals.len();

        returned
    }
}

impl<R: RangeBounds<Note>> From<R> for NoteIter<R> {
    fn from(note_range: R) -> Self {
        NoteIter::chromatic(note_range)
    }
}

impl<R: RangeBounds<Note>> From<(PitchClass, Vec<Interval>, R)> for NoteIter<R> {
    /// Constructs a note iterator ([`NoteIter`]) starting from a [`PitchClass`], over an ordered sequence of
    /// [`Interval`]s relative to the starting pitch within the given note range.
    fn from(value: (PitchClass, Vec<Interval>, R)) -> Self {
        let (root, intervals, note_range) = value;
        if let Some(first) = intervals.first() {
            let start_pitch_class = root + *first;
            let mut steps = intervals.windows(2).fold(vec![], |mut acc, i| {
                acc.push(Interval(i[1].0 - i[0].0));

                acc
            });

            if steps.is_empty() {
                steps.push(Interval::Octave)
            } else {
                let sum_interval = steps.iter().copied().sum::<Interval>();
                if sum_interval.to_simple() != Interval::P1 {
                    steps.push(sum_interval.to_simple().inversion())
                }
            }

            let first_range_note = match note_range.start_bound() {
                Bound::Included(n) => *n,
                Bound::Excluded(n) => Note(n.0 + 1),
                Bound::Unbounded => Note(0),
            };

            // Find the first position within the note_range where the interval pattern can begin
            // This will align the pattern such that the first pitch is as early as possible within the note_range
            let (first_note, interval_idx_offset) = {
                let mut interval_step_idx = steps.len() - 1;
                let mut starting_pitch = start_pitch_class;
                let mut interval_offset = Interval::P1;
                let mut closest_starting_distance =
                    first_range_note.pitch_class().interval_to(&starting_pitch);

                // Walk backwards through the interval steps (up to an octave, exclusive) to find any pitches from the
                // tail end of the interval pattern that will fit within the note_range
                while interval_step_idx > 0
                    && first_range_note.pitch_class().interval_to(
                        &(starting_pitch + steps[interval_step_idx].inversion().to_simple()),
                    ) < closest_starting_distance
                    && interval_offset + steps[interval_step_idx] < Interval::Octave
                {
                    starting_pitch += steps[interval_step_idx].inversion().to_simple();
                    interval_offset += steps[interval_step_idx];
                    closest_starting_distance =
                        first_range_note.pitch_class().interval_to(&starting_pitch);
                    interval_step_idx -= 1;
                }

                let starting_note = starting_pitch.at_or_above(&first_range_note);
                if note_range.contains(&starting_note) {
                    (Some(starting_note), (interval_step_idx + 1) % steps.len())
                } else {
                    (None, 0)
                }
            };

            NoteIter {
                curr: first_note,
                intervals: steps,
                interval_idx: interval_idx_offset,
                bounds: note_range,
            }
        } else {
            // Return empty iterator
            NoteIter {
                curr: None,
                intervals: Vec::new(),
                interval_idx: Default::default(),
                bounds: note_range,
            }
        }
    }
}

impl<R> Iterator for NoteIter<R>
where
    R: RangeBounds<Note>,
{
    type Item = Note;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(note) = self.curr {
            let return_note = if self.bounds.contains(&note) {
                Some(note)
            } else {
                None
            };

            self.curr = self.next_interval().map(|interval| note + interval);

            return_note
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Chord, ChordShape, Interval, IntervalCollection, Key, Mode, Note, NoteIterator, PitchClass,
        PitchClassCollection, Scale,
    };

    #[test]
    fn key_notes_boundary_test() {
        let roots = vec![PitchClass(0), PitchClass(9), PitchClass(11)];
        let scales = Scale::values();
        let modes = vec![Mode::Ionian, Mode::Dorian, Mode::Aeolian, Mode::Locrian];
        let lengths = [0, 1, 11, 12, 13, 23];
        let offsets = [0, 1, 11, 12, 13, 23];

        // let mut seq = 0_usize;
        for root in roots.clone() {
            for scale in scales.clone() {
                for mode in modes.clone() {
                    for length in lengths {
                        for offset in offsets {
                            let key = Key::from((root, scale, mode));
                            let key_pitches = key.pitch_classes();

                            let note_range = Note(offset)..Note(offset + length);
                            let output = key.notes_in_range(note_range.clone());
                            let out_of_key = output
                                .iter()
                                .filter(|n| !key_pitches.contains(&n.pitch_class()))
                                .collect::<Vec<_>>();
                            assert!(
                                out_of_key.is_empty(),
                                "`{:?}.notes_in_range({:?})` produced out of key notes.\nOutput: {:?}\nOut of key: {:?}",
                                key, note_range, output, out_of_key
                            )
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn chord_note_iter_smoke_test() {
        let roots = [PitchClass(0), PitchClass(7), PitchClass(11)];
        let chord_shapes = ChordShape::all();
        let range_lens = (0..=36).collect::<Vec<_>>();
        let range_offsets = (0..=12).collect::<Vec<_>>();

        for root in roots {
            for shape in chord_shapes.clone() {
                for range_len in range_lens.clone() {
                    for range_offset in range_offsets.clone() {
                        let first_range_note = Note(range_offset);
                        let note_range = first_range_note..(first_range_note + Interval(range_len));
                        let chord = Chord::from((root, shape));
                        let chord_pitches = chord.pitch_classes();
                        let chord_notes = chord.notes_in_range(note_range.clone());

                        if note_range
                            .clone()
                            .contains(&chord.root.at_or_above(&first_range_note))
                        {
                            assert!(
                                chord_notes.contains(&chord.root.at_or_above(&first_range_note)),
                                "{:?} could contain chord root of {:?}, but it doesn't.",
                                note_range,
                                chord
                            );
                        }

                        assert!(
                            chord_notes
                                .iter()
                                .all(|n| chord_pitches.contains(&n.pitch_class())),
                            "{:?}.notes_in_range({:?}) produced notes not within the chord.",
                            chord,
                            note_range
                        );

                        let note_count_lower_bound = range_len as usize
                            / (first_range_note.pitch_class().interval_to(&chord.root)
                                + *chord.intervals().last().unwrap()
                                + chord.intervals().last().unwrap().inversion())
                            .0 as usize
                            * chord.intervals().len();

                        assert!(chord_notes.len() >= note_count_lower_bound,
                            "{:?} has room for at least {:?} notes of {:?}, but only {:?} were produced.",
                            note_range, note_count_lower_bound, chord, chord_notes.len());

                        if note_count_lower_bound >= chord.intervals().len() {
                            assert!(
                                chord_pitches.iter().all(|p| {
                                    chord_notes.iter().any(|n| n.pitch_class() == *p)
                                }),
                                "{:?} has room for all notes of {:?}, but they weren't all produced.", note_range, chord);
                        }
                    }
                }
            }
        }
    }
}
