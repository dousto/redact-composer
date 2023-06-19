use super::{Rhythm, Subdivision};

#[test]
fn over_should_repeat_for_longer_range() {
    assert_eq!(
        Rhythm(vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
            Subdivision {
                timing: 3..4,
                is_rest: false
            },
        ])
        .over(0..9),
        vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
            Subdivision {
                timing: 3..4,
                is_rest: false
            },
            Subdivision {
                timing: 4..5,
                is_rest: false
            },
            Subdivision {
                timing: 5..7,
                is_rest: false
            },
            Subdivision {
                timing: 7..8,
                is_rest: false
            },
            Subdivision {
                timing: 8..9,
                is_rest: false
            },
        ]
    )
}

#[test]
fn should_trim_over_shorter_range() {
    assert_eq!(
        Rhythm(vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
            Subdivision {
                timing: 3..4,
                is_rest: false
            },
        ])
        .over(0..3),
        vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
        ]
    )
}

#[test]
fn should_return_same_over_same_range() {
    assert_eq!(
        Rhythm(vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
            Subdivision {
                timing: 3..4,
                is_rest: false
            },
        ])
        .over(0..4),
        vec![
            Subdivision {
                timing: 0..1,
                is_rest: false
            },
            Subdivision {
                timing: 1..3,
                is_rest: false
            },
            Subdivision {
                timing: 3..4,
                is_rest: false
            },
        ]
    )
}
