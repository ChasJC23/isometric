use std::collections::VecDeque;
use lazy_static::lazy_static;
use regex::{CaptureMatches, Regex};
use quick_xml::events::{Event, BytesStart, BytesEnd};

use crate::path::{Command, CommandType};
use crate::shapes::{Shape, ShapePrimitive};
use crate::vector::{Vec2, Vec3};

lazy_static! {
    static ref PATH_REGEX: Regex = Regex::new(r"(?i)(?P<cmd>[MVHLZ])\s*(?P<nums>(([+-]?\d+\.?\d*(E\d+)?)(\s|,)?)*)").unwrap();
}

pub struct ObjectSvgIter<'a> {
    shape_iter: Box<dyn Iterator<Item=&'a Shape> + 'a>,
    width: f64,
    height: f64,
    event_stack: Vec<Event<'a>>,
    path_iter: Option<Box<dyn Iterator<Item=Event<'a>> + 'a>>,
    // thought I could get away with this not being a pointer as it implements clone,
    // but the iterators can outlive self, meaning this also needs to be able to outlive self.
    // If I find a way to reference the lifetime of self, that would be awesome.
    light_vector: &'a Vec3<f64>,
    object_colour: &'a Vec3<f64>,
}

impl<'a> ObjectSvgIter<'a> {
    pub fn from_vec(shapes: &'a Vec<Shape>, width: f64, height: f64, light_vector: &'a Vec3<f64>, object_colour: &'a Vec3<f64>) -> ObjectSvgIter<'a> {
        let shape_iter = Box::new(shapes.iter());
        let mut result = ObjectSvgIter {
            event_stack: vec![],
            path_iter: None,
            shape_iter,
            light_vector,
            object_colour,
            width, height,
        };
        result.add_svg_tags();
        result
    }

    fn add_svg_tags(&mut self) {
        let mut start_bytes = BytesStart::new("svg");
        let width = self.width.to_string();
        let height = self.height.to_string();
        start_bytes.push_attribute(("width", width.as_str()));
        start_bytes.push_attribute(("height", height.as_str()));
        start_bytes.push_attribute(("version", "1.1"));
        start_bytes.push_attribute(("xmlns","http://www.w3.org/2000/svg"));
        let start_svg = Event::Start(start_bytes);
        let end_svg = Event::End(BytesEnd::new("svg"));
        self.event_stack.push(end_svg);
        self.event_stack.push(start_svg);
    }
}
impl<'a> Iterator for ObjectSvgIter<'a> {
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.event_stack.pop();
        if let Some(path_iter) = &mut self.path_iter {
            if let Some(next_path) = path_iter.next() {
                self.event_stack.push(next_path)
            }
            else {
                self.path_iter = None;
            }
        } else if let Some(current_shape) = self.shape_iter.next() {
            self.path_iter = Some(Box::new(current_shape.components.iter().map(|component| component.generate_path(*self.light_vector, *self.object_colour))));
            let group_start = Event::Start(BytesStart::new("g"));
            let group_end = Event::End(BytesEnd::new("g"));
            self.event_stack.push(group_end);
            self.event_stack.push(group_start);
        }
        result
    }
}

pub struct ToDStringIter<'a> {
    command_iter: ToSvgCommandIter<'a>,
    char_queue: VecDeque<char>,
}
impl<'a> ToDStringIter<'a> {
    pub fn from_vec(points: &'_ Vec<Vec2<f64>>) -> ToDStringIter {
        ToDStringIter {
            command_iter: ToSvgCommandIter::from_vec(points),
            char_queue: VecDeque::new(),
        }
    }
}
impl<'a> Iterator for ToDStringIter<'a> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.char_queue.len() == 0 {
            if let Some(command) = self.command_iter.next() {
                self.char_queue.push_back(command.cmd_type.to_opcode());
                for param in command.params {
                    let str_repr = param.to_string();
                    for char in str_repr.chars() {
                        self.char_queue.push_back(char);
                    }
                    self.char_queue.push_back(' ');
                }
            }
        }
        self.char_queue.pop_front()
    }
}

