/// Context structs involved during composition rendering.
pub mod context;

/// Basic n-ary tree implementation.
pub mod tree;

use crate::error::RendererError;

use std::fmt::Formatter;
use std::iter::successors;
use std::ops::Deref;
use std::{any::TypeId, collections::HashMap, fmt::Debug, ops::Add};
use Vec;

use crate::{Element, Segment, SegmentRef};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::render::context::CompositionContext;

/// [`Result`](std::result::Result) with a default error type of [`RendererError`].
pub type Result<T, E = RendererError> = std::result::Result<T, E>;

/// Defines render behavior for a specific [`Element`](Self::Element).
///
/// Every render operation during composition receives a
/// [`SegmentRef<Self::Element>`](crate::SegmentRef<Self::Element>) with
/// [`CompositionContext`] and may return [`Vec<Segment`>] on success, or
/// [`RendererError::MissingContext`] in the case that its render dependencies are not satisfied
/// (which will be retried later).
pub trait Renderer {
    /// The particular [`Element`] this [`Renderer`] renders.
    type Element: Element;

    /// Renderers a [`SegmentRef<Self::Element>`] with [`CompositionContext`], returning additional
    /// [`Segment`]s as children.
    fn render(
        &self,
        segment: SegmentRef<Self::Element>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>>;
}

/// Wraps a [`Segment`] with additional render-related information.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderSegment {
    /// The wrapped [`Segment`].
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub segment: Segment,
    /// Seed used for [`CompositionContext`] rng when this segment is rendered.
    pub seed: u64,
    /// Initially `false`, becoming `true` only after this segment has been successfully rendered.
    pub rendered: bool,
    /// Stores the latest encountered [`RendererError`] for debugging.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<RendererError>,
}

/// Implements a [`Renderer`] via a wrapped closure.
///
/// Most commonly used to implement a [`Renderer`] which does not require its own struct/state.
///
/// ```
/// # use serde::{Deserialize, Serialize};
/// # use redact_composer_core::derive::Element;
/// # use redact_composer_core::elements::PlayNote;
/// # use redact_composer_core::IntoSegment;
/// # use redact_composer_core::render::AdhocRenderer;
/// # #[derive(Element, Debug, Serialize, Deserialize)]
/// # struct SomeElement;
/// let renderer = AdhocRenderer::<SomeElement>::new(|segment, context| {
///     Ok(vec![
///         PlayNote {note: 60, velocity: 100 }
///         .over(segment.timing)
///     ])
/// });
/// ```
#[allow(missing_debug_implementations)] // TODO
pub struct AdhocRenderer<T: Element> {
    /// Closure implementing the signature of
    /// [`Renderer::render`](crate::render::Renderer::render).
    #[allow(clippy::type_complexity)]
    func: Box<dyn Fn(SegmentRef<T>, CompositionContext) -> Result<Vec<Segment>>>,
}

impl<T: Element> AdhocRenderer<T> {
    /// Creates an [`AdhocRenderer`] from a closure.
    pub fn new(
        func: impl Fn(SegmentRef<T>, CompositionContext) -> Result<Vec<Segment>> + 'static,
    ) -> AdhocRenderer<T> {
        AdhocRenderer {
            func: Box::new(func),
        }
    }
}

impl<T: Element> Renderer for AdhocRenderer<T> {
    type Element = T;

    /// Renders a [`Element`] by calling the [`AdhocRenderer`]s wrapped closure.
    fn render(
        &self,
        segment: SegmentRef<Self::Element>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        (self.func)(segment, context)
    }
}

/// A group of [`Renderer`]s for a single [`Renderer::Element`]. This group is itself a
/// [`Renderer`] which renders as a unit, returning [`crate::error::RendererError`] if any of its
/// [`Renderer`]s do.
#[allow(missing_debug_implementations)] // TODO
pub struct RendererGroup<T> {
    /// The renderers of this group.
    pub renderers: Vec<Box<dyn Renderer<Element = T>>>,
}

impl<T> RendererGroup<T> {
    /// Creates an empty [`RendererGroup`].
    pub fn new() -> RendererGroup<T> {
        RendererGroup { renderers: vec![] }
    }
}

