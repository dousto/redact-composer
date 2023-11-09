use std::any::Any;
use std::borrow::Borrow;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::successors;
use std::ops::{Bound, Range, RangeBounds};

use rand::{thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash64;

use crate::composer::context::CompositionContext;
use crate::error::{ConversionError, RendererError};

use self::render::{RenderEngine, Tree};

use crate::musical;

#[cfg(test)]
mod test;

pub mod context;
pub mod render;

pub fn renderers() -> RenderEngine {
    RenderEngine::new() + musical::renderers()
}

/// A marker trait for any object that will be used as a composition element.
/// To make a custom type for use as a composition element, simply do:
/// ```
/// # use redact_composer::composer::SegmentType;
/// # use serde::{Deserialize, Serialize};
/// #[derive(Debug, Serialize, Deserialize)]
/// pub struct CustomCompositionElement {
///     // Struct fields
/// }
///
/// // This macro is needed to be able to serialize/deserialize custom types
/// #[typetag::serde]
/// impl SegmentType for CustomCompositionElement {}
/// ```
///
/// See [`render::Renderer`] for details on implementing renderers for a custom type.
#[typetag::serde]
pub trait SegmentType: Debug + AsAny + 'static {
    /// Implemented if this is a 'passthrough' type which does not itself render, but holds a
    /// reference to a type that does.
    /// Mainly used to provide a common 'tag' type for an unknown set of other types, enabling
    /// context lookups or other operations that depend on type.
    fn wrapped_type(&self) -> Option<&dyn SegmentType> {
        None
    }
}

fn unwrap_segment_type(segment_type: &dyn SegmentType) -> &dyn SegmentType {
    successors(Some(segment_type), |&s| s.wrapped_type())
        .last()
        .unwrap()
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: SegmentType> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum SeedType {
    Random,
    FixedSeed(u64),
}

#[derive(Debug)]
pub struct TypedSegment<'a, T: SegmentType> {
    pub value: &'a T,
    pub time_range: Range<i32>,
}

impl<'a, T> TryFrom<&'a CompositionSegment> for TypedSegment<'a, T>
where
    T: SegmentType,
{
    type Error = ConversionError;

    fn try_from(value: &'a CompositionSegment) -> std::result::Result<Self, Self::Error> {
        if let Some(converted) = value.segment_type_as::<T>() {
            Ok(TypedSegment {
                value: converted,
                time_range: value.time_range.clone(),
            })
        } else {
            return Err(ConversionError::TypeMismatch);
        }
    }
}

/// Simple struct to represent a given [`SegmentType`] which spans over a time range (`start..end`).
#[derive(Debug, Serialize, Deserialize)]
pub struct CompositionSegment {
    #[serde(rename = "type")]
    pub segment_type: Box<dyn SegmentType>,
    #[serde(flatten)]
    pub time_range: Range<i32>,
    seeded_from: SeedType,
}

impl CompositionSegment {
    pub fn new<T: Borrow<Range<i32>>>(
        composition_type: impl SegmentType,
        time_range: T,
    ) -> CompositionSegment {
        CompositionSegment {
            segment_type: Box::new(composition_type),
            time_range: time_range.borrow().clone(),
            seeded_from: SeedType::Random,
        }
    }

    pub fn named<T: Borrow<Range<i32>>>(
        hashable_name: impl Hash,
        composition_type: impl SegmentType,
        time_range: T,
    ) -> CompositionSegment {
        let mut hasher = XxHash64::with_seed(0);
        hashable_name.hash(&mut hasher);

        CompositionSegment {
            segment_type: Box::new(composition_type),
            time_range: time_range.borrow().clone(),
            seeded_from: SeedType::FixedSeed(hasher.finish()),
        }
    }

    /// Due to the type-erased nature of a [`CompositionSegment`]'s [`SegmentType`], this method
    /// allows access downcasting to the specific type. Returns [`Option<&T>`], containing the
    /// reference to the [`SegmentType`] or [`None`] if the type does not match.
    pub fn segment_type_as<T: SegmentType>(&self) -> Option<&T> {
        successors(Some(&*self.segment_type), |s| s.wrapped_type())
            .find_map(|s| s.as_any().downcast_ref::<T>())
    }
}

impl RangeBounds<i32> for CompositionSegment {
    fn start_bound(&self) -> Bound<&i32> {
        self.time_range.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.time_range.end_bound()
    }
}

impl RangeBounds<i32> for &CompositionSegment {
    fn start_bound(&self) -> Bound<&i32> {
        self.time_range.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.time_range.end_bound()
    }
}

impl<'a, T: SegmentType> RangeBounds<i32> for TypedSegment<'a, T> {
    fn start_bound(&self) -> Bound<&i32> {
        self.time_range.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.time_range.end_bound()
    }
}

/// As a convention, this [`SegmentType`] indicates the root of a composition tree.
#[derive(Debug, Serialize, Deserialize)]
pub struct Composition;

#[typetag::serde]
impl SegmentType for Composition {}

/// Note: Use [`redact_composer::musical::midi::Instrument`] for the time being.
#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub program: u8,
}