pub struct ToSvgCommandIter<'a> {
    points_iter: Box<dyn Iterator<Item=Vec2<f64>> + 'a>,
    first: bool,
    last_point: Vec2<f64>,
    current_point: Vec2<f64>,
    closed: bool,
    finished: bool,
}
impl<'a> ToSvgCommandIter<'a> {
    pub fn from_vec(points: &'_ Vec<Vec2<f64>>) -> ToSvgCommandIter {
        ToSvgCommandIter {
            points_iter: Box::new(points.iter().cloned()),
            first: true,
            last_point: Vec2 { x: 0.0, y: 0.0 },
            current_point: Vec2 { x: 0.0, y: 0.0 },
            closed: false,
            finished: false,
        }
    }
}
impl<'a> Iterator for ToSvgCommandIter<'a> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        // bodged myself into a corner with this one huh
        if let Some(mut next_point) = self.points_iter.next() {
            if self.first {
                self.first = false;
                self.finished = true;
                self.current_point = next_point;
                let mut params = vec![next_point.x, next_point.y];
                while let Some(next_point) = self.points_iter.next() {
                    self.last_point = self.current_point;
                    self.current_point = next_point;
                    if self.last_point.x == self.current_point.x || self.last_point.y == self.current_point.y {
                        self.finished = false;
                        break;
                    }
                    params.push(next_point.x);
                    params.push(next_point.y);
                }
                Some(Command { cmd_type: CommandType::MoveToAbs, params })
            }
            else if self.current_point.x == self.last_point.x {
                let mut params = vec![self.current_point.y];
                while next_point.x == self.current_point.x {
                    params.push(next_point.y);
                    self.last_point = self.current_point;
                    self.current_point = next_point;
                    next_point = if let Some(next_point) = self.points_iter.next() {
                        next_point
                    } else {
                        self.finished = true;
                        break;
                    }
                }
                self.last_point = self.current_point;
                self.current_point = next_point;
                Some(Command { cmd_type: CommandType::VertAbs, params })
            }
            else if self.current_point.y == self.last_point.y {
                let mut params = vec![self.current_point.x];
                while next_point.y == self.current_point.y {
                    params.push(next_point.x);
                    self.last_point = self.current_point;
                    self.current_point = next_point;
                    next_point = if let Some(next_point) = self.points_iter.next() {
                        next_point
                    } else {
                        self.finished = true;
                        break;
                    }
                }
                self.last_point = self.current_point;
                self.current_point = next_point;
                Some(Command { cmd_type: CommandType::HorizAbs, params })
            }
            else {
                let mut params = vec![self.current_point.x, self.current_point.y];
                while next_point.x != self.current_point.x || next_point.y != self.current_point.y {
                    params.push(next_point.x);
                    params.push(next_point.y);
                    self.last_point = self.current_point;
                    self.current_point = next_point;
                    next_point = if let Some(next_point) = self.points_iter.next() {
                        next_point
                    } else {
                        self.finished = true;
                        break;
                    }
                }
                self.last_point = self.current_point;
                self.current_point = next_point;
                Some(Command { cmd_type: CommandType::LineToAbs, params })
            }
        }
        else {
            if self.closed {
                None
            }
            else if self.finished {
                self.closed = true;
                Some(Command { cmd_type: CommandType::ClosePath, params: vec![] })
            }
            else if self.current_point.x == self.last_point.x {
                self.finished = true;
                Some(Command { cmd_type: CommandType::VertAbs, params: vec![self.current_point.y] })
            }
            else if self.current_point.y == self.last_point.y {
                self.finished = true;
                Some(Command { cmd_type: CommandType::HorizAbs, params: vec![self.current_point.x] })
            }
            else {
                self.finished = true;
                Some(Command { cmd_type: CommandType::LineToAbs, params: vec![self.current_point.x, self.current_point.y] })
            }
        }
    }
}

