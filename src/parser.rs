use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{BufRead};
use std::rc::Rc;

use lazy_static::lazy_static;
use quick_xml;
use quick_xml::events::{BytesStart, Event};
use regex::{Regex};

use crate::iter::{PrimitiveIter};
use crate::shapes::{Shape, ShapeComponent};
use crate::vector::Vec3;

lazy_static!{
    static ref COLOUR_REGEX: Regex = Regex::new(r"fill:#(?P<r>[\d|a-f]{2})(?P<g>[\d|a-f]{2})(?P<b>[\d|a-f]{2})").unwrap();
}

#[cfg(test)] #[allow(illegal_floating_point_literal_pattern)]
mod tests {
    use quick_xml::reader::Reader;
    use quick_xml::events::BytesStart;
    use crate::parser::{parse_component, parse_shapes};
    use crate::shapes::{Shape, ShapeComponent, ShapePrimitive};
    use crate::vector::{Vec2, Vec3};
    use crate::vectp;

    #[test]
    fn test_parse_shapes() {
        let mut reader = Reader::from_str(r#"
        <g inkscape:label="11111111">
        <path style="fill:#80ff80" d="m 46 33 19 5 v -19 l -14 -15 -13 14 z M 11 59 32 45 h -9 L 16 30 v 4 z" />
        <path style="fill:#008080" d="M 68 40 79 54 52 64 32 15 59 40 Z" />
        </g>"#);
        let table = parse_shapes(&mut reader);
        let parsed = table.get(&255).unwrap().as_ref();
        assert_matches!(parsed, ref shape if matches!(**shape, Shape {
            ref components
        } if matches!(&**components, [
            ShapeComponent {
                normal: vectp![0.0, 1.0, 0.0],
                primitives: ref first_primitives,
            },
            ShapeComponent {
                normal: vectp![0.0, 0.0, -1.0],
                primitives: ref second_primitives,
            },
        ] if matches!(**first_primitives, [
            ShapePrimitive {
                points: ref first_points
            },
            ShapePrimitive {
                points: ref second_points
            }
        ] if matches!(**first_points, [
            vectp![46.0, 33.0],
            vectp![65.0, 38.0],
            vectp![65.0, 19.0],
            vectp![51.0,  4.0],
            vectp![38.0, 18.0],
        ]) && matches!(**second_points, [
            vectp![11.0, 59.0],
            vectp![32.0, 45.0],
            vectp![23.0, 45.0],
            vectp![16.0, 30.0],
            vectp![16.0, 34.0],
        ])) && matches!(**second_primitives, [
            ShapePrimitive {
                points: ref third_points,
            }
        ] if matches!(**third_points, [
            vectp![68.0, 40.0],
            vectp![79.0, 54.0],
            vectp![52.0, 64.0],
            vectp![32.0, 15.0],
            vectp![59.0, 40.0],
        ])))))
    }
    #[test]
    fn test_parse_component_abs() {
        let mut event = BytesStart::new("path");
        event.push_attribute(("d", "M 46 33 65 38 V 19 L 51 4 38 18 Z"));
        event.push_attribute(("style", "fill:#80ff80"));
        let parsed = parse_component(event);
        assert_matches!(parsed, ShapeComponent {
            normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
            ref primitives,
        } if matches!(**primitives, [
            ShapePrimitive {
                ref points
            }
        ] if matches!(**points, [
            Vec2 { x: 46.0, y: 33.0 },
            Vec2 { x: 65.0, y: 38.0 },
            Vec2 { x: 65.0, y: 19.0 },
            Vec2 { x: 51.0, y:  4.0 },
            Vec2 { x: 38.0, y: 18.0 },
        ])));
    }
    #[test]
    fn test_parse_component_rel() {
        let mut event = BytesStart::new("path");
        event.push_attribute(("d", "m 46 33 19 5 v -19 l -14 -15 -13 14 z"));
        event.push_attribute(("style", "fill:#80ff80"));
        let parsed = parse_component(event);
        assert_matches!(parsed, ShapeComponent {
            normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
            ref primitives,
        } if matches!(**primitives, [
            ShapePrimitive {
                ref points
            }
        ] if matches!(**points, [
            Vec2 { x: 46.0, y: 33.0 },
            Vec2 { x: 65.0, y: 38.0 },
            Vec2 { x: 65.0, y: 19.0 },
            Vec2 { x: 51.0, y:  4.0 },
            Vec2 { x: 38.0, y: 18.0 },
        ])));
    }
    #[test]
    fn test_parse_component_multiple() {
        let mut event = BytesStart::new("path");
        event.push_attribute(("d", "m 46 33 19 5 v -19 l -14 -15 -13 14 z M 11 59 32 45 h -9 L 16 30 v 4 z"));
        event.push_attribute(("style", "fill:#80ff80"));
        let parsed = parse_component(event);
        assert_matches!(parsed, ShapeComponent {
            normal: vectp![0.0, 1.0, 0.0],
            ref primitives,
        } if matches!(**primitives, [
            ShapePrimitive {
                points: ref first_points
            },
            ShapePrimitive {
                points: ref second_points
            }
        ] if matches!(**first_points, [
            vectp![46.0, 33.0],
            vectp![65.0, 38.0],
            vectp![65.0, 19.0],
            vectp![51.0,  4.0],
            vectp![38.0, 18.0],
        ]) && matches!(**second_points, [
            vectp![11.0, 59.0],
            vectp![32.0, 45.0],
            vectp![23.0, 45.0],
            vectp![16.0, 30.0],
            vectp![16.0, 34.0],
        ])));
    }
}

pub fn parse_shapes<T: BufRead>(reader: &mut quick_xml::reader::Reader<T>) -> HashMap<u8, Rc<Shape>> {

    let mut buffer = Vec::new();

    let mut shapes: HashMap<u8, _> = HashMap::new();

    let mut groups = vec![];
    let mut components = vec![];

    loop {
        match reader.read_event_into(&mut buffer) {
            Err(e) =>
                panic!("Error at position {}: {}", reader.buffer_position(), e),

            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) if e.name().as_ref() == b"g" =>
                groups.append(parse_group(e).as_mut()),

            Ok(Event::Empty(e)) if e.name().as_ref() == b"path" => {
                let component = parse_component(e);
                components.push(component);
            },

            Ok(Event::End(e)) if e.name().as_ref() == b"g" => {
                let shape = Shape {
                    components,
                };
                let shape = Rc::new(shape);
                for group in groups {
                    shapes.insert(group, Rc::clone(&shape));
                }
                groups = vec![];
                components = vec![];
            },
            _ => (),
        }
    }

