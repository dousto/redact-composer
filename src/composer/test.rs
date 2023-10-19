use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::composer::context::CompositionContext;
use crate::composer::render::{AdhocRenderer, RenderEngine};
use crate::composer::{render::Tree, Composer, CompositionSegment, RenderSegment};

use super::SegmentType;

#[test]
fn serialize() {
    let composer = Composer {
        engine: renderers(),
    };
    let tree = composer.compose_with_seed(CompositionSegment::new(SerdeTestComposition, 0..100), 0);
    let serialized_tree = serde_json::to_string(&tree).unwrap();

    assert_eq!(serialized_tree, "{\"type\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seeded_from\":{\"FixedSeed\":0},\"seed\":0,\"rendered\":true,\"children\":[{\"type\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seeded_from\":\"Random\",\"seed\":1287509791301768306,\"rendered\":true},{\"type\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seeded_from\":\"Random\",\"seed\":7056400819414448509,\"rendered\":true}]}");
}

#[test]
fn deserialize() {
    let serialized = "{\"type\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seeded_from\":{\"FixedSeed\":0},\"seed\":0,\"rendered\":true,\"children\":[{\"type\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seeded_from\":\"Random\",\"seed\":1287509791301768306,\"rendered\":true},{\"type\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seeded_from\":\"Random\",\"seed\":7056400819414448509,\"rendered\":true}]}";
    let tree: Tree<RenderSegment> = serde_json::from_str(serialized).unwrap();

    // There should be three tree nodes
    assert_eq!(tree.len(), 3);

    // The root should be type SerdeTestComposition
    assert!((*tree.root().unwrap().value.segment.segment_type)
        .as_any()
        .is::<SerdeTestComposition>());

    // The root's children should all be type SerdeTestComplexType
    assert!(tree.root().unwrap().children.iter().all(|child_idx| (*tree
        .get(*child_idx)
        .unwrap()
        .value
        .segment
        .segment_type)
        .as_any()
        .is::<SerdeTestComplexType>()))
}

#[test]
fn serde_equivalence() {
    let composer = Composer {
        engine: renderers(),
    };
    let tree = composer.compose_with_seed(CompositionSegment::new(SerdeTestComposition, 0..100), 0);
    let serialized_tree = serde_json::to_string(&tree).unwrap();

    let deserialized_tree: Tree<RenderSegment> =
        serde_json::from_str(serialized_tree.as_str()).unwrap();

    assert_eq!(
        serialized_tree,
        serde_json::to_string(&deserialized_tree).unwrap()
    )
}

fn renderers() -> RenderEngine {
    RenderEngine::new()
        + AdhocRenderer::from(
            |_: &SerdeTestComposition, _: &Range<i32>, _: &CompositionContext| {
                Ok(vec![
                    CompositionSegment::new(
                        SerdeTestComplexType {
                            some_data: String::from("test1"),
                            more_data: 1,
                        },
                        0..2,
                    ),
                    CompositionSegment::new(
                        SerdeTestComplexType {
                            some_data: String::from("test2"),
                            more_data: 2,
                        },
                        2..4,
                    ),
                ])
            },
        )
}

// Some test composition components used above
#[derive(Debug, Serialize, Deserialize)]
struct SerdeTestComposition;

#[typetag::serde]
impl SegmentType for SerdeTestComposition {}

#[derive(Debug, Serialize, Deserialize)]
struct SerdeTestComplexType {
    some_data: String,
    more_data: i32,
}

#[typetag::serde]
impl SegmentType for SerdeTestComplexType {}
