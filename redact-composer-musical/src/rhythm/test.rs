use super::{Rhythm, Subdivision};

#[test]
fn over_should_repeat_for_longer_range() {
    assert_eq!(
        Rhythm(vec![
            Subdivision {
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
                is_rest: false
            },
            Subdivision {
                start: 3,
                end: 4,
                is_rest: false
            },
        ])
        .over(0..9),
        vec![
            Subdivision {
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
                is_rest: false
            },
            Subdivision {
                start: 3,
                end: 4,
                is_rest: false
            },
            Subdivision {
                start: 4,
                end: 5,
                is_rest: false
            },
            Subdivision {
                start: 5,
                end: 7,
                is_rest: false
            },
            Subdivision {
                start: 7,
                end: 8,
                is_rest: false
            },
            Subdivision {
                start: 8,
                end: 9,
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
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
                is_rest: false
            },
            Subdivision {
                start: 3,
                end: 4,
                is_rest: false
            },
        ])
        .over(0..3),
        vec![
            Subdivision {
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
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
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
                is_rest: false
            },
            Subdivision {
                start: 3,
                end: 4,
                is_rest: false
            },
        ])
        .over(0..4),
        vec![
            Subdivision {
                start: 0,
                end: 1,
                is_rest: false
            },
            Subdivision {
                start: 1,
                end: 3,
                is_rest: false
            },
            Subdivision {
                start: 3,
                end: 4,
                is_rest: false
            },
        ]
    )
}
