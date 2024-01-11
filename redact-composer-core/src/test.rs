use serde::{Deserialize, Serialize};

use crate::derive::Element;
use crate::error::RendererError::MissingContext;
use crate::render::{AdhocRenderer, RenderEngine};
use crate::{Composer, Composition, Segment};

#[test]
fn serialize() {
    let composer = Composer::from(renderers());
    let comp = composer.compose_with_seed(Segment::new(SerdeTestComposition, 0..100), 0);
    let serialized_comp = serde_json::to_string(&comp).unwrap();

    assert_eq!(serialized_comp, "{\"options\":{\"ticks_per_beat\":480},\"tree\":{\"element\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seed\":0,\"rendered\":true,\"children\":[{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seed\":1287509791301768306,\"rendered\":true},{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seed\":7056400819414448509,\"rendered\":true},{\"element\":{\"SerdeTestError\":null},\"start\":0,\"end\":4,\"seed\":2005398531044258662,\"rendered\":false,\"error\":{\"MissingContext\":\"MissingType\"}}]}}");
}

#[test]
fn deserialize() {
    let serialized = "{\"options\":{\"ticks_per_beat\":480},\"tree\":{\"element\":{\"SerdeTestComposition\":null},\"start\":0,\"end\":100,\"seeded_from\":{\"FixedSeed\":0},\"seed\":0,\"rendered\":true,\"children\":[{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test1\",\"more_data\":1}},\"start\":0,\"end\":2,\"seed\":1287509791301768306,\"rendered\":true},{\"element\":{\"SerdeTestComplexType\":{\"some_data\":\"test2\",\"more_data\":2}},\"start\":2,\"end\":4,\"seed\":7056400819414448509,\"rendered\":true}]}}";
    let comp: Composition = serde_json::from_str(serialized).unwrap();

    // There should be three tree nodes
    assert_eq!(comp.tree.len(), 3);

    // The root should be type SerdeTestComposition
    assert!((*comp.tree.root().unwrap().value.segment.element)
        .as_any()
        .is::<SerdeTestComposition>());

    // The root's children should all be type SerdeTestComplexType
    assert!(comp
        .tree
        .root()
        .unwrap()
        .children
        .iter()
        .all(
            |child_idx| (*comp.tree.get(*child_idx).unwrap().value.segment.element)
                .as_any()
                .is::<SerdeTestComplexType>()
        ))
}

#[test]
fn serde_equivalence() {
    let composer = Composer::from(renderers());
    let comp = composer.compose_with_seed(Segment::new(SerdeTestComposition, 0..100), 0);
    let serialized_comp = serde_json::to_string(&comp).unwrap();

    let deserialized_comp: Composition = serde_json::from_str(serialized_comp.as_str()).unwrap();

    assert_eq!(
        serialized_comp,
        serde_json::to_string(&deserialized_comp).unwrap()
    )
}

fn renderers() -> RenderEngine {
    RenderEngine::new()
        + AdhocRenderer::<SerdeTestComposition>::new(|_, _| {
            Ok(vec![
                Segment::new(
                    SerdeTestComplexType {
                        some_data: String::from("test1"),
                        more_data: 1,
                    },
                    0..2,
                ),
                Segment::new(
                    SerdeTestComplexType {
                        some_data: String::from("test2"),
                        more_data: 2,
                    },
                    2..4,
                ),
                Segment::new(SerdeTestError, 0..4),
            ])
        })
        + AdhocRenderer::<SerdeTestError>::new(|_, _| {
            Err(MissingContext(String::from("MissingType")))
        })
}

// Some test composition components used above
#[derive(Element, Serialize, Deserialize, Debug)]
struct SerdeTestComposition;

#[derive(Element, Serialize, Deserialize, Debug)]
struct SerdeTestComplexType {
    some_data: String,
    more_data: i32,
}

#[derive(Element, Serialize, Deserialize, Debug)]
struct SerdeTestError;
