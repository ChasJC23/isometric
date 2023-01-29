use crate::vector::{Vec2, Vec3};
use itertools::Itertools;
use crate::iter::ToDStringIter;
use crate::vect;

#[cfg(test)]
mod tests {
    use std::ops::Neg;
    use itertools::assert_equal;
    use crate::shapes::{contains, obscures, Polygonal, Shape, ShapeComponent, ShapePrimitive};
    use crate::vect;
    use crate::vector::{Vec2, Vec3};

    fn rot90<T: Neg<Output=T> + Copy>(v: Vec2<T>) -> Vec2<T> {
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
    fn test_contains() {
        let shape = gen_square(1.0);
        // a square contains its centre
        assert!( contains(&shape, Vec2 { x: 0.0, y: 0.0 }));
        // a square contains its boundary
        assert!( contains(&shape, Vec2 { x: 1.0, y: 0.0 }));
        // check opposite boundary, where there exists the possibility of two intersections
        assert!( contains(&shape, Vec2 { x: -1.0, y: 0.0 }));
        // check points outside the boundaries of the square
        let mut point = Vec2 { x: 2.0, y: 0.0 };
        for _ in 0..4 {
            assert!(!contains(&shape, point));
            point = rot90(point);
        }
    }
    #[test]
    fn test_contains_parallel() {
        let shape = gen_square(1.0);
        // parallel edge cases
        assert!( contains(&shape, Vec2 { x: 0.0, y: 1.0 }));
        assert!( contains(&shape, Vec2 { x: 0.0, y: -1.0 }));
    }
    #[test]
    fn test_contains_virtual_boundary() {
        // place virtual edge on the vector path
        let shape = ShapePrimitive { points: vec![
            Vec2 { x: 1.0, y: 1.0 },
            Vec2 { x: 1.0, y:-1.0 },
            Vec2 { x:-1.0, y:-1.0 },
            Vec2 { x:-1.0, y: 1.0 },
        ] };
        // a square contains its centre
        assert!( contains(&shape, Vec2 { x: 0.0, y: 0.0 }));
        // a square contains its boundary
        assert!( contains(&shape, Vec2 { x: 1.0, y: 0.0 }));
        // check opposite boundary, where there exists the possibility of two intersections
        assert!( contains(&shape, Vec2 { x: -1.0, y: 0.0 }));
        // checking virtual line again just in case
        assert!(!contains(&shape, Vec2 { x: -2.0, y: 0.0 }));
    }
    #[test]
    fn test_contains_corner() {
        let shape = gen_45square(1.0);
        // sanity check
        assert!( contains(&shape, Vec2 { x: 0.0, y: 0.5 }));
        assert!(!contains(&shape, Vec2 { x:-1.0, y: 0.5 }));
        assert!(!contains(&shape, Vec2 { x: 1.0, y: 0.5 }));

        // check line intersecting right corner
        assert!( contains(&shape, Vec2 { x: 0.0, y: 0.0 }));
        assert!( contains(&shape, Vec2 { x: 1.0, y: 0.0 }));
        assert!( contains(&shape, Vec2 { x:-1.0, y: 0.0 }));
        assert!(!contains(&shape, Vec2 { x:-2.0, y: 0.0 }));

        // check line intersecting top corner
        assert!( contains(&shape, Vec2 { x: 0.0, y: 1.0 }));
        assert!(!contains(&shape, Vec2 { x:-1.0, y: 1.0 }));
    }

    #[test]
    fn primitive_lines_iter() {
        let primitive = ShapePrimitive { points: vec![
            Vec2 { x:-1.5, y: 2.2 },
            Vec2 { x:-1.9, y: 0.0 },
            Vec2 { x:-1.0, y:-2.5 },
            Vec2 { x: 2.7, y:-1.1 },
            Vec2 { x: 1.0, y: 1.0 },
        ] };
        assert_equal(primitive.lines_iter(), vec![
            (Vec2 { x:-1.5, y: 2.2 }, Vec2 { x:-1.9, y: 0.0 }),
            (Vec2 { x:-1.9, y: 0.0 }, Vec2 { x:-1.0, y:-2.5 }),
            (Vec2 { x:-1.0, y:-2.5 }, Vec2 { x: 2.7, y:-1.1 }),
            (Vec2 { x: 2.7, y:-1.1 }, Vec2 { x: 1.0, y: 1.0 }),
            (Vec2 { x: 1.0, y: 1.0 }, Vec2 { x:-1.5, y: 2.2 })
        ]);
    }
    #[test]
    fn component_lines_iter() {
        let component = ShapeComponent {
            normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
            primitives: vec![
            ShapePrimitive { points: vec![
                Vec2 { x:-1.5, y: 2.2 },
                Vec2 { x:-1.9, y: 0.0 },
                Vec2 { x:-1.0, y:-2.5 },
                Vec2 { x: 2.7, y:-1.1 },
                Vec2 { x: 1.0, y: 1.0 },
            ] },
            ShapePrimitive { points: vec![
                Vec2 { x: 1.3, y: 4.4 },
                Vec2 { x: 2.7, y:-0.7 },
                Vec2 { x:-3.3, y: 0.3 },
                Vec2 { x:-1.3, y: 2.1 },
                Vec2 { x:-0.7, y: 4.4 },
            ] },
        ] };
        assert_equal(component.lines_iter(), vec![
            (Vec2 { x:-1.5, y: 2.2 }, Vec2 { x:-1.9, y: 0.0 }),
            (Vec2 { x:-1.9, y: 0.0 }, Vec2 { x:-1.0, y:-2.5 }),
            (Vec2 { x:-1.0, y:-2.5 }, Vec2 { x: 2.7, y:-1.1 }),
            (Vec2 { x: 2.7, y:-1.1 }, Vec2 { x: 1.0, y: 1.0 }),
            (Vec2 { x: 1.0, y: 1.0 }, Vec2 { x:-1.5, y: 2.2 }),
            (Vec2 { x: 1.3, y: 4.4 }, Vec2 { x: 2.7, y:-0.7 }),
            (Vec2 { x: 2.7, y:-0.7 }, Vec2 { x:-3.3, y: 0.3 }),
            (Vec2 { x:-3.3, y: 0.3 }, Vec2 { x:-1.3, y: 2.1 }),
            (Vec2 { x:-1.3, y: 2.1 }, Vec2 { x:-0.7, y: 4.4 }),
            (Vec2 { x:-0.7, y: 4.4 }, Vec2 { x: 1.3, y: 4.4 })
        ]);
    }
    #[test]
    fn shape_lines_iter() {
        let shape = Shape {
            components: vec![
                ShapeComponent {
                    normal: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    primitives: vec![
                    ShapePrimitive { points: vec![
                        Vec2 { x:-1.5, y: 2.2 },
                        Vec2 { x:-1.9, y: 0.0 },
                        Vec2 { x:-1.0, y:-2.5 },
                        Vec2 { x: 2.7, y:-1.1 },
                        Vec2 { x: 1.0, y: 1.0 },
                    ] },
                    ShapePrimitive { points: vec![
                        Vec2 { x: 1.3, y: 4.4 },
                        Vec2 { x: 2.7, y:-0.7 },
                        Vec2 { x:-3.3, y: 0.3 },
                        Vec2 { x:-1.3, y: 2.1 },
                        Vec2 { x:-0.7, y: 4.4 },
                    ] },
                ] },
                ShapeComponent {
                    normal: Vec3 { x: 1.0, y: 0.0, z: 0.0 },
                    primitives: vec![
                        ShapePrimitive { points: vec![
                            Vec2 { x: 5.4, y:-2.6 },
                            Vec2 { x: 3.0, y:-3.9 },
                            Vec2 { x: 5.4, y:-5.2 },
                            Vec2 { x: 6.3, y:-4.7 },
                            Vec2 { x: 2.9, y:-2.9 },
                        ] },
                    ],
                }
            ],
        };
        assert_equal(shape.lines_iter(), vec![
            (Vec2 { x:-1.5, y: 2.2 }, Vec2 { x:-1.9, y: 0.0 }),
            (Vec2 { x:-1.9, y: 0.0 }, Vec2 { x:-1.0, y:-2.5 }),
            (Vec2 { x:-1.0, y:-2.5 }, Vec2 { x: 2.7, y:-1.1 }),
            (Vec2 { x: 2.7, y:-1.1 }, Vec2 { x: 1.0, y: 1.0 }),
            (Vec2 { x: 1.0, y: 1.0 }, Vec2 { x:-1.5, y: 2.2 }),
            (Vec2 { x: 1.3, y: 4.4 }, Vec2 { x: 2.7, y:-0.7 }),
            (Vec2 { x: 2.7, y:-0.7 }, Vec2 { x:-3.3, y: 0.3 }),
            (Vec2 { x:-3.3, y: 0.3 }, Vec2 { x:-1.3, y: 2.1 }),
            (Vec2 { x:-1.3, y: 2.1 }, Vec2 { x:-0.7, y: 4.4 }),
            (Vec2 { x:-0.7, y: 4.4 }, Vec2 { x: 1.3, y: 4.4 }),
            (Vec2 { x: 5.4, y:-2.6 }, Vec2 { x: 3.0, y:-3.9 }),
            (Vec2 { x: 3.0, y:-3.9 }, Vec2 { x: 5.4, y:-5.2 }),
            (Vec2 { x: 5.4, y:-5.2 }, Vec2 { x: 6.3, y:-4.7 }),
            (Vec2 { x: 6.3, y:-4.7 }, Vec2 { x: 2.9, y:-2.9 }),
            (Vec2 { x: 2.9, y:-2.9 }, Vec2 { x: 5.4, y:-2.6 })
        ]);
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
        assert!(obscures(&shape, &shape));
        assert!(obscures(&shape, &rotated));
        assert!(obscures(&rotated, &shape));
        let shape = gen_45square(1.0);
        assert!(obscures(&shape, &shape));
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
}

fn contains(a: &impl Polygonal, p: Vec2<f64>) -> bool {
    let mut direction: Vec2<f64> = vect![1.0, 0.0];
    let mut intersections = 0;
    let Some(mut sp_0) = a.points_iter().last() else {
        return false;
    };
    for (sp_1, sp_2) in a.lines_iter() {
        let edge = sp_2 - sp_1;
        let prev_edge = sp_1 - sp_0;
        let Vec2 { x: mut lambda, y: mut mu } = intersection_parameters(sp_1, edge, p, direction);
        // this will happen if the direction we choose is parallel to the line we want to check against.
        // Easiest way around it is just try again in a different direction!
        if lambda.is_nan() || mu.is_nan() {
            direction = vect![0.0, 1.0];
            Vec2 { x: lambda, y: mu } = intersection_parameters(sp_1, edge, p, direction);
        }
        // explicitly include the boundary
        if 0.0 <= lambda && lambda <= 1.0 && mu == 0.0 {
            return true;
        }
        if (
            0.0 < lambda && lambda < 1.0 ||
            // if we intersect a corner, use the cross product to see if we actually go through it
            lambda == 0.0 && Vec2::cross(prev_edge, direction).signum() == Vec2::cross(edge, direction).signum()
        ) && mu > 0.0
        {
            intersections += 1;
        }
        sp_0 = sp_1;
    }
    (intersections & 1) == 1
}
fn obscures(a: &impl Polygonal, b: &impl Polygonal) -> bool {
    for point in b.points_iter() {
        if !contains(a, point) {
            return false;
        }
    }
    true
}

pub trait Polygonal {
    fn points_iter(&self) -> Box<dyn Iterator<Item=Vec2<f64>> + '_>;
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item=&mut Vec2<f64>> + '_>;
    fn lines_iter(&self) -> Box<dyn Iterator<Item=(Vec2<f64>, Vec2<f64>)> + '_>;
    fn left(&self) -> f64 {
        self.points_iter().map(|p| p.x).reduce(f64::min).unwrap()
    }
    fn right(&self) -> f64 {
        self.points_iter().map(|p| p.x).reduce(f64::max).unwrap()
    }
    fn top(&self) -> f64 {
        self.points_iter().map(|p| p.y).reduce(f64::min).unwrap()
    }
    fn bottom(&self) -> f64 {
        self.points_iter().map(|p| p.y).reduce(f64::max).unwrap()
    }
    fn shift(&mut self, offset: Vec2<f64>) {
        self.points_iter_mut().for_each(|p| *p += offset);
    }
    fn width(&self) -> f64 {
        self.right() - self.left()
    }
    fn height(&self) -> f64 {
        self.bottom() - self.top()
    }
    fn centre(&self) -> Vec2<f64> {
        vect![self.left() + self.right(), self.top() + self.bottom()] / 2.0
    }
    fn move_to(&mut self, point: Vec2<f64>) {
        self.shift(point - self.centre())
    }
}

#[derive(Debug, Clone)]
pub struct ShapePrimitive {
    pub points: Vec<Vec2<f64>>,
}

impl Polygonal for ShapePrimitive {

    fn points_iter(&self) -> Box<dyn Iterator<Item=Vec2<f64>> + '_> {
        Box::new(self.points.iter().cloned())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item=&mut Vec2<f64>> + '_> {
        Box::new(self.points.iter_mut())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item=(Vec2<f64>, Vec2<f64>)> + '_> {
        Box::new(self.points.iter().cloned().circular_tuple_windows())
    }
}
impl ShapePrimitive {
    pub fn del_if_obscured_by(self, other: &impl Polygonal) -> Option<Self> {
        Some(self).del_if_obscured_by(other)
    }
    pub fn generate_d(&self) -> String {
        let iter = ToDStringIter::from_vec(&self.points);
        iter.collect()
    }
}

#[derive(Debug, Clone)]
pub struct ShapeComponent {
    pub normal: Vec3<f64>,
    pub primitives: Vec<ShapePrimitive>,
}

impl Polygonal for ShapeComponent {