impl<T> Default for RendererGroup<T> {
    fn default() -> Self {
        RendererGroup::new()
    }
}

impl<T, R> Add<R> for RendererGroup<T>
where
    R: Renderer<Element = T> + 'static,
{
    type Output = Self;

    fn add(mut self, rhs: R) -> Self::Output {
        self.renderers.push(Box::new(rhs));

        self
    }
}

impl<T: Element> Renderer for RendererGroup<T> {
    type Element = T;

    fn render(
        &self,
        segment: SegmentRef<Self::Element>,
        context: CompositionContext,
    ) -> Result<Vec<Segment>> {
        let mut result_children = vec![];

        for renderer in &self.renderers {
            result_children.append(&mut renderer.render(segment, context)?);
        }

        Ok(result_children)
    }
}

trait ErasedRenderer {
    fn render(&self, segment: &Segment, context: CompositionContext) -> Result<Vec<Segment>>;
}

impl<T: Renderer> ErasedRenderer for T {
    fn render(&self, segment: &Segment, context: CompositionContext) -> Result<Vec<Segment>> {
        self.render(segment.try_into()?, context)
    }
}

/// A mapping of [`Element`] to [`Renderer`]s used to delegate rendering of generic
/// [`Segment`]s via their [`Element`]. Only one [`Renderer`] per type is
/// allowed in the current implementation.
#[allow(missing_debug_implementations)] // TODO
#[derive(Default)]
pub struct RenderEngine {
    renderers: HashMap<TypeId, Box<dyn ErasedRenderer>>,
}

impl Debug for RenderEngine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // TODO
        write!(f, "RenderEngine {{ /* TODO */ }}")
    }
}

impl RenderEngine {
    /// Creates an empty [`RenderEngine`].
    pub fn new() -> RenderEngine {
        RenderEngine {
            renderers: HashMap::new(),
        }
    }

    /// Adds a [`Renderer`] to this [`RenderEngine`], replacing any existing [`Renderer`] for
    /// the corresponding [`Renderer::Element`].
    pub fn add_renderer<R: Renderer + 'static>(&mut self, renderer: R) {
        self.renderers
            .insert(TypeId::of::<R::Element>(), Box::new(renderer));
    }

    /// Returns the [`Renderer`] corresponding to the given [`&dyn Element`], if one exists.
    fn renderer_for(&self, element: &dyn Element) -> Option<&dyn ErasedRenderer> {
        self.renderers
            .get(&element.as_any().type_id())
            .map(Box::deref)
    }

    /// Determines if this [`RenderEngine`] can render a given `&dyn` [`Element`]. (i.e. whether
    /// it has a mapped renderer for the given `&dyn` [`Element`])
    ///
    /// This checks not only the given `&dyn` [`Element`], but also any types it wraps.
    pub fn can_render(&self, element: &dyn Element) -> bool {
        successors(Some(element), |&s| s.wrapped_element()).any(|s| self.can_render_specific(s))
    }

    /// Determines if this [`RenderEngine`] can render a given `&dyn` [`Element`]. Only checks
    /// the given type, ignoring any wrapped types (unlike [`Self::can_render`]).
    pub fn can_render_specific(&self, element: &dyn Element) -> bool {
        self.renderers.contains_key(&element.as_any().type_id())
    }

    /// Renders a [`Element`] over a given time range with supplied context, delegating to
    /// [`Renderer`]s mapped to its type and wrapped types if any. If no mapped [`Renderer`]
    /// for the type or wrapped types exists, [`None`] is returned.
    pub fn render(
        &self,
        segment: &Segment,
        context: CompositionContext,
    ) -> Option<Result<Vec<Segment>>> {
        let renderables = successors(Some(&*segment.element), |&s| s.wrapped_element())
            .filter(|s| self.can_render_specific(*s))
            .collect::<Vec<_>>();

        if renderables.is_empty() {
            None
        } else {
            let mut generated_segments = vec![];

            for renderable in renderables {
                if let Some(renderer) = self.renderer_for(renderable) {
                    let result = renderer.render(segment, context);

                    if let Ok(mut segments) = result {
                        generated_segments.append(&mut segments);
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
    R: Renderer<Element = S> + 'static,
    S: Element,
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
