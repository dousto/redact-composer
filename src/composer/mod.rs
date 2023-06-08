use std::any::Any;
use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};

use rand::{thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;

use crate::composer::context::CompositionContext;

use self::render::Tree;

pub mod context;
pub mod render;

/// The base trait for any object that will be used as a composition element.
/// It should implement [`SegmentType::render`], or otherwise implement [`ConcreteSegmentType`].
pub trait SegmentType: Debug + AsAny + 'static {
    /// Returns a [`bool`] indicating whether this [`SegmentType`] should render. Generally this
    /// is not needed to be implemented, as any [`SegmentType`] that doesn't render should just
    /// implement the [`ConcreteSegmentType`] trait.
    fn renderable(&self) -> bool {
        true
    }

    /// Defines how the segment will render (producing child [`CompositionSegment`]s).
    fn render(&self, begin: i32, end: i32, context: CompositionContext) -> RenderResult;
}

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any + Debug + 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Trait used to mark [`SegmentType`]s which do not render anything.
pub trait ConcreteSegmentType {}

impl<T> SegmentType for T
where
    T: ConcreteSegmentType + Debug + 'static,
{
    fn renderable(&self) -> bool {
        false
    }

    fn render(&self, _begin: i32, _end: i32, _context: CompositionContext) -> RenderResult {
        RenderResult::Success(None)
    }
}

/// Simple struct to represent a given [`SegmentType`] which spans over a time range (`begin..end`).
#[derive(Debug)]
pub struct CompositionSegment {
    pub segment_type: Box<dyn SegmentType>,
    pub begin: i32,
    pub end: i32,
}

impl CompositionSegment {
    pub fn new<T: SegmentType>(composition_type: T, begin: i32, end: i32) -> CompositionSegment {
        CompositionSegment {
            segment_type: Box::new(composition_type),
            begin,
            end,
        }
    }

    /// Due to the non-generic-typed nature of [CompositionSegment], this method allows access downcasting to
    /// the specific type. Returns [`Option<&T>`], containing the reference to the [`SegmentType`] or [`None`]
    /// if the type does not match.
    pub fn segment_type_as<T: SegmentType>(&self) -> Option<&T> {
        (&*self.segment_type)
            .as_any()
            .downcast_ref::<T>()
            .or_else(|| {
                (&*self.segment_type)
                    .as_any()
                    .downcast_ref::<Part>()
                    .and_then(|p| (&*p.0).as_any().downcast_ref::<T>())
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

/// Note: Use [`redact_composer::musical::midi::Instrument`] for the time being.
#[derive(Debug)]
pub struct Instrument {
    pub program: u8,
}

impl ConcreteSegmentType for Instrument {}

#[derive(Debug)]
pub struct PlayNote {
    pub note: u8,
    pub velocity: u8,
}

impl ConcreteSegmentType for PlayNote {}

/// Part is a special signifier to group notes that are to be played by a single instrument at a time.
#[derive(Debug)]
pub struct Part(pub Box<dyn SegmentType>);

/// This is a simple pass-through implementation to the wrapped [`SegmentType`].
impl SegmentType for Part {
    fn render(&self, begin: i32, end: i32, context: CompositionContext) -> RenderResult {
        self.0.render(begin, end, context)
    }
}

impl Part {
    pub fn new(wrapped_type: impl SegmentType + 'static) -> Part {
        Part(Box::new(wrapped_type))
    }
}

// Composer stuff

#[derive(Debug)]
pub struct RenderSegment {
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
        Self::compose_with_seed(seg, thread_rng().next_u64())
    }
    /// Generates a render tree ([`Vec<Node<RenderSegment>>`]) from a starting [CompositionSegment].
    pub fn compose_with_seed(seg: CompositionSegment, seed: u64) -> Tree<RenderSegment> {
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
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
                if (&*render_tree[idx].value.segment.segment_type)
                    .as_any()
                    .is::<Part>()
                {
                    let mut parent = &render_tree[idx].parent;
                    while let Some(pidx) = parent {
                        if (&*render_tree[*pidx].value.segment.segment_type)
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

                match result {
                    RenderResult::Success(Some(segments)) => {
                        let inserts: Vec<RenderSegment> = segments
                            .into_iter()
                            .map(|s| RenderSegment {
                                rendered: !s.segment_type.renderable(),
                                segment: s,
                                seed: rng.next_u64(),
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
            if rendered_node_count <= 0 {
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
