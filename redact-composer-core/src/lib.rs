#![deny(missing_docs, missing_debug_implementations)]
//! Core library for `redact_composer`. Lib-level crates should depend on this rather than the
//! application-level `redact_composer`.

extern crate self as redact_composer_core;

/// Error types.
pub mod error;

/// Types and traits used for and during composition rendering.
pub mod render;

/// Timing related structs and elements.
pub mod timing;
/// Re-exports of non-deterministic [`std::collections`], with deterministic defaults.
pub mod util;

#[cfg(test)]
mod test;

use rand::{thread_rng, RngCore, SeedableRng};
use rand_chacha::ChaCha12Rng;
use std::any::TypeId;
use std::collections::{Bound, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::iter::successors;
use std::ops::{Range, RangeBounds};
use twox_hash::XxHash64;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::ConversionError;
use crate::render::context::CompositionContext;
use crate::render::{tree::Tree, RenderEngine, RenderSegment};
use crate::timing::{Timing, STANDARD_BEAT_LENGTH};

/// Contains the derive macro of [`Element`]. Specifically kept separate in core, so
/// exporting trait vs macro can be done separately
pub mod derive {
    pub use redact_composer_derive::ElementCore as Element;
}

use std::any::Any;

const LOG: &str = "redact_composer";

/// Marker trait for any type that will be used as a composition element.
///
/// Can be implemented via its derive macro:
/// ```no_run
/// # use serde::{Deserialize, Serialize};
/// # use redact_composer_core::derive::Element;
/// # #[derive(Debug, Serialize, Deserialize)]
/// #[derive(Element)]
/// pub struct CustomCompositionElement;
/// ```
///
/// If implementing maually, remember to tag the impl block with `#[typetag::serde]` for proper
/// serialization behavior.
///
/// **Advanced**: Overriding the default [`wrapped_element`](crate::Element::wrapped_element) method
/// indicates another element this one wraps. Wrapped elements will render alongside their wrappers,
/// producing a cumulative set of children. Mainly used to provide a common 'tag' type for an
/// unknown set of other elements, enabling context lookups or other operations that depend on
/// element type.
#[cfg_attr(feature = "serde", typetag::serde)]
pub trait Element: Debug + AsAny + 'static {
    /// None.
    fn wrapped_element(&self) -> Option<&dyn Element> {
        None
    }
}

/// Convenience trait for converting to [`&dyn Any`].
pub trait AsAny {
    /// Converts this to a [`&dyn Any`].
    fn as_any(&self) -> &dyn Any;
}

impl<T: Element> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A (type-erased) [`Element`] spanning a [`Timing`] interval.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Segment {
    /// The element this segment represents.
    pub element: Box<dyn Element>,
    /// The timing interval this segment spans.
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub timing: Timing,
    /// An optional name. When being rendered, this segment's Rng are seeded with a combined hash of
    /// this name as well as the segment's parent seed.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub name: Option<String>,
}

impl Segment {
    /// Creates a new [`Segment`] from a [`Element`] which starts/ends
    /// according to `timing`.
    pub fn new(composition_type: impl Element, timing: impl Into<Timing>) -> Segment {
        Segment {
            element: Box::new(composition_type),
            timing: timing.into(),
            name: Option::default(),
        }
    }

    /// Gives this [`Segment`] a name, used to seed its [`Rng`](rand::Rng), and returns the new named version. Useful if
    /// you want certain segments to be repeated/reproduced (i.e. be rendered with the same Rng).
    pub fn named(mut self, name: String) -> Self {
        self.name = Some(name);

        self
    }

    /// Removes the [`Segment`]'s a name, if it has one, returning the resulting unnamed segment.
    pub fn unnamed(mut self) -> Self {
        self.name = None;

        self
    }

    /// Gets the contained element if its type matches type `Element`, otherwise, `None` is
    /// returned.
    pub fn element_as<Element: crate::Element>(&self) -> Option<&Element> {
        successors(Some(&*self.element), |s| s.wrapped_element())
            .find_map(|s| s.as_any().downcast_ref::<Element>())
    }
}

impl RangeBounds<i32> for Segment {
    fn start_bound(&self) -> Bound<&i32> {
        self.timing.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.timing.end_bound()
    }
}

impl RangeBounds<i32> for &Segment {
    fn start_bound(&self) -> Bound<&i32> {
        self.timing.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.timing.end_bound()
    }
}

