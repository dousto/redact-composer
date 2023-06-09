use std::any::Any;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::ops::{Bound, RangeBounds};

use rand::{thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};
use twox_hash::XxHash64;

use crate::composer::context::CompositionContext;

use self::render::Tree;

#[cfg(test)]
mod test;

pub mod context;
pub mod render;

/// The base trait for any object that will be used as a composition element.
/// It should implement [`SegmentType::render`], or otherwise implement [`ConcreteSegmentType`].
#[typetag::serde]
pub trait SegmentType: Debug + AsAny + 'static {
    /// Returns a [`bool`] indicating whether this [`SegmentType`] should render. Generally this
    /// is not needed to be implemented, as any [`SegmentType`] that doesn't render should just
    /// implement the [`ConcreteSegmentType`] trait.
    fn renderable(&self) -> bool {
        true
    }

    /// Defines how the segment will render (producing child [`CompositionSegment`]s).
    fn render(&self, _begin: i32, _end: i32, _context: CompositionContext) -> RenderResult {
        RenderResult::Success(None)
    }
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Debug + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum SeedType {
    Random,
    FixedSeed(u64),
}

/// Simple struct to represent a given [`SegmentType`] which spans over a time range (`begin..end`).
#[derive(Debug, Serialize, Deserialize)]
pub struct CompositionSegment {
    #[serde(rename = "type")]
    pub segment_type: Box<dyn SegmentType>,
    pub begin: i32,
    pub end: i32,
    seeded_from: SeedType,
}

impl CompositionSegment {
    pub fn new<T: SegmentType>(composition_type: T, begin: i32, end: i32) -> CompositionSegment {
        CompositionSegment {
            segment_type: Box::new(composition_type),
            begin,
            end,
            seeded_from: SeedType::Random,
        }
    }

    pub fn named<T: SegmentType>(
        hashable_name: impl Hash,
        composition_type: T,
        begin: i32,
        end: i32,
    ) -> CompositionSegment {
        let mut hasher = XxHash64::with_seed(0);
        hashable_name.hash(&mut hasher);

        CompositionSegment {
            segment_type: Box::new(composition_type),
            begin,
            end,
            seeded_from: SeedType::FixedSeed(hasher.finish()),
        }
    }

    /// Due to the non-generic-typed nature of [CompositionSegment], this method allows access downcasting to
    /// the specific type. Returns [`Option<&T>`], containing the reference to the [`SegmentType`] or [`None`]
    /// if the type does not match.
    pub fn segment_type_as<T: SegmentType>(&self) -> Option<&T> {
        (*self.segment_type)
            .as_any()
            .downcast_ref::<T>()
            .or_else(|| {
                (*self.segment_type)
                    .as_any()
                    .downcast_ref::<Part>()
                    .and_then(|p| (*p.0).as_any().downcast_ref::<T>())
            })
    }

    fn render(&self, context: CompositionContext) -> RenderResult {
        self.segment_type.render(self.begin, self.end, context)
    }
}

impl RangeBounds<i32> for CompositionSegment {
    fn start_bound(&self) -> Bound<&i32> {
        Bound::Included(&self.begin)
    }

    fn end_bound(&self) -> Bound<&i32> {
        Bound::Excluded(&self.end)
    }
}

impl RangeBounds<i32> for &CompositionSegment {
    fn start_bound(&self) -> Bound<&i32> {
        Bound::Included(&self.begin)
    }

    fn end_bound(&self) -> Bound<&i32> {
        Bound::Excluded(&self.end)
    }
}

/// Note: Use [`redact_composer::musical::midi::Instrument`] for the time being.
#[derive(Debug, Serialize, Deserialize)]
pub struct Instrument {
    pub program: u8,
}

#[typetag::serde]
impl SegmentType for Instrument {
    fn renderable(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayNote {
    pub note: u8,
    pub velocity: u8,
}

#[typetag::serde]
impl SegmentType for PlayNote {
    fn renderable(&self) -> bool {
        false
    }
}

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
    fn render(&self, begin: i32, end: i32, context: CompositionContext) -> RenderResult {
        self.0.render(begin, end, context)
    }
}

impl Part {
    pub fn instrument(wrapped_type: impl SegmentType + 'static) -> Part {
        Part(Box::new(wrapped_type), PartType::Instrument)
    }

    pub fn percussion(wrapped_type: impl SegmentType + 'static) -> Part {
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
}

pub enum RenderResult {
    Success(Option<Vec<CompositionSegment>>),
    MissingContext,
}

pub struct Composer {}

impl Composer {
    pub fn compose(seg: CompositionSegment) -> Tree<RenderSegment> {
        let mut hasher = XxHash64::with_seed(0);
        thread_rng().next_u64().hash(&mut hasher);
        Self::compose_with_seed(seg, hasher.finish())
    }
    /// Generates a render tree ([`Vec<Node<RenderSegment>>`]) from a starting [CompositionSegment].
    pub fn compose_with_seed(mut seg: CompositionSegment, seed: u64) -> Tree<RenderSegment> {
        seg.seeded_from = SeedType::FixedSeed(seed);
        let mut render_tree = Tree::new();
        render_tree.insert(
            RenderSegment {
                rendered: false,
                seed,
                segment: seg,
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
                if (*render_tree[idx].value.segment.segment_type)
                    .as_any()
                    .is::<Part>()
                {
                    let mut parent = &render_tree[idx].parent;
                    while let Some(pidx) = parent {
                        if (*render_tree[*pidx].value.segment.segment_type)
                            .as_any()
                            .is::<Part>()
                        {
                            panic!("{}", "Part is not allowed to be nested.");
                        }
                        parent = &render_tree[*pidx].parent
                    }
                }

                let composition_context = CompositionContext::new(&render_tree, &render_tree[idx]);

                let result = render_tree[idx].value.segment.render(composition_context);

                let mut hasher = XxHash64::with_seed(0);
                let _ = &render_tree[idx].value.seed.hash(&mut hasher);
                let mut rng = ChaCha12Rng::seed_from_u64(hasher.finish());

                match result {
                    RenderResult::Success(Some(segments)) => {
                        let inserts: Vec<RenderSegment> = segments
                            .into_iter()
                            .map(|s| RenderSegment {
                                rendered: !s.segment_type.renderable(),
                                seed: match s.seeded_from {
                                    SeedType::Random => {
                                        let mut hasher = XxHash64::with_seed(0);
                                        rng.next_u64().hash(&mut hasher);
                                        hasher.finish()
                                    }
                                    SeedType::FixedSeed(seed) => {
                                        let mut hasher = XxHash64::with_seed(0);
                                        let _ = &render_tree[idx].value.seed.hash(&mut hasher);
                                        seed.hash(&mut hasher);
                                        hasher.finish()
                                    }
                                },
                                segment: s,
                            })
                            .collect();

                        for new_render in inserts {
                            render_tree.insert(new_render, Some(idx));
                        }

                        render_tree[idx].value.rendered = true;
                        rendered_node_count += 1;
                    }
                    RenderResult::Success(None) => {
                        render_tree[idx].value.rendered = true;
                        rendered_node_count += 1;
                    }
                    RenderResult::MissingContext => (),
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
