#![cfg(test)]

use std::ops::Neg;

use crate::shapes::{CircleDirection, Containment, get_containment, obscures, Polygonal, ShapePrimitive};
use crate::vect;
use crate::vector::Vec2;

fn rot90<T: Neg<Output = T> + Copy>(v: Vec2<T>) -> Vec2<T> {
    vect![-v.y, v.x]
}
fn gen_square(size: f64) -> ShapePrimitive {
    ShapePrimitive { points: vec![
        Vec2 { x: size, y: size },
        Vec2 { x:-size, y: size },
        Vec2 { x:-size, y:-size },
        Vec2 { x: size, y:-size },
    ] }
}
fn gen_45square(size: f64) -> ShapePrimitive {
    ShapePrimitive { points: vec![
        Vec2 { x: size, y: 0.0  },
        Vec2 { x: 0.0 , y: size },
        Vec2 { x:-size, y: 0.0  },
        Vec2 { x: 0.0 , y:-size },
    ] }
}
fn gen_90square(size: f64) -> ShapePrimitive {
    ShapePrimitive { points: vec![
        Vec2 { x: size, y: size },
        Vec2 { x: size, y:-size },
        Vec2 { x:-size, y:-size },
        Vec2 { x:-size, y: size },
    ] }
}

#[test]
fn test_combination() {
    let points = [
        vect![-1.7, 4.27],
        vect![-3.56, 2.54],
        vect![-2.46, -3.8],
        vect![0.59, -1.36],
        vect![2.65, -0.74],
        vect![0.5, 1.89],
        vect![1.0, 4.25],
        vect![4.89, 2.15],
        vect![4.41, -2.96],
    ];
    let s1 = ShapePrimitive { points: points[0..=6].to_vec() };
    let mut s2 = ShapePrimitive { points: points[2..=8].to_vec() };

    s2.points.reverse();

    let result = ShapePrimitive::combine_common_edges(&s1, &s2).unwrap();
    let expected = ShapePrimitive { points: vec![
        vect![-2.46, -3.8],
        vect![-3.56, 2.54],
        vect![-1.7, 4.27],
        vect![1.0, 4.25],
        vect![4.89, 2.15],
        vect![4.41, -2.96],
    ] };

    assert!(obscures(&result, &expected));
    assert!(obscures(&expected, &result));
}

#[test]
fn test_contains() {
    let shape = gen_square(1.0);
    // a square contains its centre
    assert!(get_containment(&shape, Vec2 { x: 0.0, y: 0.0 }) == Containment::Inside);
    // a square contains its boundary
    assert!(get_containment(&shape, Vec2 { x: 1.0, y: 0.0 }) == Containment::Edge);
    // check opposite boundary, where there exists the possibility of two intersections
    assert!(get_containment(&shape, Vec2 { x: -1.0, y: 0.0 }) == Containment::Edge);
    // check points outside the boundaries of the square
    let mut point = Vec2 { x: 2.0, y: 0.0 };
    for _ in 0..4 {
        assert!(get_containment(&shape, point) == Containment::Outside);
        point = rot90(point);
    }
}
#[test]
fn test_contains_parallel() {
    let shape = gen_square(1.0);
    // parallel edge cases
    assert!( get_containment(&shape, Vec2 { x: 0.0, y: 1.0 }) == Containment::Edge);
    assert!( get_containment(&shape, Vec2 { x: 0.0, y: -1.0 }) == Containment::Edge);
}
#[test]
fn test_contains_corner() {
    let shape = gen_45square(1.0);
    // sanity check
    assert!(get_containment(&shape, Vec2 { x: 0.0, y: 0.5 }) == Containment::Inside);
    assert!(get_containment(&shape, Vec2 { x:-1.0, y: 0.5 }) == Containment::Outside);
    assert!(get_containment(&shape, Vec2 { x: 1.0, y: 0.5 }) == Containment::Outside);

    // check line intersecting right corner
    assert!(get_containment(&shape, Vec2 { x: 0.0, y: 0.0 }) == Containment::Inside);
    assert!(get_containment(&shape, Vec2 { x: 1.0, y: 0.0 }) == Containment::Edge);
    assert!(get_containment(&shape, Vec2 { x:-1.0, y: 0.0 }) == Containment::Edge);
    assert!(get_containment(&shape, Vec2 { x:-2.0, y: 0.0 }) == Containment::Outside);

    // check line intersecting top corner
    assert!(get_containment(&shape, Vec2 { x: 0.0, y: 1.0 }) == Containment::Edge);
    assert!(get_containment(&shape, Vec2 { x:-1.0, y: 1.0 }) == Containment::Outside);
}

#[test]
fn test_obscures() {
    let inner = gen_45square(1.0);
    let outer = gen_45square(2.0);
    assert!( obscures(&outer, &inner));
    assert!(!obscures(&inner, &outer));
}
#[test]
fn test_obscures_self() {
    let shape = gen_square(1.0);
    let rotated = gen_90square(1.0);
    assert!( obscures(&shape, &shape));
    assert!( obscures(&shape, &rotated));
    assert!( obscures(&rotated, &shape));
    let shape = gen_45square(1.0);
    assert!( obscures(&shape, &shape));
}
#[test]
fn test_not_obscures() {
    let mut a = gen_45square(1.0);
    a.shift(Vec2 { x: 2.0, y: 0.0 });
    let mut b = gen_45square(1.0);
    b.shift(Vec2 { x: -2.0, y: 0.0 });
    assert!(!obscures(&a, &b));
    assert!(!obscures(&b, &a));
}
#[test]
fn test_partial_obscures() {
    let mut a = gen_45square(2.0);
    a.shift(Vec2 { x: 1.0, y: 0.0 });
    let mut b = gen_45square(2.0);
    b.shift(Vec2 { x: -1.0, y: 0.0 });
    assert!(!obscures(&a, &b));
    assert!(!obscures(&b, &a));
}
#[test]
fn test_orbit_direction() {
    let sq = gen_45square(2.0);
    assert!(sq.draw_direction() == CircleDirection::CounterClockwise)
}