/// A typed view of a [`Segment`] (references to its fields).
#[derive(Debug)]
pub struct SegmentRef<'a, T: Element> {
    /// The element reference.
    pub element: &'a T,
    /// The segment's timing reference.
    pub timing: &'a Timing,
    /// The segment's name reference.
    pub name: &'a Option<String>,
}

impl<'a, T: Element> Clone for SegmentRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T: Element> Copy for SegmentRef<'a, T> {}

impl<'a, T: Element> TryFrom<&'a Segment> for SegmentRef<'a, T> {
    type Error = ConversionError;

    fn try_from(value: &'a Segment) -> std::result::Result<Self, Self::Error> {
        if let Some(casted_ref) = value.element_as::<T>() {
            Ok(SegmentRef {
                element: casted_ref,
                timing: &value.timing,
                name: &value.name,
            })
        } else {
            Err(ConversionError::TypeMismatch)
        }
    }
}

impl<'a, T> IntoSegment for SegmentRef<'a, T>
where
    T: Element + Clone,
{
    /// Turns this [`SegmentRef`] into a new [`Segment`] with a given
    /// `timing`.
    ///
    /// Note: This also copies the `name` if it exists. If this is not desired, use:
    /// ```
    /// # use redact_composer_core::elements::PlayNote;
    /// # use redact_composer_core::SegmentRef;
    /// # use redact_composer_core::timing::Timing;
    /// # use redact_composer_core::IntoSegment;
    /// # let timing = Timing::from(0..1);
    /// # let s = (PlayNote { note: 0, velocity: 0}, Timing::from(0..1), None);
    /// # let segment_ref = SegmentRef { element: &s.0,timing: &s.1, name: &s.2 };
    /// let segment = segment_ref.into_segment(timing).unnamed();
    /// ```
    fn into_segment(self, timing: impl Into<Timing>) -> Segment {
        if let Some(name) = self.name {
            self.element
                .clone()
                .into_segment(timing)
                .named(name.clone())
        } else {
            self.element.clone().into_segment(timing)
        }
    }

    fn over(self, timing: impl Into<Timing>) -> Segment {
        self.into_segment(timing)
    }
}

impl<'a, T: Element> RangeBounds<i32> for SegmentRef<'a, T> {
    fn start_bound(&self) -> Bound<&i32> {
        self.timing.start_bound()
    }

    fn end_bound(&self) -> Bound<&i32> {
        self.timing.end_bound()
    }
}

impl<'a, T: Element> From<SegmentRef<'a, T>> for Timing {
    fn from(value: SegmentRef<'a, T>) -> Self {
        *value.timing
    }
}

impl<'a, T: Element> From<&SegmentRef<'a, T>> for Timing {
    fn from(value: &SegmentRef<'a, T>) -> Self {
        *value.timing
    }
}

impl<'a, T: Element> From<SegmentRef<'a, T>> for Range<i32> {
    fn from(value: SegmentRef<'a, T>) -> Self {
        value.timing.into()
    }
}

impl<'a, T: Element> From<&SegmentRef<'a, T>> for Range<i32> {
    fn from(value: &SegmentRef<'a, T>) -> Self {
        value.timing.into()
    }
}

/// Conversion methods to create a [`Segment`] from a [`Element`].
pub trait IntoSegment: private::Sealed {
    /// Creates a [`Segment`] from this element, spanning the given time range.
    fn into_segment(self, timing: impl Into<Timing>) -> Segment;

    /// Creates a [`Segment`] from this element, spanning the given time range.
    fn over(self, timing: impl Into<Timing>) -> Segment;
}

impl<T: Element> IntoSegment for T {
    /// Creates a [`Segment`] from this element, spanning the given time range.
    fn into_segment(self, timing: impl Into<Timing>) -> Segment {
        Segment::new(self, timing)
    }

    /// Creates a [`Segment`] from this element, spanning the given time range.
    fn over(self, timing: impl Into<Timing>) -> Segment {
        self.into_segment(timing)
    }
}

/// Core types implementing [`Element`].
pub mod elements {
    use super::PartType;
    use crate::derive::Element;
    use crate::Element;

    #[cfg(feature = "serde")]
    use serde::{Deserialize, Serialize};

    /// Play a note with a velocity.
    #[derive(Element, Clone, Copy, Debug, Eq, PartialEq)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    pub struct PlayNote {
        /// Note represented as u8, with `note % 12 == 0` representing 'C'.
        pub note: u8,
        /// The strength of attack of the note.
        pub velocity: u8,
    }

    /// Wraps another element, indicating that notes rendered from the wrapped element are to be
    /// played by a single instrument at a time.
    #[derive(Element, Debug)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    #[element(wrapped_element = self.wrapped_element())]
    pub struct Part(pub(super) Box<dyn Element>, pub(super) PartType);
}
use elements::Part;
use log::{debug, info, log_enabled, trace, warn, Level};

