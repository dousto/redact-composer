use std::collections::HashMap;

use composer::{Renderer, CompositionContext, RenderResult};

use crate::composer::{CompositionSegment, SegmentType};
#[macro_use]

pub mod musical;
pub mod composer;

///
/// This will eventually be converted to a lib, but for the time being is used to easily run during development
///

fn main() {
    let renderer = TestRenderer {};
    let c = composer::Composer {
        renderer: &renderer
    };

    c.compose(CompositionSegment {
        begin: 0, end: 128,
        segment_type: SegmentType::Abstract { abstract_type: String::from("Song"), name: None, data: HashMap::new() }}
    );
}

// ------------------------------------------
// This stuff will be moved to a separate lib
// ------------------------------------------

pub struct TestRenderer;

impl Renderer for TestRenderer {

    fn render(&self,
        abstract_type: &String, _name: &Option<String>, _data: &HashMap<String, String>,
        _begin: u8, _end: u8,
        _context: &CompositionContext
    ) -> RenderResult {
        // let key = match &_context.get(&String::from("Chord")).unwrap().segment_type {
        //     SegmentType::Abstract { abstract_type: _, name: _, data } => {
        //         data.get(&String::from("key")).unwrap()
        //     },
        //     _ => panic!()
        // };

        match abstract_type.as_str() {
            "Song" => RenderResult::Success {
                segments: Some(vec![
                    CompositionSegment {
                        segment_type: SegmentType::Abstract {
                            abstract_type: String::from("Part"), name: Some(String::from("Part1")), data: HashMap::new()
                        },
                        begin: 0, end: 0
                    },
                    CompositionSegment {
                            segment_type: SegmentType::Abstract {
                                abstract_type: String::from("Part"), name: Some(String::from("Part2")), data: HashMap::new()
                            },
                            begin: 0, end: 0
                    },
                ]),
                data: None
            },
            "Part" => RenderResult::Success {
                segments: Some(vec![
                    CompositionSegment {
                        segment_type: SegmentType::Instrument { program: 0 },
                        begin: 0, end: 0
                        }
                ]),
                data: None
            },
            _ => RenderResult::Success { segments: None, data: None }
        }
    }
}



#[cfg(test)]
mod tests {

    #[test]
    fn create_tree() {
    }
}