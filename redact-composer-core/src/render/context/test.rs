use crate::render::context::TimingRelation::{
    Before, BeginningWithin, During, EndingWithin, Overlapping, Within,
};
use std::ops::Bound::{Excluded, Included, Unbounded};

use super::TimingConstraint;

#[test]
fn during() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), true),
        (
            ((Unbounded, Included(0)), (Included(-10), Unbounded)),
            false,
        ),
        (((Unbounded, Included(0)), (Unbounded, Excluded(0))), false),
        (((Unbounded, Included(0)), (Unbounded, Included(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Excluded(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Included(0))), true),
        (((Included(0), Unbounded), (Unbounded, Included(10))), false),
        (((Included(0), Unbounded), (Included(0), Unbounded)), true),
        (((Included(0), Unbounded), (Excluded(0), Unbounded)), false),
        (((Excluded(0), Unbounded), (Included(0), Unbounded)), true),
        (((Excluded(0), Unbounded), (Excluded(0), Unbounded)), true),
        (((Included(0), Included(10)), (Unbounded, Unbounded)), true),
        (
            ((Included(0), Included(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Included(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Excluded(10))),
            false,
        ),
        (((Excluded(0), Included(10)), (Unbounded, Unbounded)), true),
        (
            ((Excluded(0), Included(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Excluded(10))),
            false,
        ),
        (((Included(0), Excluded(10)), (Unbounded, Unbounded)), true),
        (
            ((Included(0), Excluded(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Excluded(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Excluded(10))),
            true,
        ),
        (((Excluded(0), Excluded(10)), (Unbounded, Unbounded)), true),
        (
            ((Excluded(0), Excluded(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Excluded(10))),
            true,
        ),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: During,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::during({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}

#[test]
fn within() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), true),
        (
            ((Unbounded, Included(0)), (Included(-10), Unbounded)),
            false,
        ),
        (((Unbounded, Included(0)), (Unbounded, Excluded(0))), true),
        (((Unbounded, Included(0)), (Unbounded, Included(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Excluded(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Included(0))), false),
        (((Included(0), Unbounded), (Unbounded, Included(10))), false),
        (((Included(0), Unbounded), (Included(0), Unbounded)), true),
        (((Included(0), Unbounded), (Excluded(0), Unbounded)), true),
        (((Excluded(0), Unbounded), (Included(0), Unbounded)), false),
        (((Excluded(0), Unbounded), (Excluded(0), Unbounded)), true),
        (((Included(0), Included(10)), (Unbounded, Unbounded)), false),
        (
            ((Included(0), Included(10)), (Included(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Included(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Included(10))),
            false,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Excluded(10))),
            false,
        ),
        (((Excluded(0), Included(10)), (Unbounded, Unbounded)), false),
        (
            ((Excluded(0), Included(10)), (Included(0), Unbounded)),
            false,
        ),
        (
            ((Excluded(0), Included(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Included(10))),
            false,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Excluded(10))),
            false,
        ),
        (((Included(0), Excluded(10)), (Unbounded, Unbounded)), false),
        (
            ((Included(0), Excluded(10)), (Included(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Excluded(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Included(10))),
            false,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Excluded(10))),
            false,
        ),
        (((Excluded(0), Excluded(10)), (Unbounded, Unbounded)), false),
        (
            ((Excluded(0), Excluded(10)), (Included(0), Unbounded)),
            false,
        ),
        (
            ((Excluded(0), Excluded(10)), (Excluded(0), Unbounded)),
            false,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Included(10))),
            false,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Excluded(10))),
            false,
        ),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: Within,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::within({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}

#[test]
fn beginning_within() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Excluded(10), Unbounded)), true),
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), true),
        (((Included(1), Unbounded), (Included(1), Unbounded)), true),
        (((Included(1), Unbounded), (Included(0), Unbounded)), false),
        (((Unbounded, Included(1)), (Included(1), Unbounded)), true),
        (((Unbounded, Included(1)), (Included(2), Unbounded)), false),
        (
            ((Included(1), Excluded(10)), (Excluded(8), Included(10))),
            true,
        ),
        (
            ((Included(1), Excluded(10)), (Excluded(9), Included(10))),
            true,
        ),
        (
            ((Included(1), Excluded(10)), (Excluded(2), Included(10))),
            true,
        ),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: BeginningWithin,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::beginning_within({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}

#[test]
fn ending_within() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Unbounded, Excluded(10))), true),
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), true),
        (((Included(1), Unbounded), (Unbounded, Included(1))), true),
        (((Included(1), Unbounded), (Unbounded, Included(0))), false),
        (((Unbounded, Included(1)), (Unbounded, Included(1))), true),
        (((Unbounded, Included(1)), (Unbounded, Included(2))), false),
        (
            ((Included(1), Excluded(10)), (Included(0), Excluded(10))),
            true,
        ),
        (
            ((Included(1), Excluded(10)), (Included(0), Excluded(11))),
            false,
        ),
        (
            ((Included(1), Excluded(10)), (Included(0), Excluded(2))),
            true,
        ),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: EndingWithin,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::ending_within({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}

#[test]
fn overlapping() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), true),
        (((Unbounded, Included(0)), (Included(-10), Unbounded)), true),
        (((Unbounded, Included(0)), (Included(0), Unbounded)), true),
        (((Unbounded, Included(0)), (Excluded(0), Unbounded)), false),
        (((Unbounded, Included(0)), (Unbounded, Excluded(0))), true),
        (((Unbounded, Included(0)), (Unbounded, Included(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Excluded(0))), true),
        (((Unbounded, Excluded(0)), (Unbounded, Included(0))), true),
        (((Included(0), Unbounded), (Unbounded, Included(10))), true),
        (((Included(0), Unbounded), (Included(0), Unbounded)), true),
        (((Included(0), Unbounded), (Excluded(0), Unbounded)), true),
        (((Excluded(0), Unbounded), (Included(0), Unbounded)), true),
        (((Excluded(0), Unbounded), (Excluded(0), Unbounded)), true),
        (((Included(0), Included(10)), (Unbounded, Unbounded)), true),
        (
            ((Included(0), Included(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Included(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Included(0), Included(10)), (Unbounded, Excluded(10))),
            true,
        ),
        (((Excluded(0), Included(10)), (Unbounded, Unbounded)), true),
        (
            ((Excluded(0), Included(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Excluded(0), Included(10)), (Unbounded, Excluded(10))),
            true,
        ),
        (((Included(0), Excluded(10)), (Unbounded, Unbounded)), true),
        (
            ((Included(0), Excluded(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Excluded(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Included(0), Excluded(10)), (Unbounded, Excluded(10))),
            true,
        ),
        (((Excluded(0), Excluded(10)), (Unbounded, Unbounded)), true),
        (
            ((Excluded(0), Excluded(10)), (Included(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Excluded(0), Unbounded)),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Included(10))),
            true,
        ),
        (
            ((Excluded(0), Excluded(10)), (Unbounded, Excluded(10))),
            true,
        ),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: Overlapping,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::overlapping({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}

#[test]
fn before() {
    let test_cases = vec![
        (((Unbounded, Unbounded), (Unbounded, Unbounded)), false),
        (((Included(0), Unbounded), (Unbounded, Unbounded)), false),
        (((Included(0), Unbounded), (Unbounded, Included(0))), false),
        (((Included(0), Unbounded), (Unbounded, Included(-1))), true),
        (((Included(0), Unbounded), (Unbounded, Excluded(0))), true),
        (((Included(0), Unbounded), (Unbounded, Excluded(1))), false),
    ];

    for ((ref_range, target_range), expectation) in test_cases {
        let result = TimingConstraint {
            relation: Before,
            ref_range,
        }
        .matches(&target_range);
        assert!(
            result == expectation,
            "TimeRelation::before({:?}).matches({:?}) was {:?}, expected {:?}",
            ref_range,
            target_range,
            result,
            expectation
        )
    }
}