/// Indicates whether a part is an instrument, or percussion.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PartType {
    /// Instrument part.
    Instrument,
    /// Percussion part.
    Percussion,
}

impl Part {
    /// Creates a new instrument part from the given element.
    pub fn instrument(wrapped_element: impl Element) -> crate::elements::Part {
        Part(Box::new(wrapped_element), PartType::Instrument)
    }

    /// Creates a new percussion part from the given element.
    pub fn percussion(wrapped_element: impl Element) -> crate::elements::Part {
        Part(Box::new(wrapped_element), PartType::Percussion)
    }

    /// Returns the wrapped element.
    pub fn wrapped_element(&self) -> Option<&dyn Element> {
        Some(&*self.0)
    }

    /// Returns the type of this part.
    pub fn part_type(&self) -> &PartType {
        &self.1
    }
}

/// Options used by a [`Composer`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ComposerOptions {
    /// The number of ticks per beat.
    pub ticks_per_beat: i32,
}

impl Default for ComposerOptions {
    fn default() -> Self {
        Self {
            ticks_per_beat: STANDARD_BEAT_LENGTH,
        }
    }
}

/// Provides methods to create compositions using a [`RenderEngine`] and its
/// [`Renderer`](render::Renderer)s.
#[derive(Debug, Default)]
pub struct Composer {
    /// The render engine used when rendering compositions.
    pub engine: RenderEngine,
    /// The composer's options.
    pub options: ComposerOptions,
}

impl From<RenderEngine> for Composer {
    fn from(value: RenderEngine) -> Self {
        Composer {
            engine: value,
            ..Default::default()
        }
    }
}

/// Options used during the rendering of a [`Composition`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompositionOptions {
    /// The number of ticks per beat.
    pub ticks_per_beat: i32,
}

impl Default for CompositionOptions {
    fn default() -> Self {
        Self {
            ticks_per_beat: STANDARD_BEAT_LENGTH,
        }
    }
}

