use std::borrow::Cow;
use std::cell::RefCell;
use std::io::BufRead;
use std::rc::Rc;

use lazy_static::lazy_static;
use quick_xml;
use quick_xml::events::{BytesStart, Event};
use regex::Regex;

use crate::iter::PrimitiveIter;
use crate::shapes::{Shape, ShapeComponent};
use crate::vector::Vec3;

lazy_static!{
    static ref COLOUR_REGEX: Regex = Regex::new(r"fill:#(?P<r>[\d|a-f]{2})(?P<g>[\d|a-f]{2})(?P<b>[\d|a-f]{2})").unwrap();
}

mod tests;

pub fn parse_shapes<T: BufRead>(reader: &mut quick_xml::reader::Reader<T>) -> [Option<Rc<RefCell<Shape>>>; 256] {

    let mut buffer = Vec::new();

    const INIT: Option<Rc<RefCell<Shape>>> = None;
    let mut shapes = [INIT; 256];

    let mut groups = vec![];
    let mut components = vec![];

    loop {
        match reader.read_event_into(&mut buffer) {
            Err(e) => panic!("Error at position {}: {}", reader.buffer_position(), e),

            Ok(Event::Eof) => break,

            Ok(Event::Start(e)) if e.name().as_ref() == b"g" => {
                groups.append(parse_group(e).as_mut())
            }

            Ok(Event::Empty(e)) if e.name().as_ref() == b"path" => {
                let component = parse_component(e);
                components.push(component);
            }

            Ok(Event::End(e)) if e.name().as_ref() == b"g" => {
                let shape = Shape::new(components);
                let shape = Rc::new(RefCell::new(shape));
                for group in groups {
                    shapes[group as usize] = Some(Rc::clone(&shape));
                }
                groups = vec![];
                components = vec![];
            }
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
            }
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
                    z: r / magnitude,
                });
            }
            _ => (),
        };
    }
    if let (Some(normal), Some(primitives)) = (normal, primitives) {
        ShapeComponent {
            normal,
            primitives,
        }
    }
    else {
        panic!("Something went wrong...");
    }
}
