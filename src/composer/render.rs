use std::iter::successors;
use std::ops::Index;
use std::{
    any::TypeId,
    collections::HashMap,
    fmt::Debug,
    ops::{Add, Range},
};

use crate::error::RendererError;
use serde::{Deserialize, Serialize};

use super::{context::CompositionContext, CompositionElement, CompositionSegment};

pub type Result<T, E = RendererError> = std::result::Result<T, E>;

/// Trait used to describe render behavior. Every render operation during composition
/// involves a [`&CompositionElement`], [`&Range<i32>`], and [`&CompositionContext`] which produces
/// additional [`CompositionSegment`]s.
///
/// Renderers may return [`Vec<CompositionSegment`>] on success, or [`Error::MissingContext`] in the
/// case that its render dependencies are not satisfied.
pub trait Renderer {
    type Item: CompositionElement;

    /// Render a [`&CompositionElement`] for a given time range [`&Range<i32>`] and [`&CompositionContext`].
    ///
    /// Returns a [`Vec<CompositionSegment`>] on success, or [`Error::MissingContext`] in the case
    /// that its render dependencies are not satisfied.
    fn render(
        &self,
        segment: &Self::Item,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Result<Vec<CompositionSegment>>;
}

/// A struct implementing [`Renderer`] via a wrapped closure.
///
/// Most commonly used to implement a [`Renderer`] which does not require its own struct/state.
pub struct AdhocRenderer<T>
where
    T: CompositionElement,
{
    /// Boxed closure implementing the signature of [`Renderer::render`].
    pub func: Box<dyn Fn(&T, &Range<i32>, &CompositionContext) -> Result<Vec<CompositionSegment>>>,
}

impl<F, T> From<F> for AdhocRenderer<T>
where
    F: Fn(&T, &Range<i32>, &CompositionContext) -> Result<Vec<CompositionSegment>> + 'static,
    T: CompositionElement,
{
    /// Converts a closure into an [`AdhocRenderer`].
    fn from(value: F) -> Self {
        AdhocRenderer {
            func: Box::new(value),
        }
    }
}

impl<T> Renderer for AdhocRenderer<T>
where
    T: CompositionElement,
{
    type Item = T;

    /// Renders a [`CompositionElement`] by calling the [`AdhocRenderer`]s wrapped closure.
    fn render(
        &self,
        segment: &Self::Item,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Result<Vec<CompositionSegment>> {
        (self.func)(segment, time_range, context)
    }
}

/// A group of [`Renderer`]s for a single [`Renderer::Item`]. This group is itself a
/// [`Renderer`] which renders as a unit, returning [`Error::MissingContext`] if any of its
/// [`Renderer`]s do.
pub struct RendererGroup<T> {
    pub renderers: Vec<Box<dyn Renderer<Item = T>>>,
}

impl<T> RendererGroup<T> {
    pub fn new() -> RendererGroup<T> {
        RendererGroup { renderers: vec![] }
    }
}

impl<T, R> Add<R> for RendererGroup<T>
where
    R: Renderer<Item = T> + 'static,
{
    type Output = Self;

    fn add(mut self, rhs: R) -> Self::Output {
        self.renderers.push(Box::new(rhs));

        self
    }
}

impl<T> Renderer for RendererGroup<T>
where
    T: CompositionElement,
{
    type Item = T;

    fn render(
        &self,
        segment: &Self::Item,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Result<Vec<CompositionSegment>> {
        let mut result_children = vec![];

        for renderer in &self.renderers {
            result_children.append(&mut renderer.render(segment, time_range, context)?)
        }

        return Ok(result_children);
    }
}

trait ErasedRenderer {
    fn render(
        &self,
        segment: &dyn CompositionElement,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Result<Vec<CompositionSegment>>;
}

impl<T> ErasedRenderer for T
where
    T: Renderer,
{
    fn render(
        &self,
        segment: &dyn CompositionElement,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Result<Vec<CompositionSegment>> {
        self.render(
            segment.as_any().downcast_ref::<T::Item>().unwrap(),
            time_range,
            context,
        )
    }
}

/// A mapping of [`CompositionElement`] to [`Renderer`]s (via [`TypeId`]) used to delegate rendering of generic
/// [`CompositionSegment`]s via their [`CompositionElement`]. Only one [`Renderer`] per type is allowed
/// in the current implementation.
pub struct RenderEngine {
    renderers: HashMap<TypeId, Box<dyn ErasedRenderer>>,
}

impl RenderEngine {
    pub fn new() -> RenderEngine {
        RenderEngine {
            renderers: HashMap::new(),
        }
    }

    /// Adds a [`Renderer`] to this [`RenderEngine`], replacing any existing [`Renderer`] for
    /// the corresponding [`Renderer::Item`].
    pub fn add_renderer<R: Renderer + 'static>(&mut self, renderer: R) {
        self.renderers
            .insert(TypeId::of::<R::Item>(), Box::new(renderer));
    }

    /// Returns the [`Renderer`] corresponding to the given [`&dyn CompositionElement`], if one exists.
    fn renderer_for(&self, segment: &dyn CompositionElement) -> Option<&Box<dyn ErasedRenderer>> {
        self.renderers.get(&segment.as_any().type_id())
    }

    /// Determines if this [`RenderEngine`] can render a given `&dyn` [`CompositionElement`]. (i.e. whether
    /// it has a mapped renderer for the given `&dyn` [`CompositionElement`])
    ///
    /// This checks not only the given `&dyn` [`CompositionElement`], but also any types it wraps.
    /// See [`CompositionElement::wrapped_element`].
    pub fn can_render(&self, segment: &dyn CompositionElement) -> bool {
        successors(Some(segment), |&s| s.wrapped_element()).any(|s| self.can_render_specific(s))
    }

    /// Determines if this [`RenderEngine`] can render a given `&dyn` [`CompositionElement`]. Only checks
    /// the given type, ignoring any wrapped types (unlike [`Self::can_render`]).
    pub fn can_render_specific(&self, segment: &dyn CompositionElement) -> bool {
        self.renderers.contains_key(&segment.as_any().type_id())
    }

    /// Renders a [`CompositionElement`] over a given time range with supplied context, delegating to
    /// [`Renderer`]s mapped to its type and wrapped types if any. If no mapped [`Renderer`]
    /// for the type or wrapped types exists, [`None`] is returned.
    pub fn render(
        &self,
        segment: &dyn CompositionElement,
        time_range: &Range<i32>,
        context: &CompositionContext,
    ) -> Option<Result<Vec<CompositionSegment>>> {
        let renderables = successors(Some(segment), |&s| s.wrapped_element())
            .filter(|s| self.can_render_specific(*s))
            .collect::<Vec<_>>();

        if renderables.is_empty() {
            None
        } else {
            let mut generated_segments = vec![];

            for renderable in renderables {
                if let Some(renderer) = self.renderer_for(renderable) {
                    let result = renderer.render(renderable, time_range, context);

                    if let Ok(mut segments) = result {
                        generated_segments.append(&mut segments)
                    } else {
                        return Some(result);
                    }
                }
            }

            Some(Ok(generated_segments))
        }
    }
}

impl<R, S> Add<R> for RenderEngine
where
    R: Renderer<Item = S> + 'static,
    S: CompositionElement,
{
    type Output = Self;

    fn add(mut self, rhs: R) -> Self::Output {
        self.add_renderer(rhs);

        self
    }
}

impl Add<RenderEngine> for RenderEngine {
    type Output = Self;

    fn add(mut self, rhs: RenderEngine) -> Self::Output {
        self.renderers.extend(rhs.renderers);

        self
    }
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

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn root(&self) -> Option<&Node<T>> {
        self.get(0)
    }

    pub fn get(&self, idx: usize) -> Option<&Node<T>> {
        self.nodes.get(idx)
    }

    pub fn node_iter<'a>(&'a self, start: &'a Node<T>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: None,
        }
    }

    pub fn node_iter_with_skip<'a>(&'a self, start: &'a Node<T>, skip: Vec<usize>) -> NodeIter<T> {
        NodeIter {
            tree: self,
            idx_idx: 0,
            idxs: vec![&start.idx],
            skip: Some(skip),
        }
    }

    pub fn iter(&self) -> NodeIter<T> {
        match self.root() {
            Some(root) => self.node_iter(root),
            None => NodeIter {
                tree: self,
                idx_idx: 0,
                idxs: vec![],
                skip: None,
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

impl<T> Default for Tree<T> {
    fn default() -> Self {
        Self::new()
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
    skip: Option<Vec<usize>>,
}

impl<'a, T> Iterator for NodeIter<'a, T> {
    type Item = &'a Node<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.idxs.get(self.idx_idx) {
            Some(idx) => {
                let ret = &self.tree.nodes[**idx];
                self.idxs.append(
                    &mut ret
                        .children
                        .iter()
                        .filter(|n| {
                            if let Some(skip) = &self.skip {
                                !skip.contains(n)
                            } else {
                                true
                            }
                        })
                        .collect(),
                );
                self.idx_idx += 1;

                Some(ret)
            }
            None => None,
        }
    }
}

impl<Idx: std::slice::SliceIndex<[Node<T>]>, T> Index<Idx> for Tree<T> {
    type Output = Idx::Output;

    fn index(&self, index: Idx) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<Idx: std::slice::SliceIndex<[Node<T>]>, T> std::ops::IndexMut<Idx> for Tree<T> {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> Serialize for Tree<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializeHelperNode::from((&self[0], self)).serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Tree<T>
where
    T: Deserialize<'de> + Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(DeserializeHelperNode::deserialize(deserializer)?.into())
    }
}

// Private serialization helper struct
#[derive(Serialize)]
struct SerializeHelperNode<'a, T> {
    #[serde(flatten)]
    pub val: &'a T,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<SerializeHelperNode<'a, T>>,
}

#[derive(Deserialize)]
struct DeserializeHelperNode<T> {
    #[serde(flatten)]
    pub val: T,
    #[serde(default = "Vec::new")]
    pub children: Vec<DeserializeHelperNode<T>>,
}

impl<'a, T> From<(&'a Node<T>, &'a Tree<T>)> for SerializeHelperNode<'a, T> {
    fn from(value: (&'a Node<T>, &'a Tree<T>)) -> SerializeHelperNode<'a, T> {
        let (node, tree) = value;
        SerializeHelperNode {
            val: &node.value,
            children: node
                .children
                .iter()
                .map(|n| SerializeHelperNode::from((&tree[*n], tree)))
                .collect::<Vec<_>>(),
        }
    }
}

impl<T> From<DeserializeHelperNode<T>> for Tree<T> {
    fn from(value: DeserializeHelperNode<T>) -> Self {
        let mut nodes_to_add = vec![(0_usize, value, None)];
        let mut nodes = vec![];
        let mut id_counter = 1;

        while !nodes_to_add.is_empty() {
            let mut next_nodes = nodes_to_add
                .drain(..)
                .flat_map(|(idx, n, parent)| {
                    let (value, children) = (n.val, n.children);

                    let child_idx_range = id_counter..(id_counter + children.len());
                    id_counter += children.len();

                    nodes.push(Node {
                        idx,
                        value,
                        parent,
                        children: Vec::from_iter(child_idx_range.clone()),
                    });

                    child_idx_range
                        .zip(children.into_iter())
                        .map(move |(child_idx, child_node)| (child_idx, child_node, Some(idx)))
                })
                .collect::<Vec<_>>();

            nodes_to_add.append(&mut next_nodes);
        }

        Tree { nodes }
    }
}