    shapes
}

fn parse_group(e: BytesStart) -> Vec<u8> {
    let mut group_name: Option<Cow<[u8]>> = None;
    for attr in e.attributes().with_checks(false) {
        let attr = attr.unwrap();
        if attr.key.as_ref() == b"inkscape:label" {
            group_name = Some(attr.value);
            break;
        }
    }
    let group_name = String::from_utf8(Vec::from(group_name.unwrap())).unwrap();
    let mut groups = vec![];
    for bit_string in group_name.split(';') {
        let group_num = u8::from_str_radix(bit_string, 2).unwrap();
        groups.push(group_num);
    }
    groups
}

fn parse_component(e: BytesStart) -> ShapeComponent {

    let mut normal = None;
    let mut primitives = None;
    
    for attr in e.attributes() {
        let attr = attr.unwrap();
        match attr.key.as_ref() {
            b"d" => {
                let path = String::from_utf8(Vec::from(attr.value.as_ref())).unwrap();
                let primitives_iter = PrimitiveIter::from_str(&path);
                primitives = Some(primitives_iter.collect());
            },
            b"style" => {
                let style_str = String::from_utf8(Vec::from(attr.value.as_ref())).unwrap();
                let caps = &COLOUR_REGEX.captures(&style_str).unwrap();

                let r = (i32::from_str_radix(&caps["r"], 16).unwrap() - 128) as f64;
                let g = (i32::from_str_radix(&caps["g"], 16).unwrap() - 128) as f64;
                let b = (i32::from_str_radix(&caps["b"], 16).unwrap() - 128) as f64;

                let magnitude = f64::sqrt(r * r + g * g + b * b);

                // accidentally got my dimensions the wrong way round
                normal = Some(Vec3 {
                    x: b / magnitude,
                    y: g / magnitude,
                    z: r / magnitude
                });
            },
            _ => ()
        };
    }
    if let (Some(normal), Some(primitives)) = (normal, primitives) {
        ShapeComponent {
            normal,
            primitives,
        }
    }
    else {
        panic!("Something fucked up");
    }
}
