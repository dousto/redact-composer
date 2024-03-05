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

            let (first_note, interval_idx_offset) =
                if first_range_note.pitch_class() == start_pitch_class {
                    (Some(first_range_note), 0)
                } else {
                    let mut start_interval_idx = steps.len() - 1;
                    let mut try_note = (start_pitch_class - steps[start_interval_idx])
                        .at_or_above(&first_range_note);
                    while !note_range.contains(&try_note) && start_interval_idx > 0 {
                        start_interval_idx -= 1;
                        try_note = (try_note.pitch_class() - steps[start_interval_idx])
                            .at_or_above(&first_range_note);
                    }

                    if note_range.contains(&try_note) {
                        (Some(try_note), start_interval_idx)
                    } else {
                        (None, start_interval_idx)
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
