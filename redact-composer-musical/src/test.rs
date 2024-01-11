use crate::elements::{Chord, Key, Mode, Scale};

use super::Notes;

#[test]
fn note_ranges() {
    assert_eq!(
        Notes::from(vec![0, 1, 2, 9, 10, 11]).in_range(24..=38),
        [24, 25, 26, 33, 34, 35, 36, 37, 38]
    )
}

#[test]
fn note_range_works_with_notes_outside_1_to_12_range() {
    let notes = Notes::from(vec![0, 1, 2, 9, 10, 11]);
    assert_eq!(
        Notes::from(notes.in_range(24..=38)).in_range(24..=38),
        [24, 25, 26, 33, 34, 35, 36, 37, 38]
    );
}

#[test]
fn middle_c_major_scale() {
    assert_eq!(
        Notes::from(
            Key {
                tonic: 0,
                scale: Scale::Major,
                mode: Mode::Ionian
            }
            .scale()
        )
        .in_range(60..=72),
        [60, 62, 64, 65, 67, 69, 71, 72]
    )
}

#[test]
fn middle_c_minor_scale() {
    assert_eq!(
        Notes::from(
            Key {
                tonic: 0,
                scale: Scale::Minor,
                mode: Mode::Ionian
            }
            .scale()
        )
        .in_range(60..=72),
        [60, 62, 63, 65, 67, 69, 70, 72]
    )
}

#[test]
fn middle_c_natural_minor_scale() {
    assert_eq!(
        Notes::from(
            Key {
                tonic: 0,
                scale: Scale::NaturalMinor,
                mode: Mode::Ionian
            }
            .scale()
        )
        .in_range(60..=72),
        [60, 62, 63, 65, 67, 68, 70, 72]
    )
}

#[test]
fn chord_i_degrees() {
    assert_eq!(Chord::I.degrees(), [0, 2, 4])
}

#[test]
fn chord_ii_degrees() {
    assert_eq!(Chord::II.degrees(), [1, 3, 5])
}

#[test]
fn chord_iii_degrees() {
    assert_eq!(Chord::III.degrees(), [2, 4, 6])
}

#[test]
fn chord_iv_degrees() {
    assert_eq!(Chord::IV.degrees(), [3, 5, 0])
}

#[test]
fn chord_v_degrees() {
    assert_eq!(Chord::V.degrees(), [4, 6, 1])
}

#[test]
fn chord_vi_degrees() {
    assert_eq!(Chord::VI.degrees(), [5, 0, 2])
}

#[test]
fn chord_vii_degrees() {
    assert_eq!(Chord::VII.degrees(), [6, 1, 3])
}

// region: Chord to/from String conversion tests

#[test]
fn chord_i_to_string() {
    assert_eq!(Chord::I.to_string(), Chord::I_STR)
}

#[test]
fn chord_i_from_string() {
    assert_eq!(Chord::from(Chord::I_STR), Chord::I)
}

#[test]
fn chord_ii_to_string() {
    assert_eq!(Chord::II.to_string(), Chord::II_STR)
}

#[test]
fn chord_ii_from_string() {
    assert_eq!(Chord::from(Chord::II_STR), Chord::II)
}

#[test]
fn chord_iii_to_string() {
    assert_eq!(Chord::III.to_string(), Chord::III_STR)
}

#[test]
fn chord_iii_from_string() {
    assert_eq!(Chord::from(Chord::III_STR), Chord::III)
}

#[test]
fn chord_iv_to_string() {
    assert_eq!(Chord::IV.to_string(), Chord::IV_STR)
}

#[test]
fn chord_iv_from_string() {
    assert_eq!(Chord::from(Chord::IV_STR), Chord::IV)
}

#[test]
fn chord_v_to_string() {
    assert_eq!(Chord::V.to_string(), Chord::V_STR)
}

#[test]
fn chord_v_from_string() {
    assert_eq!(Chord::from(Chord::V_STR), Chord::V)
}

#[test]
fn chord_vi_to_string() {
    assert_eq!(Chord::VI.to_string(), Chord::VI_STR)
}

#[test]
fn chord_vi_from_string() {
    assert_eq!(Chord::from(Chord::VI_STR), Chord::VI)
}

#[test]
fn chord_vii_to_string() {
    assert_eq!(Chord::VII.to_string(), Chord::VII_STR)
}

#[test]
fn chord_vii_from_string() {
    assert_eq!(Chord::from(Chord::VII_STR), Chord::VII)
}

// endregion: Chord to/from String conversion tests

// region: Scale to/from String conversion tests
#[test]
fn major_to_string() {
    assert_eq!(Scale::Major.to_string(), Scale::MAJOR_STR)
}

#[test]
fn major_from_string() {
    assert_eq!(Scale::from(Scale::MAJOR_STR), Scale::Major)
}

#[test]
fn minor_to_string() {
    assert_eq!(Scale::Minor.to_string(), Scale::MINOR_STR)
}

#[test]
fn minor_from_string() {
    assert_eq!(Scale::from(Scale::MINOR_STR), Scale::Minor)
}

#[test]
fn natural_minor_to_string() {
    assert_eq!(Scale::NaturalMinor.to_string(), Scale::NATURAL_MINOR_STR)
}

#[test]
fn natural_minor_from_string() {
    assert_eq!(Scale::from(Scale::NATURAL_MINOR_STR), Scale::NaturalMinor)
}

#[test]
fn harmonic_minor_to_string() {
    assert_eq!(Scale::HarmonicMinor.to_string(), Scale::HARMONIC_MINOR_STR)
}

#[test]
fn harmonic_minor_from_string() {
    assert_eq!(Scale::from(Scale::HARMONIC_MINOR_STR), Scale::HarmonicMinor)
}

// endregion String conversion tests