    fn points_iter(&self) -> Box<dyn Iterator<Item=Vec2<f64>> + '_> {
        Box::new(self.primitives.iter().map(|p| p.points_iter()).flatten())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item=&mut Vec2<f64>> + '_> {
        Box::new(self.primitives.iter_mut().map(|p| p.points_iter_mut()).flatten())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item=(Vec2<f64>, Vec2<f64>)> + '_> {
        Box::new(self.primitives.iter().map(|p| p.lines_iter()).flatten())
    }
}
impl ShapeComponent {
    pub fn del_if_obscured_by(self, other: &impl Polygonal) -> Option<Self> {
        Some(self).del_if_obscured_by(other)
    }
    pub fn generate_d(&self) -> String {
        // I mean this works, but it can definitely be done better
        let mut result = String::new();
        for primitive in &self.primitives {
            result += &primitive.generate_d();
        }
        result
    }
    pub fn generate_path(&self, light_vector: Vec3<f64>, object_colour: Vec3<f64>) -> quick_xml::events::Event {
        let mut tag_bytes = quick_xml::events::BytesStart::new("path");
        let d = self.generate_d();
        tag_bytes.push_attribute(("d", d.as_str()));
        tag_bytes.push_attribute(("style", self.generate_css(light_vector, object_colour).as_str()));
        quick_xml::events::Event::Empty(tag_bytes)
    }
    fn generate_css(&self, light_vector: Vec3<f64>, object_colour: Vec3<f64>) -> String {
        let mut brightness = Vec3::dot(self.normal, light_vector);
        brightness = f64::max(brightness, 0.0);
        let object_colour = object_colour * brightness;
        // little bit funky but it works out fine
        let object_colour = object_colour * 256.0;
        format!("fill:#{:02x}{:02x}{:02x}", object_colour.x as u8, object_colour.y as u8, object_colour.z as u8)
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    components: Vec<ShapeComponent>,
}

impl Polygonal for Shape {
    fn points_iter(&self) -> Box<dyn Iterator<Item=Vec2<f64>> + '_> {
        Box::new(self.components.iter().map(|p| p.points_iter()).flatten())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item=&mut Vec2<f64>> + '_> {
        Box::new(self.components.iter_mut().map(|p| p.points_iter_mut()).flatten())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item=(Vec2<f64>, Vec2<f64>)> + '_> {
        Box::new(self.components.iter().map(|p| p.lines_iter()).flatten())
    }
}
impl Shape {
    pub fn new(components: Vec<ShapeComponent>) -> Shape {
        Shape {
            components
        }
    }
    pub fn component_iter(&self) -> impl Iterator<Item=&ShapeComponent> {
        self.components.iter()
    }
    pub fn del_if_obscured_by(self, other: &impl Polygonal) -> Option<Self> {
        Some(self).del_if_obscured_by(other)
    }
}