#[typetag::serde]
impl SegmentType for Instrument {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayNote {
    pub note: u8,
    pub velocity: u8,
}

#[typetag::serde]
impl SegmentType for PlayNote {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PartType {
    Instrument,
    Percussion,
}

/// Part is a special signifier to group notes that are to be played by a single instrument at a time.
#[derive(Debug, Serialize, Deserialize)]
pub struct Part(pub Box<dyn SegmentType>, pub PartType);

/// This is a simple pass-through implementation to the wrapped [`SegmentType`].
#[typetag::serde]
impl SegmentType for Part {
    fn wrapped_type(&self) -> Option<&dyn SegmentType> {
        Some(&*self.0)
    }
}

impl Part {
    pub fn instrument(wrapped_type: impl SegmentType) -> Part {
        Part(Box::new(wrapped_type), PartType::Instrument)
    }

    pub fn percussion(wrapped_type: impl SegmentType) -> Part {
        Part(Box::new(wrapped_type), PartType::Percussion)
    }
}

// Composer stuff

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderSegment {
    #[serde(flatten)]
    pub segment: CompositionSegment,
    pub seed: u64,
    pub rendered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RendererError>,
}

pub struct Composer {
    pub engine: RenderEngine,
}

impl Composer {
    /// Generates a render tree ([`Vec<Node<RenderSegment>>`]) from a starting [CompositionSegment].
    pub fn compose(&self, seg: CompositionSegment) -> Tree<RenderSegment> {
        let mut hasher = XxHash64::with_seed(0);
        thread_rng().next_u64().hash(&mut hasher);
        self.compose_with_seed(seg, hasher.finish())
    }

    pub fn compose_with_seed(&self, mut seg: CompositionSegment, seed: u64) -> Tree<RenderSegment> {
        seg.seeded_from = SeedType::FixedSeed(seed);
        let mut render_tree = Tree::new();
        render_tree.insert(
            RenderSegment {
                rendered: false,
                seed,
                segment: seg,
                error: None,
            },
            None,
        );

        let mut render_pass = 1;

        let mut rendered_node_count: usize;
        loop {
            // The high level loop flow is as follows:
            // 1. Search the tree (render_nodes) for all unrendered CompositionSegments nodes
            // 2. For each unrendered `RenderSegment` node, call its renderer which produces child CompositionSegments..
            // 3. Add to the composition tree based on the RenderResult outputs (as previously mentioned)
            //       Note: New nodes are always inserted as unrendered.
            // 4. Repeat until a state is reached where no additional nodes are rendered.
            rendered_node_count = 0;

            let unrendered: Vec<usize> = render_tree
                .iter()
                .filter(|n| !n.value.rendered)
                .map(|n| n.idx)
                .collect();

            for idx in unrendered {
                if let Some(_) = render_tree[idx].value.segment.segment_type_as::<Part>() {
                    let mut parent = &render_tree[idx].parent;
                    while let Some(pidx) = parent {
                        if let Some(_) = render_tree[*pidx].value.segment.segment_type_as::<Part>()
                        {
                            panic!("{}", "Part is not allowed to be nested.");
                        }
                        parent = &render_tree[*pidx].parent
                    }
                }

                let composition_context = CompositionContext::new(&render_tree, &render_tree[idx]);

                let result = self.engine.render(
                    unwrap_segment_type(&*render_tree[idx].value.segment.segment_type),
                    &render_tree[idx].value.segment.time_range,
                    &composition_context,
                );

                let mut hasher = XxHash64::with_seed(0);
                render_tree[idx].value.seed.hash(&mut hasher);
                let mut rng = ChaCha12Rng::seed_from_u64(hasher.finish());

                if let Some(render_res) = result {
                    match render_res {
                        render::Result::Err(err) => {
                            render_tree[idx].value.error = Some(err);
                        }
                        render::Result::Ok(segments) => {
                            let inserts: Vec<RenderSegment> = segments
                                .into_iter()
                                .map(|s| RenderSegment {
                                    rendered: !self
                                        .engine
                                        .can_render(unwrap_segment_type(&*s.segment_type)),
                                    seed: match s.seeded_from {
                                        SeedType::Random => {
                                            let mut hasher = XxHash64::with_seed(0);
                                            rng.next_u64().hash(&mut hasher);
                                            hasher.finish()
                                        }
                                        SeedType::FixedSeed(seed) => {
                                            let mut hasher = XxHash64::with_seed(0);
                                            render_tree[idx].value.seed.hash(&mut hasher);
                                            seed.hash(&mut hasher);
                                            hasher.finish()
                                        }
                                    },
                                    segment: s,
                                    error: None,
                                })
                                .collect();

                            for new_render in inserts {
                                render_tree.insert(new_render, Some(idx));
                            }

                            render_tree[idx].value.rendered = true;
                            render_tree[idx].value.error = None;
                            rendered_node_count += 1;
                        }
                    }
                }
            }

            println!(
                "Render pass {:?} rendered {:?} nodes.",
                render_pass, rendered_node_count
            );
            render_pass += 1;
            if rendered_node_count == 0 {
                break;
            }
        }

        render_tree
            .iter()
            .filter(|n| !n.value.rendered)
            .for_each(|n| println!("Unrendered: {:?}", n));

        render_tree
    }
}
