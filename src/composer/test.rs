use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::composer::context::CompositionContext;
use crate::composer::render::{AdhocRenderer, RenderEngine};
use crate::composer::{render::Tree, Composer, CompositionSegment, RenderSegment};
use crate::error::RendererError::MissingContext;

use super::CompositionElement;

#[test]
fn serialize() {
    let composer = Composer {
        engine: renderers(),
    };
    let tree = composer.compose_with_seed(CompositionSegment::new(SerdeTestComposition, 0..100), 0);
    let serialized_tree = serde_json::to_string(&tree).unwrap();

    assert_eq!(serialized_tree, "{\"element\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seeded_from\":{\"FixedSeed\":0},\"seed\":0,\"rendered\":true,\"children\":[{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seeded_from\":\"Random\",\"seed\":1287509791301768306,\"rendered\":true},{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seeded_from\":\"Random\",\"seed\":7056400819414448509,\"rendered\":true},{\"element\":{\"SerdeTestError\":null},\"start\":0,\"end\":4,\"seeded_from\":\"Random\",\"seed\":2005398531044258662,\"rendered\":false,\"error\":{\"MissingContext\":\"MissingType\"}}]}");
}

#[test]
fn deserialize() {
    let serialized = "{\"element\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seeded_from\":{\"FixedSeed\":0},\"seed\":0,\"rendered\":true,\"children\":[{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seeded_from\":\"Random\",\"seed\":1287509791301768306,\"rendered\":true},{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seeded_from\":\"Random\",\"seed\":7056400819414448509,\"rendered\":true}]}";
    let tree: Tree<RenderSegment> = serde_json::from_str(serialized).unwrap();

    // There should be three tree nodes
    assert_eq!(tree.len(), 3);

    // The root should be type SerdeTestComposition
    assert!((*tree.root().unwrap().value.segment.element)
        .as_any()
        .is::<SerdeTestComposition>());

    // The root's children should all be type SerdeTestComplexType
    assert!(tree.root().unwrap().children.iter().all(|child_idx| (*tree
        .get(*child_idx)
        .unwrap()
        .value
        .segment
        .element)
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
                    CompositionSegment::new(SerdeTestError, 0..4),
                ])
            },
        )
        + AdhocRenderer::from(
            |_: &SerdeTestError, _: &Range<i32>, _: &CompositionContext| {
                Err(MissingContext(String::from("MissingType")))
            },
        )
}

// Some test composition components used above
#[derive(Debug, Serialize, Deserialize)]
struct SerdeTestComposition;

#[typetag::serde]
impl CompositionElement for SerdeTestComposition {}

#[derive(Debug, Serialize, Deserialize)]
struct SerdeTestComplexType {
    some_data: String,
    more_data: i32,
}

#[typetag::serde]
impl CompositionElement for SerdeTestComplexType {}

#[derive(Debug, Serialize, Deserialize)]
struct SerdeTestError;

#[typetag::serde]
impl CompositionElement for SerdeTestError {}
