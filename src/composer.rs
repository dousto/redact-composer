use std::{fmt::Debug, collections::HashMap};

#[derive(Debug, PartialEq)]
pub enum SegmentType {
    Abstract { abstract_type: String, name: Option<String>, data: HashMap<String, String> },
    Instrument { program: i8 },
    PlayNote { note: u8, velocity: u8 }
}

#[derive(Debug, PartialEq)]
pub struct CompositionSegment {
    pub segment_type: SegmentType,
    pub begin: u8,
    pub end: u8,
}
#[derive(Debug, PartialEq)]
pub struct RenderSegment {
    pub segment: CompositionSegment,
    pub rendered: bool,
}

pub enum RenderResult {
    Success { segments: Option<Vec<CompositionSegment>>, data: Option<HashMap<String, String>> },
    MissingContext
}

pub trait Renderer {
    fn render(&self,
        abstract_type: &String, name: &Option<String>, data: &HashMap<String, String>,
        begin: u8, end: u8,
        context: &CompositionContext
    ) -> RenderResult;
}

#[derive(Debug)]
pub struct Node<T> {
    pub idx: usize,
    pub value: T,
    pub parent: Option<usize>,
    pub children: Vec<usize>
}

pub struct Composer<'a> {
    pub renderer: &'a dyn Renderer
}

impl<'a> Composer<'a> {
    pub fn compose(&self, seg: CompositionSegment) -> () {
        let mut render_nodes = vec![
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
                    SegmentType::Abstract { abstract_type, name, data  } => {

                        let composition_context = CompositionContext {
                            tree: &render_nodes[..],
                            start: &render_nodes[idx],
                        };

                        // &render_nodes[idx].value.segment
                        let result = self.renderer.render(
                            abstract_type, name, data,
                            render_nodes[idx].value.segment.begin, render_nodes[idx].value.segment.end,
                            &composition_context
                        );

                        match result {
                            RenderResult::Success { segments, data: new_data } => {
                                match segments {
                                    Some(segs) => {
                                        let inserts: Vec<RenderSegment> = segs.into_iter()
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
                                            render_nodes[idx].value.rendered = true;
                                        }
                                    },
                                    None => (),
                                }

                                match new_data {
                                    Some(new_data) => {
                                        match &mut render_nodes[idx].value.segment.segment_type {
                                            SegmentType::Abstract { abstract_type: _, name: _, data } => {
                                                println!("{:?}", data);
                                                data.clear();
                                                data.extend(new_data.into_iter());
                                            },
                                            _ => ()
                                        }
                                    },
                                    None => (),
                                }

                                println!("{:?}", render_nodes[idx]);
                                rendered_node_count += 1;
                            },
                            RenderResult::MissingContext => todo!(),
                        }
                    },
                    _ => println!("{:?}", render_nodes[idx])
                    
                };
            }

            println!("Rendered {:?} nodes.", rendered_node_count);
            if rendered_node_count <= 0 { break; }
        }

        println!("{:?}", render_nodes)
    }
}

// Type used during the render of abstract CompositionSegments which allows lookup of data from other composition tree nodes.
// 
// `tree` is a slice snapshot of the current composition tree
// `start` is the node being rendered. Lookups are relative to this node.
pub struct CompositionContext<'a> {
    tree: &'a [Node<RenderSegment>],
    start: &'a Node<RenderSegment>
}

impl<'a> CompositionContext<'a> {
    /// Look up the deepest CompositionSegment matching `abstract_type` node whose (begin, end) bounds wholly contains the `start` node.
    pub fn get(&self, abstract_type: &String) -> Option<&'a CompositionSegment> {
        let mut node_iters: Vec<usize> = vec![0];
        let mut matching_segment: Option<&'a CompositionSegment> = None;
        
        while !node_iters.is_empty() {
            node_iters = node_iters.into_iter().filter(|idx| {
                let render_segment = &self.tree[*idx].value;
                render_segment.rendered
                && render_segment.segment.begin <= self.start.value.segment.begin
                && render_segment.segment.end >= self.start.value.segment.end
            }).collect();

            for idx in &node_iters {
                match &self.tree[*idx].value.segment.segment_type {
                    SegmentType::Abstract { abstract_type: ctx_type, .. } => {
                        if ctx_type == abstract_type {
                            matching_segment = Some(&self.tree[*idx].value.segment)
                        }
                    },
                    _ => ()
                }
            }

            node_iters = node_iters.iter().flat_map(|idx| &self.tree[*idx].children).map(|t| *t).collect();
        }

        matching_segment
    }
}
