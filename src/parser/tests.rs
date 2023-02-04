#![cfg(test)]
#![allow(illegal_floating_point_literal_pattern)]

use quick_xml::events::BytesStart;
use crate::parser::{parse_component};
use crate::shapes::{ShapeComponent, ShapePrimitive};
use crate::vector::{Vec2, Vec3};
use crate::vectp;

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