use itertools::Itertools;

use crate::vector::{Vec2, Vec3};
use crate::iter::ToDStringIter;
use crate::{vect, vectp};

mod tests;

fn inclusive_contains(a: &impl Polygonal, p: Vec2<f64>) -> bool {
    match get_containment(a, p) {
        Containment::Outside => false,
        _ => true,
    }
}

fn exclusive_contains(a: &impl Polygonal, p: Vec2<f64>) -> bool {
    match get_containment(a, p) {
        Containment::Inside => true,
        _ => false,
    }
}

enum Containment {
    Inside,
    Edge,
    Outside,
}

fn get_containment(a: &impl Polygonal, p: Vec2<f64>) -> Containment {
    let mut direction = vect![1.0, 0.0];
    let mut intersections = 0;
    let Some(mut sp_0) = a.points_iter().last() else {
        return Containment::Outside;
    };
    for (sp_1, sp_2) in a.lines_iter() {
        let edge = sp_2 - sp_1;
        let prev_edge = sp_1 - sp_0;
        let vectp![mut lambda, mut mu] = intersection_parameters(sp_1, edge, p, direction);
        // this will happen if the direction we choose is parallel to the line we want to check against.
        // Easiest way around it is just try again in a different direction!
        if lambda.is_nan() || mu.is_nan() {
            direction = direction.rot(1.0);
            vect![lambda, mu] = intersection_parameters(sp_1, edge, p, direction);
        }
        // boundary
        if 0.0 <= lambda && lambda <= 1.0 && mu == 0.0 {
            return Containment::Edge;
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
    if (intersections & 1) == 1 {
        Containment::Inside
    }
    else {
        Containment::Outside
    }
}

fn obscures(a: &impl Polygonal, b: &impl Polygonal) -> bool {
    for point in b.points_iter() {
        if !inclusive_contains(a, point) {
            return false;
        }
    }
    true
}

pub trait Polygonal {

    fn points_iter(&self) -> Box<dyn Iterator<Item = Vec2<f64>> + '_>;
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Vec2<f64>> + '_>;
    fn lines_iter(&self) -> Box<dyn Iterator<Item = (Vec2<f64>, Vec2<f64>)> + '_>;
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

    fn points_iter(&self) -> Box<dyn Iterator<Item = Vec2<f64>> + '_> {
        Box::new(self.points.iter().cloned())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Vec2<f64>> + '_> {
        Box::new(self.points.iter_mut())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item = (Vec2<f64>, Vec2<f64>)> + '_> {
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
    pub fn combine_common_edges(&self, other: &ShapePrimitive) -> Option<ShapePrimitive> {

        let cmn1 = self.points.iter().cloned().enumerate().find_or_first(|(_, p)| other.points.contains(p));
        let Some((mut my_i1, mut cmn1)) = cmn1 else {
            return None;
        };
        if my_i1 == 0 {
            my_i1 = self.points.len() - 1;
            while other.points.contains(&self.points[my_i1]) {
                cmn1 = self.points[my_i1];
                my_i1 -= 1;
                if my_i1 == 0 {
                    return Some(self.clone());
                }
            }
            my_i1 = (my_i1 + 1) % self.points.len();
        }

        let mut my_i2 = (my_i1 + 1) & self.points.len();
        let mut cmn2 = self.points[my_i2];
        while other.points.contains(&self.points[my_i2]) {
            cmn2 = self.points[my_i2];
            my_i2 = (my_i2 + 1) % self.points.len();
            if my_i2 == my_i1 {
                return Some(self.clone());
            }
        }
        if my_i2 == 0 {
            my_i2 = self.points.len() - 1;
        }
        else {
            my_i2 -= 1;
        }

        let their_i1 = other.points.iter().cloned().enumerate().find_or_first(|(_, p)| *p == cmn1).unwrap().0;
        let their_i2 = other.points.iter().cloned().enumerate().find_or_first(|(_, p)| *p == cmn2).unwrap().0;

        let backwards = self.draw_direction() != other.draw_direction();

        let mut points = vec![self.points[my_i2]];
        let mut index = (my_i2 + 1) % self.points.len();

        #[derive(PartialEq)]
        enum Which {
            Me,
            Them,
        }
        let mut which = Which::Me;
        while index != my_i2 || which != Which::Me {
            match which {
                Which::Me => {
                    points.push(self.points[index]);
                    index = (index + 1) % self.points.len();
                    if index == my_i1 {
                        index = their_i1;
                        which = Which::Them;
                    }
                },
                Which::Them => {
                    points.push(other.points[index]);
                    if backwards {
                        if index == 0 {
                            index = other.points.len() - 1;
                        }
                        else {
                            index -= 1;
                        }
                    }
                    else {
                        index = (index + 1) % other.points.len();
                    }
                    if index == their_i2 {
                        index = my_i2;
                        which = Which::Me;
                    }
                },
            }
        }

        Some(ShapePrimitive { points })
    }
    fn draw_direction(&self) -> CircleDirection {
        let line_vectors: Vec<_> = self.points.iter().cloned().circular_tuple_windows().map(|(p1, p2)| p2 - p1).collect();
        let mut angle = 0.0;
        for (line1, line2) in line_vectors.into_iter().circular_tuple_windows::<(Vec2<f64>, Vec2<f64>)>() {
            angle += f64::asin(Vec2::cross(line1, line2) / (line1.magnitude() * line2.magnitude()));
        }
        if angle > 0.0 {
            CircleDirection::CounterClockwise
        }
        else {
            CircleDirection::Clockwise
        }
    }
}

#[derive(PartialEq)]
enum CircleDirection {
    Clockwise,
    CounterClockwise,
}

#[derive(Debug, Clone)]
pub struct ShapeComponent {
    // TODO: having everything in here public is *fine*, but should probably be changed at some point.
    pub normal: Vec3<f64>,
    pub primitives: Vec<ShapePrimitive>,
}

impl Polygonal for ShapeComponent {

    fn points_iter(&self) -> Box<dyn Iterator<Item = Vec2<f64>> + '_> {
        Box::new(self.primitives.iter().map(|p| p.points_iter()).flatten())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Vec2<f64>> + '_> {
        Box::new(self.primitives.iter_mut().map(|p| p.points_iter_mut()).flatten())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item = (Vec2<f64>, Vec2<f64>)> + '_> {
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
    pub fn generate_path<'a, 'b>(&'a self, light_vector: Vec3<f64>, object_colour: Vec3<f64>) -> quick_xml::events::Event<'b> {
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
    fn points_iter(&self) -> Box<dyn Iterator<Item = Vec2<f64>> + '_> {
        Box::new(self.components.iter().map(|p| p.points_iter()).flatten())
    }
    fn points_iter_mut(&mut self) -> Box<dyn Iterator<Item = &mut Vec2<f64>> + '_> {
        Box::new(self.components.iter_mut().map(|p| p.points_iter_mut()).flatten())
    }
    fn lines_iter(&self) -> Box<dyn Iterator<Item = (Vec2<f64>, Vec2<f64>)> + '_> {
        Box::new(self.components.iter().map(|p| p.lines_iter()).flatten())
    }
}
impl Shape {
    pub fn new(components: Vec<ShapeComponent>) -> Shape {
        Shape { components }
    }
    pub fn component_iter(&self) -> impl Iterator<Item = &ShapeComponent> {
        self.components.iter()
    }
    pub fn into_component_iter(self) -> impl Iterator<Item = ShapeComponent> {
        self.components.into_iter()
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
            }
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
            }
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
            }
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
            }
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
                }
                else {
                    Some(s)
                }
            }
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
                }
                else {
                    Some(s)
                }
            }
            None => self,
        }
    }
}

trait OptReducible {
    fn del_whats_obscured_by(self, other: &impl Polygonal) -> Self;
}

impl OptReducible for Option<ShapePrimitive> {
    fn del_whats_obscured_by(self, other: &impl Polygonal) -> Self {
        match self {
            Some(mut s) => {
                s.points = s.points.into_iter()
                    .filter(|p| exclusive_contains(other, *p))
                    .collect();
                if s.points.len() <= 2 {
                    None
                }
                else {
                    Some(s)
                }
            }
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
