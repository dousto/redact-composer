use std::{fmt::Debug, marker::PhantomData};

/// Represents the type for each CompositionSegment. The generic parameter is 
/// a delegate type chosen by the [`Renderer`] implementation.
/// 
/// [`Abstract`](SegmentType::Abstract) and [`Part`](SegmentType::Part) both represent abstract types
/// which will be passed to a [`Renderer`] during [`Composer::compose()`].
#[derive(Debug, PartialEq)]
pub enum SegmentType<T> {
    /// An abstract segment that will be further rendered by a [`Renderer<T>`].
    Abstract(T),

    /// Almost identical to [`SegmentType::Abstract`], but in addition, marks a group
    /// which includes all children of the [`CompositionSegment`] (used to assign midi channels)
    Part(T),

    /// Represents a midi program to be used for all [`SegmentType::PlayNote`] children within
    /// its [`CompositionSegment`]'s time range.
    Instrument { program: i8 },

    /// A note to be played with given pitch and velocity. Timing of the played note comes from
    /// its associated [`CompositionSegment`]
    PlayNote { note: u8, velocity: u8 }
}

#[derive(Debug, PartialEq)]
pub struct CompositionSegment<T> {
    pub segment_type: SegmentType<T>,
    pub begin: u32,
    pub end: u32,
}
#[derive(Debug, PartialEq)]
pub struct RenderSegment<T> {
    pub segment: CompositionSegment<T>,
    pub rendered: bool,
}

pub enum RenderResult<T> {
    Success(Option<Vec<CompositionSegment<T>>>),
    MissingContext
}

/// Trait defining render behavior for a generic type.
pub trait Renderer<T> {
    /// Renders a given [T] with the timing (begin, end) and [CompositionContext].
    /// 
    /// Outputs a [RenderResult] containing the children [CompositionSegment]s of the input node.
    /// Children are required to be fully contained in the timing of the input node
    /// (`child.begin >= begin` and `child.end <= end).
    fn render(&self, t: &T, begin: u32, end: u32, context: &CompositionContext<T>
    ) -> RenderResult<T>;
}

#[derive(Debug)]
pub struct Node<T> {
    pub idx: usize,
    pub value: T,
    pub parent: Option<usize>,
    pub children: Vec<usize>
}

pub struct Composer<T, K: Renderer<T>> {
    pub renderer: K,
    _dummy: PhantomData<T>
}

impl<T, K> Composer<T, K>
   where T: Debug, K: Renderer<T> {
    pub fn new(renderer: K) -> Composer<T, K> {
        Composer {
            renderer,
            _dummy: PhantomData,
        }
    }

    /// Generates a render tree ([`Vec<Node<RenderSegment<T>>>`]) from a starting [CompositionSegment].
    pub fn compose(&self, seg: CompositionSegment<T>) -> Vec<Node<RenderSegment<T>>> {
        let mut render_nodes: Vec<Node<RenderSegment<T>>> = vec![
            Node {
                parent: None,
                idx: 0,
                value: RenderSegment { rendered: false, segment: seg },
                children: vec![]
            }
        ];

        let mut rendered_node_count: usize;
        loop {
            // The high level loop flow is as follows:
            // 1. Search the tree (render_nodes) for all unrendered CompositionSegments nodes
            // 2. For each unrendered `SegmentType::Abstract` node, call its renderer which produces child CompositionSegments,
            //       data updates (HashMap<String, String>) for itself, or both.
            // 3. Add to/update the composition tree based on the RenderResult outputs (as previously mentioned)
            //       Note: New nodes are always inserted as unrendered.
            // 4. Repeat until a state is reached where no additional nodes can be rendered.
            rendered_node_count = 0;
            let unrendered: Vec<usize> = render_nodes.iter().filter(|n| !n.value.rendered).map(|n| n.idx).collect();

            for idx in unrendered {
                match &render_nodes[idx].value.segment.segment_type {
                    SegmentType::Part(_) => {
                        let mut parent = &render_nodes[idx].parent;
                        while let Some(pidx) = parent {
                            match &render_nodes[*pidx].value.segment.segment_type {
                                SegmentType::Part(_) => panic!("{}", "SegmentType::Part(_) is not allowed to be nested."),
                                _ => ()
                            }
                            parent = &render_nodes[*pidx].parent
                        }
                    },
                    _ => (),
                }
                match &render_nodes[idx].value.segment.segment_type {
                    SegmentType::Abstract(t) | SegmentType::Part(t) => {
                        let composition_context = CompositionContext {
                            tree: &render_nodes[..],
                            start: &render_nodes[idx],
                        };

                        let result = self.renderer.render(
                            t,
                            render_nodes[idx].value.segment.begin, render_nodes[idx].value.segment.end,
                            &composition_context
                        );

                        match result {
                            RenderResult::Success(segments) => {
                                match segments {
                                    Some(segs) => {
                                        let inserts: Vec<RenderSegment<T>> = segs.into_iter()
                                        .map(|s| RenderSegment {
                                            rendered: false,
                                            segment: s
                                        }).collect();

                                        for new_render in inserts {
                                            let next_idx = render_nodes.len();
                                            render_nodes.push(
                                                Node {
                                                    idx: next_idx,
                                                    parent: Some(idx),
                                                    value: new_render,
                                                    children: vec![],
                                                }
                                            );
                                            render_nodes[idx].children.push(next_idx);
                                        }
                                    },
                                    None => (),
                                }

                                println!("{:?}", render_nodes[idx]);
                                render_nodes[idx].value.rendered = true;
                                rendered_node_count += 1;
                            },
                            RenderResult::MissingContext => (),
                        }
                    },
                    _ => println!("{:?}", render_nodes[idx])
                    
                };
            }

            println!("Rendered {:?} nodes.", rendered_node_count);
            if rendered_node_count <= 0 { break; }
        }

        for node in &render_nodes {
            println!("{:?}", node)
        }

        render_nodes
    }
}

/// Type used during the render of abstract CompositionSegments which allows lookup
/// of data from other composition tree nodes.
/// 
/// ## Fields
/// * `tree: [[Node<RenderSegment<T>>]]` A slice snapshot of the current composition tree
/// * `start: [Node<RenderSegment<T>>]` The node being rendered. Lookups are relative to this node.
pub struct CompositionContext<'a, T> {
    tree: &'a [Node<RenderSegment<T>>],
    start: &'a Node<RenderSegment<T>>
}

impl<'a, T> CompositionContext<'a, T> {
    /// Look up the deepest CompositionSegment matching `abstract_type` node whose (begin, end) bounds wholly contains the `start` node.
    pub fn get<F, K>(&self, func: F) -> Option<K> 
        where F: Fn(&T) -> Option<K> {
        let mut node_iters: Vec<usize> = vec![0];
        let mut matching_thing: Option<K> = None;
        
        while !node_iters.is_empty() {
            node_iters = node_iters.into_iter().filter(|idx| {
                let render_segment = &self.tree[*idx].value;
                render_segment.rendered
                && render_segment.segment.begin <= self.start.value.segment.begin
                && render_segment.segment.end >= self.start.value.segment.end
            }).collect();

            for idx in &node_iters {
                match &self.tree[*idx].value.segment.segment_type {
                    SegmentType::Abstract(thing) => {
                        let found = func(thing);

                        if found.is_some() { matching_thing = found }
                    },
                    _ => ()
                }
            }

            node_iters = node_iters.iter().flat_map(|idx| &self.tree[*idx].children).map(|t| *t).collect();
        }

        matching_thing
    }
}