pub trait OptObscurable {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self;
}

impl OptObscurable for Option<Shape> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                let mut new_components = vec![];
                for component in s.components {
                    if let Some(new_component) = component.del_if_obscured_by(other) {
                        new_components.push(new_component);
                    }
                }
                if new_components.len() == 0 {
                    None
                }
                else {
                    let s = Shape { components: new_components };
                    Some(s)
                }
            },
            None => None,
        }
    }
}

impl OptObscurable for Option<&mut Shape> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                s.components = s.components.clone().into_iter()
                    .map(|c| Some(c).del_if_obscured_by(other))
                    .filter(|c| c.is_some())
                    .map(|c| c.unwrap())
                    .collect();

                if s.components.len() == 0 {
                    None
                }
                else {
                    Some(s)
                }
            },
            None => None,
        }
    }
}

impl OptObscurable for Option<ShapeComponent> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                let mut new_primitives = vec![];
                for primitive in s.primitives {
                    if let Some(new_primitive) = primitive.del_if_obscured_by(other) {
                        new_primitives.push(new_primitive);
                    }
                }
                if new_primitives.len() == 0 {
                    None
                }
                else {
                    let s = ShapeComponent { primitives: new_primitives, normal: s.normal };
                    Some(s)
                }
            },
            None => None,
        }
    }
}