impl From<ComposerOptions> for CompositionOptions {
    fn from(value: ComposerOptions) -> Self {
        Self {
            ticks_per_beat: value.ticks_per_beat,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// A composition output, including the tree of rendered segments, produced from
/// [`Composer::compose`].
pub struct Composition {
    /// The options used during this composition.
    pub options: CompositionOptions,
    /// The tree of rendered [`Segment`]s produced during composition.
    pub tree: Tree<RenderSegment>,
}

impl Composer {
    /// Generates a [`Composition`] from a starting [Segment].
    pub fn compose(&self, seg: Segment) -> Composition {
        let mut hasher = XxHash64::with_seed(0);
        thread_rng().next_u64().hash(&mut hasher);
        self.compose_with_seed(seg, hasher.finish())
    }
    /// Generates a [`Composition`] from a starting [Segment], using a seed to to
    /// create a reproducible output.
    pub fn compose_with_seed(&self, seg: Segment, seed: u64) -> Composition {
        info!(target: LOG, "Composing {:?} with seed {:?}.", seg, seed);
        debug!(target: LOG, "{:?}", self.options);
        let start_time = std::time::Instant::now();
        let options: CompositionOptions = self.options.into();
        let mut render_tree = Tree::new();
        let mut type_cache: Vec<HashSet<TypeId>> = Vec::new();
        let node_id = render_tree.insert(
            RenderSegment {
                rendered: false,
                seed,
                segment: seg,
                error: None,
            },
            None,
        );
        type_cache.insert(node_id, HashSet::default());

        // Nodes are rendered in depth-first order, meaning any children of a node will be rendered
        // before its siblings (assuming their required context is available). Nodes which cannot be
        // rendered (due to missing context) are skipped until their context dependencies are met.
        //
        // `render_stack` keeps track the (reverse) sequence of node ids to render, enabling this
        // depth-first ordering without having to do any element shifting.
        let mut render_stack = vec![0];
        loop {
            let mut added_node_count = 0;

            for render_stack_idx in (0_usize..render_stack.len()).rev() {
                let node_idx = render_stack[render_stack_idx];
                let is_top_of_render_stack = render_stack_idx + 1 == render_stack.len();

                // Already rendered nodes can be skipped (and removed if at the top of the render stack).
                if render_tree[node_idx].value.rendered {
                    if is_top_of_render_stack {
                        render_stack.pop();
                    }
                    continue;
                }

                let composition_context = CompositionContext::new(
                    &options,
                    &render_tree,
                    &render_tree[node_idx],
                    Some(&type_cache),
                );

                trace!(target: LOG, "Rendering: {:?}", &render_tree[node_idx]);
                let result = self
                    .engine
                    .render(&render_tree[node_idx].value.segment, composition_context);

                if let Some(render_res) = result {
                    match render_res {
                        // Case: Unable to render -- most commonly missing required context
                        // Later iterations will retry
                        crate::render::Result::Err(err) => {
                            trace!(target: LOG, "Rendering (Node idx: {:?}) was unsuccessful: {:?}",
                                &render_tree[node_idx].idx, err);
                            render_tree[node_idx].value.error = Some(err);
                        }
                        // Case: Successfully rendered
                        crate::render::Result::Ok(segments) => {
                            trace!(target: LOG, "Rendering (Node idx: {:?}) succeeded, producing \
                            {:?} children.", &render_tree[node_idx].idx, segments.len());

                            // Create an Rng used to generate seeds for rendered children
                            let mut hasher = XxHash64::default();
                            render_tree[node_idx].value.seed.hash(&mut hasher);
                            let mut rng = ChaCha12Rng::seed_from_u64(hasher.finish());

                            let children: Vec<RenderSegment> = segments
                                .into_iter()
                                .map(|s| RenderSegment {
                                    rendered: !self.engine.can_render(&*s.element),
                                    seed: match &s.name {
                                        None => {
                                            let mut hasher = XxHash64::default();
                                            rng.next_u64().hash(&mut hasher);
                                            hasher.finish()
                                        }
                                        Some(name) => {
                                            let mut hasher = XxHash64::default();
                                            render_tree[node_idx].value.seed.hash(&mut hasher);
                                            name.hash(&mut hasher);
                                            hasher.finish()
                                        }
                                    },
                                    segment: s,
                                    error: None,
                                })
                                .collect();

                            added_node_count += children.len();
                            let mut added_node_ids = vec![];

                            for child in children {
                                // Update the type cache (map of nodes and which other types of nodes they contain)
                                let type_ids = successors(Some(&*child.segment.element), |s| {
                                    s.wrapped_element()
                                })
                                .map(|s| s.as_any().type_id())
                                .collect::<HashSet<_>>();
                                for ancestor_idx in
                                    successors(Some(node_idx), |p_idx| render_tree[*p_idx].parent)
                                        .collect::<Vec<_>>()
                                {
                                    type_cache[ancestor_idx].extend(type_ids.iter().copied());
                                }

                                let node_id = render_tree.insert(child, Some(node_idx));
                                type_cache.insert(node_id, HashSet::default());
                                added_node_ids.push(node_id);
                            }

                            render_tree[node_idx].value.rendered = true;
                            render_tree[node_idx].value.error = None;

                            // Nodes are only rendered once so it can be removed if at the top of the stack.
                            // If not at the top, it will be removed at a later iteration (preventing
                            // unnecessary element shifting).
                            if is_top_of_render_stack {
                                render_stack.pop();
                            }
                            // Add the new node ids to the top of the render stack in reverse order
                            // (reverse order ensures they are rendered in the same order they were produced)
                            render_stack
                                .append(&mut added_node_ids.into_iter().rev().collect::<Vec<_>>());

                            // Breaking here ensures depth-first rendering by starting the iteration over
                            // from the top of the render_stack (which is where the newly added nodes are).
                            if added_node_count > 0 {
                                break;
                            }
                        }
                    }
                }
            }

            // If no nodes were added, no further progress can be made -- rendering complete.
            if added_node_count == 0 {
                break;
            }
        }

        let duration = std::time::Instant::now().duration_since(start_time);
        info!(target: LOG, "Finished composing. ({:?})", duration);

        if log_enabled!(target: LOG, Level::Warn) {
            render_tree
                .iter()
                .filter(|n| !n.value.rendered)
                .for_each(|n| warn!(target: LOG, "Unrendered: {:?}", n));
        }

        Composition {
            options: self.options.into(),
            tree: render_tree,
        }
    }
}

mod private {
    use crate::SegmentRef;

    pub trait Sealed {}

    impl<T: super::Element> Sealed for T {}
    impl<T: super::Element> Sealed for SegmentRef<'_, T> {}
}
