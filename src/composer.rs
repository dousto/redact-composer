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
    Instrument { program: u8 },

    /// A note to be played with given pitch and velocity. Timing of the played note comes from
    /// its associated [`CompositionSegment`]
    PlayNote { note: u8, velocity: u8 },
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
    MissingContext,
}

/// Trait defining render behavior for a generic type.
pub trait Renderer<T> {
    /// Renders a given [T] with the timing (begin, end) and [CompositionContext].
    ///
    /// Outputs a [RenderResult] containing the children [CompositionSegment]s of the input node.
    /// Children are required to be fully contained in the timing of the input node
    /// (`child.begin >= begin` and `child.end <= end).
    fn render(
        &self,
        t: &T,
        begin: u32,
        end: u32,
        context: &CompositionContext<T>,
    ) -> RenderResult<T>;
}

#[derive(Debug)]
pub struct Node<T> {
    pub idx: usize,
    pub value: T,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
}

#[derive(Debug)]
pub struct Tree<T> {
    nodes: Vec<Node<T>>,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree { nodes: vec![] }
    }

    pub fn root(&self) -> Option<&Node<T>> {
        self.nodes.get(0)
    }

    pub fn node_iter<'a>(&'a self, start: &'a Node<T>) -> NodeIter<T> {
        NodeIter {
            tree: &self,
            idx_idx: 0,
            idxs: vec![&start.idx],
        }
    }

    pub fn iter(&self) -> NodeIter<T> {
        match self.root() {
            Some(root) => self.node_iter(root),
            None => NodeIter {
                tree: &self,
                idx_idx: 0,
                idxs: vec![],
            },
        }
    }

    pub fn insert(&mut self, item: T, parent_idx: Option<usize>) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            idx: new_idx,
            parent: parent_idx,
            value: item,
            children: vec![],
        });

        if let Some(parent_idx) = parent_idx {
            self.nodes[parent_idx].children.push(new_idx)
        }

        new_idx
    }
}

impl<'a, T> IntoIterator for &'a Tree<T> {
    type Item = &'a Node<T>;

    type IntoIter = NodeIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct NodeIter<'a, T> {
    tree: &'a Tree<T>,
    idx_idx: usize,
    idxs: Vec<&'a usize>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.idxs.get(self.idx_idx) {
            Some(idx) => {
                let ret = &self.tree.nodes[**idx];
                self.idxs.append(&mut ret.children.iter().collect());
                self.idx_idx += 1;

                Some(ret)
            }
            None => None,
        }
    }
}

impl<T> std::ops::Index<usize> for Tree<T> {
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> std::ops::IndexMut<usize> for Tree<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

pub struct Composer<T, K: Renderer<T>> {
    pub renderer: K,
    _dummy: PhantomData<T>,
}

impl<T, K> Composer<T, K>
where
    T: Debug,
    K: Renderer<T>,
{
    pub fn new(renderer: K) -> Composer<T, K> {
        Composer {
            renderer,
            _dummy: PhantomData,
        }
    }

    /// Generates a render tree ([`Vec<Node<RenderSegment<T>>>`]) from a starting [CompositionSegment].
    pub fn compose(&self, seg: CompositionSegment<T>) -> Tree<RenderSegment<T>> {
        let mut render_tree = Tree::new();
        render_tree.insert(
            RenderSegment {
                rendered: false,
                segment: seg,
            },
            None,
        );

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

            let unrendered: Vec<usize> = render_tree
                .iter()
                .filter(|n| !n.value.rendered)
                .map(|n| n.idx)
                .collect();

            for idx in unrendered {
                if let SegmentType::Part(_) = &render_tree[idx].value.segment.segment_type {
                    let mut parent = &render_tree[idx].parent;
                    while let Some(pidx) = parent {
                        if let SegmentType::Part(_) = &render_tree[*pidx].value.segment.segment_type
                        {
                            panic!("{}", "SegmentType::Part(_) is not allowed to be nested.");
                        }
                        parent = &render_tree[*pidx].parent
                    }
                }
                match &render_tree[idx].value.segment.segment_type {
                    SegmentType::Abstract(t) | SegmentType::Part(t) => {
                        let composition_context = CompositionContext {
                            tree: &render_tree,
                            start: &render_tree[idx],
                        };

                        let result = self.renderer.render(
                            t,
                            render_tree[idx].value.segment.begin,
                            render_tree[idx].value.segment.end,
                            &composition_context,
                        );

                        match result {
                            RenderResult::Success(segments) => {
                                match segments {
                                    Some(segs) => {
                                        let inserts: Vec<RenderSegment<T>> = segs
                                            .into_iter()
                                            .map(|s| RenderSegment {
                                                rendered: false,
                                                segment: s,
                                            })
                                            .collect();

                                        for new_render in inserts {
                                            render_tree.insert(new_render, Some(idx));
                                        }
                                    }
                                    None => (),
                                }

                                println!("{:?}", render_tree[idx]);
                                render_tree[idx].value.rendered = true;
                                rendered_node_count += 1;
                            }
                            RenderResult::MissingContext => (),
                        }
                    }
                    _ => println!("{:?}", render_tree[idx]),
                };
            }

            println!("Rendered {:?} nodes.", rendered_node_count);
            if rendered_node_count <= 0 {
                break;
            }
        }

        for node in &render_tree {
            println!("{:?}", node)
        }

        render_tree
    }
}

/// Type used during the render of abstract CompositionSegments which allows lookup
/// of data from other composition tree nodes.
///
/// ## Fields
/// * `tree: &Tree<RenderSegment<T>>` A reference to the (in-progress) composition tree
/// * `start: &Node<RenderSegment<T>>` The node being rendered. Lookups are relative to this node.
pub struct CompositionContext<'a, T> {
    tree: &'a Tree<RenderSegment<T>>,
    start: &'a Node<RenderSegment<T>>,
}

impl<'a, T: Debug> CompositionContext<'a, T> {
    /// Look up the deepest CompositionSegment matching `abstract_type` node whose (begin, end) bounds wholly contains the `start` node.
    /// TODO: If multiple matches, return the match from the node with closest common ancester to start node
    pub fn get<F, K: Debug>(&self, func: F) -> Option<K>
    where
        F: Fn(&'a T) -> Option<K>,
    {
        let mut matching_thing: Option<K> = None;

        let iter = self.tree.iter().filter(|n| {
            n.value.rendered
                && n.value.segment.begin <= self.start.value.segment.begin
                && n.value.segment.end >= self.start.value.segment.end
        });

        for node in iter {
            if let SegmentType::Abstract(thing) = &node.value.segment.segment_type {
                let found = func(thing);

                if found.is_some() {
                    matching_thing = found
                }
            }
        }

        matching_thing
    }
}
