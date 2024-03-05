use crate::NoteName::C;
use crate::{Key, Mode, Note, NoteIterator, PitchClass, PitchClassCollection, Scale};

#[test]
fn middle_c_major_scale() {
    assert_eq!(
        Key {
            tonic: C.into(),
            scale: Scale::Major,
            mode: Mode::default()
        }
        .notes_in_range(Note(60)..=Note(72)),
        [
            Note(60),
            Note(62),
            Note(64),
            Note(65),
            Note(67),
            Note(69),
            Note(71),
            Note(72)
        ]
    )
}

#[test]
fn middle_c_minor_scale() {
    assert_eq!(
        Key {
            tonic: C.into(),
            scale: Scale::Minor,
            mode: Mode::default()
        }
        .notes_in_range(Note(60)..=Note(72)),
        [
            Note(60),
            Note(62),
            Note(63),
            Note(65),
            Note(67),
            Note(69),
            Note(70),
            Note(72)
        ]
    )
}

#[test]
fn middle_c_natural_minor_scale() {
    assert_eq!(
        Key {
            tonic: C.into(),
            scale: Scale::NaturalMinor,
            mode: Mode::default()
        }
        .notes_in_range(Note(60)..=Note(72)),
        [
            Note(60),
            Note(62),
            Note(63),
            Note(65),
            Note(67),
            Note(68),
            Note(70),
            Note(72)
        ]
    )
}

// region: Scale notes tests
#[test]
fn key_notes_boundary_test() {
    let tonics = vec![PitchClass(0), PitchClass(9), PitchClass(11)];
    let scales = Scale::values();
    let modes = vec![Mode::Ionian, Mode::Dorian, Mode::Aeolian, Mode::Locrian];
    let lengths = [0, 1, 11, 12, 13, 23];
    let offsets = [0, 1, 11, 12, 13, 23];

    // let mut seq = 0_usize;
    for tonic in tonics.clone() {
        for scale in scales.clone() {
            for mode in modes.clone() {
                for length in lengths {
                    for offset in offsets {
                        let key = Key { tonic, scale, mode };
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
// endregion: Scale notes tests

#[cfg(test)]
mod interval_tests {
    use crate::Interval as I;

    #[test]
    fn check_simple_interval() {
        assert!(I(0).is_simple());
        assert!(I(12).is_simple());
        assert!(!I(13).is_simple());
    }

    #[test]
    fn check_compound_interval() {
        assert!(I(13).is_compound());
        assert!(I(24).is_compound());
        assert!(!I(12).is_compound());
    }

    #[test]
    fn check_simple_inversions() {
        assert_eq!(I::P1.inversion(), I::P8);
        assert_eq!(I::P8.inversion(), I::P1);

        assert_eq!(I::m2.inversion(), I::M7);
        assert_eq!(I::M7.inversion(), I::m2);

        assert_eq!(I::M2.inversion(), I::m7);
        assert_eq!(I::m7.inversion(), I::M2);

        assert_eq!(I::m3.inversion(), I::M6);
        assert_eq!(I::M6.inversion(), I::m3);

        assert_eq!(I::M3.inversion(), I::m6);
        assert_eq!(I::m6.inversion(), I::M3);

        assert_eq!(I::P4.inversion(), I::P5);
        assert_eq!(I::P5.inversion(), I::P4);

        assert_eq!(I::A4.inversion(), I::A4);
    }

    #[test]
    fn check_compound_inversions() {
        assert_eq!(I::m9.inversion(), I::M7);
        assert_eq!(I::M9.inversion(), I::m7);
        assert_eq!(I::m10.inversion(), I::M6);
        assert_eq!(I::M10.inversion(), I::m6);
        assert_eq!(I::P11.inversion(), I::P5);
        assert_eq!(I::P12.inversion(), I::P4);
        assert_eq!(I::m13.inversion(), I::M3);
        assert_eq!(I::M13.inversion(), I::m3);
    }
}

#[cfg(test)]
mod tests {
    use crate::chord::ChordShape::maj;
    use crate::Chord;
    use crate::NoteName::*;
    use crate::{Note, NoteIterator};

    // #[test]
    // fn test_scale_note_scaling() {
    //     let test_range = 0..128;
    //
    //     let key = Key {
    //         tonic: C.into(),
    //         scale: Scale::Major,
    //         mode: Default::default(),
    //     };
    //
    //     let key_pitches = key.notes().raw().into_iter().map(|n| n % 12).collect::<Vec<_>>();
    //     for subrange in test_range.step_by(7) {
    //         assert!(
    //             key.notes().in_range(subrange..(subrange+7)).into_iter()
    //                 .all(|n| key_pitches.contains(&(n % 12)))
    //         )
    //     }
    // }

    #[test]
    fn test_chord_scaling() {
        let chord = Chord::from((F, maj));

        assert_eq!(
            chord.notes_in_range(Note::from((C, 3))..Note::from((C, 4))),
            vec![Note::from((C, 3)), Note::from((F, 3)), Note::from((A, 3))]
        )
    }
}