pub struct FromSvgCommandIter<'r, 't> {
    capture_matches: CaptureMatches<'r, 't>
}
impl<'r, 't> FromSvgCommandIter<'r, 't> {
    pub fn from_str(s: &'t str) -> FromSvgCommandIter<'r, 't> {
        FromSvgCommandIter { capture_matches: PATH_REGEX.captures_iter(s) }
    }
}
impl<'r, 't> Iterator for FromSvgCommandIter<'r, 't> {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.capture_matches.next();
        if let Some(captures) = next {
            let command = CommandType::from_opcode(&captures["cmd"]);
            let numbers = captures["nums"].split_terminator(&[',', ' '][..]);
            let numbers = numbers.map(|num| {
                if let Ok(gen_num) = num.parse::<f64>() {
                    gen_num
                } else {
                    panic!("'{}' could not be converted to a float", num);
                }
            });
            Some(Command { cmd_type: command, params: numbers.collect() })
        }
        else {
            None
        }
    }
}

pub struct SvgPointIter<'r, 't> {
    command_iter: FromSvgCommandIter<'r, 't>,
    current_point: Vec2<f64>,
    start_point: Vec2<f64>,
    current_command: Option<Command>,
    pointer: usize,
    implicit_lineto: bool,
    ret: bool,
}
impl<'r, 't> SvgPointIter<'r, 't> {
    pub fn from_str(s: &'t str) -> SvgPointIter<'r, 't> {
        let mut command_iter = FromSvgCommandIter::from_str(s);
        SvgPointIter {
            current_command: command_iter.next(),
            command_iter,
            current_point: Vec2 { x: 0.0, y: 0.0 },
            start_point: Vec2 { x: 0.0, y: 0.0 },
            pointer: 0,
            implicit_lineto: false,
            ret: false,
        }
    }
}
impl<'r, 't> Iterator for SvgPointIter<'r, 't> {
    type Item = (Vec2<f64>, bool);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(command) = &self.current_command {
            self.ret = false;
            match command.cmd_type {
                CommandType::MoveToAbs => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point = Vec2 { x, y };
                    if !self.implicit_lineto {
                        self.start_point = self.current_point;
                        self.implicit_lineto = true;
                    }
                }
                CommandType::MoveToRel => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point += (x, y);
                    if !self.implicit_lineto {
                        self.start_point = self.current_point;
                        self.implicit_lineto = true;
                    }
                }
                CommandType::LineToAbs => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point = Vec2 { x, y };
                }
                CommandType::LineToRel => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point += (x, y);
                }
                CommandType::VertAbs => {
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point.y = y;
                }
                CommandType::VertRel => {
                    let y = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point.y += y;
                }
                CommandType::HorizAbs => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point.x = x;
                }
                CommandType::HorizRel => {
                    let x = command.params[self.pointer];
                    self.pointer += 1;
                    self.current_point.x += x;
                }
                CommandType::ClosePath => {
                    self.current_point = self.start_point;
                    self.ret = true;
                }
            };
            if self.pointer == command.params.len() {
                self.current_command = self.command_iter.next();
                self.pointer = 0;
                self.implicit_lineto = false;
            }
            Some((self.current_point, self.ret))
        } else {
            None
        }
    }
}

pub struct PrimitiveIter<'r, 't> {
    point_iter: SvgPointIter<'r, 't>,
}
impl<'r, 't> PrimitiveIter<'r, 't> {
    pub fn from_str(s: &'t str) -> PrimitiveIter<'r, 't> {
        let point_iter = SvgPointIter::from_str(s);
        PrimitiveIter {
            point_iter,
        }
    }
}
impl<'r, 't> Iterator for PrimitiveIter<'r, 't> {
    type Item = ShapePrimitive;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = vec![];
        let mut next = self.point_iter.next();
        if next.is_none() {
            return None;
        }
        while let Some((pt, ret)) = next {
            if ret {
                break;
            }
            result.push(pt);
            next = self.point_iter.next();
        }
        Some(ShapePrimitive { points: result })
    }
}
