use serde::{Deserialize, Serialize};

use crate::derive::Element;
use crate::error::RendererError::MissingContext;
use crate::render::context::TimingRelation::Overlapping;
use crate::render::{AdhocRenderer, RenderEngine};
use crate::IntoSegment;
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

#[test]
fn depth_first_render_order() {
    // Create elements/renderers to form the following tree:
    //            RORoot
    //        /     |      \
    //   RONode1  RONode2  RONode3
    //      |       |        |
    //   RONode4  RONode5  RONode6
    //      |       |        |
    //   RONode7  RONode8  RONode9
    //
    // With these context dependencies (-> = "depends on"):
    // RONode1 -> RONode5
    // RONode5 -> RONode3
    //
    // This should result in the following sequence of nodes being added the tree:
    // [RORoot, RONode1, RONode2, RONode3, RONode5, RONode6, RONode9, RONode8, RONode4, RONode7]
    //
    // If this is confusing, remember the following:
    // * Nodes are added to the tree when their parent renders, but they are initially undrendered.
    // * Nodes are considered rendered only after their dependencies are satisfied and they produce children
    //   (unless they are leaf nodes).
    // * A context dependency is not satisfied until its target is in the tree *and* rendered.
    // * When a node gets its turn to render, it does it in depth (to the extent context dependencies allow)
    //   even if during the process it satisfies a dependency which was blocking a previous node render.
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RORoot;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode1;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode2;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode3;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode4;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode5;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode6;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode7;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode8;
    #[derive(Element, Serialize, Deserialize, Debug)]
    struct RONode9;

    let engine = RenderEngine::new()
        + AdhocRenderer::<RORoot>::new(|seg, _| {
            Ok(vec![
                RONode1.over(seg),
                RONode2.over(seg),
                RONode3.over(seg),
            ])
        })
        + AdhocRenderer::<RONode1>::new(|seg, ctx| {
            ctx.find::<RONode5>()
                .with_timing(Overlapping, seg)
                .require()?;
            Ok(vec![RONode4.over(seg)])
        })
        + AdhocRenderer::<RONode2>::new(|seg, _| Ok(vec![RONode5.over(seg)]))
        + AdhocRenderer::<RONode3>::new(|seg, _| Ok(vec![RONode6.over(seg)]))
        + AdhocRenderer::<RONode4>::new(|seg, _| Ok(vec![RONode7.over(seg)]))
        + AdhocRenderer::<RONode5>::new(|seg, ctx| {
            ctx.find::<RONode3>()
                .with_timing(Overlapping, seg)
                .require()?;
            Ok(vec![RONode8.over(seg)])
        })
        + AdhocRenderer::<RONode6>::new(|seg, _| Ok(vec![RONode9.over(seg)]));

    let composer = Composer::from(engine);
    let comp = composer.compose_with_seed(Segment::new(RORoot, 0..10), 0);

    assert!(comp.tree[0].value.segment.element_as::<RORoot>().is_some());
    assert!(comp.tree[1].value.segment.element_as::<RONode1>().is_some());
    assert!(comp.tree[2].value.segment.element_as::<RONode2>().is_some());
    assert!(comp.tree[3].value.segment.element_as::<RONode3>().is_some());
    assert!(comp.tree[4].value.segment.element_as::<RONode5>().is_some());
    assert!(comp.tree[5].value.segment.element_as::<RONode6>().is_some());
    assert!(comp.tree[6].value.segment.element_as::<RONode9>().is_some());
    assert!(comp.tree[7].value.segment.element_as::<RONode8>().is_some());
    assert!(comp.tree[8].value.segment.element_as::<RONode4>().is_some());
    assert!(comp.tree[9].value.segment.element_as::<RONode7>().is_some());
}