impl OptObscurable for Option<&mut ShapeComponent> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                s.primitives = s.primitives.clone().into_iter()
                    .map(|p| Some(p).del_if_obscured_by(other))
                    .filter(|p| p.is_some())
                    .map(|p| p.unwrap())
                    .collect();

                if s.primitives.len() == 0 {
                    None
                }
                else {
                    Some(s)
                }
            },
            None => None,
        }
    }
}

impl OptObscurable for Option<ShapePrimitive> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                if obscures(other, &s) {
                    None
                } else {
                    Some(s)
                }
            },
            None => self,
        }
    }
}

impl OptObscurable for Option<&mut ShapePrimitive> {
    fn del_if_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(s) => {
                if obscures(other, s) {
                    None
                } else {
                    Some(s)
                }
            },
            None => self,
        }
    }
}

// game devs hmu
fn intersection_parameters(p_1: Vec2<f64>, d_1: Vec2<f64>, p_2: Vec2<f64>, d_2: Vec2<f64>) -> Vec2<f64> {
    let lambda = Vec2::cross(p_2 - p_1, d_2) / Vec2::cross(d_1, d_2);
    let mu = Vec2::cross(p_1 - p_2, d_1) / Vec2::cross(d_2, d_1);

    vect![lambda, mu]
}
