use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{BufRead, Write};
use std::ops::Deref;
use std::rc::Rc;

use config::Config;
use itertools::Itertools;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

use crate::iter::object_svg_iter;
use crate::shapes::{Shape, Polygonal, OptObscurable, ShapePrimitive, ShapeComponent};
use crate::vector::{Vec2, Vec3};

#[cfg(test)]
#[macro_use]
extern crate assert_matches;

pub mod iter;
pub mod num;
pub mod parser;
pub mod path;
pub mod shapes;
pub mod vector;

pub fn run<I: BufRead, O: Write>(mut reader: Reader<I>, mut writer: Writer<O>, settings: Config) {
    
    let shapes = parser::parse_shapes(&mut reader);
    let cube = shapes[255].clone().unwrap();
    let (x_vec, y_vec, z_vec) = dimensions_from_cube(cube.borrow_mut().deref());

    let grid_size: Vec3<_> = settings.get::<(_, _, _)>("grid_size").unwrap().into();
    let mut grid = vec![vec![vec![0u8; grid_size.z]; grid_size.y]; grid_size.x];

    let tiles = settings.get::<Vec<(usize, usize, usize)>>("tiles").unwrap();

    for tile in tiles {
        grid[tile.0][tile.1][tile.2] = 255;
    }

    let connections = settings
        .get::<HashMap<String, Vec<(usize, usize, usize)>>>("equalities")
        .unwrap();
    let connections: HashMap<String, Vec<Vec3<usize>>> = connections.iter()
        .map(|pair| {
            let (key, arr) = pair;
            let arr = arr.iter().map(|e| Vec3::from(*e)).collect_vec();
            (key.clone(), arr)
        })
        .collect();

    let (shapes, image_width, image_height) = get_objects(grid, shapes, x_vec, y_vec, z_vec, &connections.into_values().collect_vec());

    // let shapes = combine_shapes(shapes);

    let light_vector = vect![0.3, 0.7, 0.5].normalise();
    let scene_colour = vect![0.6, 0.2, 0.9];

    for event in object_svg_iter(&shapes, image_width, image_height, light_vector, scene_colour) {
        writer.write_event(event).expect("TODO: panic message");
    }
}

fn combine_shapes(shapes: Vec<Shape>) -> Vec<Shape> {

    let components_iter = shapes.into_iter().map(|s| s.into_component_iter()).flatten();

    /*
    Primarily taken from https://stackoverflow.com/questions/39638363/how-can-i-use-a-hashmap-with-f64-as-key-in-rust
    For valid SVG input, this program will not encounter the floating point hellscape of infinities and NaNs.
    This should be perfectly fine, and even if it isn't, the side effects this would produce would be pretty easily identifiable.
    As said in the `dimensions_from_cube` function, the IEEE-754 standard requires that
    "Every NaN shall compare unordered with everything, including itself."
    If I were to expect NaNs, this would be a really serious problem! However, for a pet / terminal project
    like this, it's not the most serious concern. If someone were to sneak a NaN through
    the crude SVG parser in `parser.rs` or the `serde` and `config` crates; as far as I'm concerned,
    that's undefined behaviour. I don't mind if they crash the program or receive gibberish output.

    This problem is exactly the kind of problem introduced by strict type systems.
    This isn't saying "this is why C is the best language of all time",
    but it is something that should really be considered when designing strongly typed languages:
    * Should individual values of a type be considered in a type system?
    * If not, should they be considered in the case of enumerations?
    * If so, where's the tradeoff between compilation time and accuracy? Should I allow the type of all even numbers? How?
    */
    struct ScaryVector(f64, f64, f64);
    impl ScaryVector {
        fn key(&self) -> u64 {
            self.0.to_bits() ^ self.1.to_bits() ^ self.2.to_bits()
        }
    }
    impl Hash for ScaryVector {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.key().hash(state)
        }
    }
    impl PartialEq for ScaryVector {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1 == other.1 && self.2 == other.2
        }
    }
    impl Eq for ScaryVector {}
    impl From<Vec3<f64>> for ScaryVector {
        fn from(v: Vec3<f64>) -> Self {
            ScaryVector(v.x, v.y, v.z)
        }
    }
    impl From<ScaryVector> for Vec3<f64> {
        fn from(v: ScaryVector) -> Self {
            vect![v.0, v.1, v.2]
        }
    }
    impl Display for ScaryVector {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "Boo!")
        }
    }

    let mut primitives_hashmap: HashMap<ScaryVector, VecDeque<ShapePrimitive>> = HashMap::new();
    for component in components_iter {
        for primitive in component.primitives {
            match primitives_hashmap.get_mut(&component.normal.into()) {
                Some(vector) => {
                    vector.push_back(primitive);
                }
                None => {
                    primitives_hashmap.insert(component.normal.into(), {
                        let mut a = VecDeque::with_capacity(1);
                        a.push_back(primitive);
                        a
                    });
                }
            }
        }
    }

    for (_, queue) in &mut primitives_hashmap {
        fuse_faces(queue);
    }

    primitives_hashmap.into_iter()
        .map(|(vec, primitives)|
            Shape::new(vec![ShapeComponent {
                primitives: primitives.into(),
                normal: vec.into()
            }])
        ).collect()
}

fn fuse_faces(shapes: &mut VecDeque<ShapePrimitive>) {
    loop {
        let original_len = shapes.len();
        if original_len <= 1 { return; }
        let mut was_fused = false;
        let Some(current) = shapes.pop_front() else { return; };
        for shape in shapes.iter_mut() {
            match current.combine_common_edges(shape) {
                Some(fused) => {
                    *shape = fused;
                    was_fused = true;
                    break;
                }
                None => (),
            }
        }
        if !was_fused {
            shapes.push_back(current);
        }
        let final_len = shapes.len();
        if original_len == final_len {
            return;
        }
    }
}

fn get_objects(grid: Vec<Vec<Vec<u8>>>, shapes: [Option<Rc<RefCell<Shape>>>; 256], x_vec: Vec2<f64>, y_vec: Vec2<f64>, z_vec: Vec2<f64>, connections: &[Vec<Vec3<usize>>]) -> (Vec<Shape>, f64, f64) {

    // TODO: should probably put this elsewhere huh
    let cube = shapes[255].clone().unwrap();
    let cube = cube.borrow();
    let shape_size = vect![cube.width(), cube.height()];
    let centre_reference = cube.centre();

    let grid_size = vect![grid.len(), grid[0].len(), grid[0][0].len()];

    // the size of our projected board
    let board_width = grid_size.x as f64 * x_vec.x + grid_size.z as f64 * -z_vec.x;
    let board_height = grid_size.x as f64 * x_vec.y + grid_size.y as f64 * -y_vec.y + grid_size.z as f64 * z_vec.y;

    let origin = vect![
        grid_size.z as f64 * -z_vec.x,
        grid_size.y as f64 * -y_vec.y
    ];

    let mut to_draw: Vec<(Option<Rc<RefCell<Shape>>>, Vec3<usize>)> = vec![];

    for depth in 0..grid_size.x + grid_size.y + grid_size.z {
        for x in 0..usize::min(grid_size.x, depth + 1) {
            for y in 0..usize::min(grid_size.y, depth + 1 - x) {
                let z = depth - x - y;
                if z >= grid_size.z { continue; } // might do the maths to avoid this at some point
                let centre = origin + x_vec * x as f64 + y_vec * y as f64 + z_vec * z as f64;

                if let Some(shape) = &shapes[grid[x][y][z] as usize] {
                    let mut existing_connection = None;
                    let mut new_shape = true;

                    for connection in connections {
                        if connection.contains(&vect![x, y, z]) {
                            existing_connection = Some(connection);
                        }
                    }

                    let shape_cell = {
                        if let Some(connection) = existing_connection {
                            'a: {
                                for (existing_shape, pos) in &to_draw {
                                    if connection.contains(&pos) {
                                        match existing_shape {
                                            Some(s) => {
                                                new_shape = false;
                                                break 'a s.clone();
                                            },
                                            None => (),
                                        }
                                    }
                                }
                                Rc::new((**shape).clone())
                            }
                        }
                        else {
                            Rc::new((**shape).clone())
                        }
                    };

                    // This condition is here for "connected" shapes.
                    // I would check why this is necessary and fix it proper; but line-by-line debugging shows me
                    // the original copy of the shape is put in the right place, so this is good enough.
                    if new_shape {
                        let mut shape = shape_cell.borrow_mut();

                        // the centre of the shape might not be the same as the centre of the encapsulating cube
                        let offset = (shape.centre() - centre_reference + shape_size / 2.0) % shape_size - shape_size / 2.0;

                        shape.move_to(centre + offset);
                        drop(shape);
                    }

                    for (opt_old_shape_cell, _old_pos) in &mut to_draw {
                        let mut delete_this = false;
                        match opt_old_shape_cell {
                            Some(old_shape_cell) => {
                                let old_shape = &mut *old_shape_cell.borrow_mut();
                                let mut opt = Some(old_shape);
                                if old_shape_cell.as_ptr() == shape_cell.as_ptr() {
                                    // would be borrowing mutably in two places if this wasn't here!
                                    delete_this = true;
                                }
                                else {
                                    opt = opt.del_if_obscured_by(&*shape_cell.borrow());
                                    // opt = delete_the_stragglers(opt, &*shape_cell.borrow());
                                    delete_this = opt.is_none();
                                }
                            }
                            None => (),
                        }
                        if delete_this {
                            *opt_old_shape_cell = None;
                        }
                    }

                    to_draw.push((Some(shape_cell), vect![x, y, z]));
                }
            }
        }
    }

    (
        to_draw.into_iter()
            .map(|e| e.0.clone())
            .filter(|e| e.is_some())
            .map(|e| (*e.unwrap().borrow()).clone())
            .collect(),
        board_width,
        board_height,
    )
}

fn dimensions_from_cube(cube: &Shape) -> (Vec2<f64>, Vec2<f64>, Vec2<f64>) {
    
    // this information could be derived in a different way, but I'm not sure how to format supplying it...
    let mut x_vec = vect![0.0, 0.0];
    let mut y_vec = vect![0.0, 0.0];
    let mut z_vec = vect![0.0, 0.0];
    let (mut h_r, mut h_g, mut h_b) = (0.0, 0.0, 0.0);

    for component in cube.component_iter() {
        /*
        having read into https://github.com/rust-lang/rust/issues/41620 concerning these warnings,
        float comparisons are an absolute mess. I'm using ranges because I'm a good boy
        and I know my sources are only u8, so precision is ~1/256 when mapped to [0,1], half of which >0.001.

        getting the sneaky float shenanigans out the way (https://ieeexplore.ieee.org/document/8766229):
        5.11:
        * "Comparisons shall ignore the sign of zero (so +0 = -0)"
        * "Infinite operands of the same sign shall compare equal"
        * "Every NaN shall compare unordered with everything, including itself."
        So the equality relating to bit-strings is not the same as equality relating to floats.
        If this is a reason why your code breaks on version update, it probably deserves the ensuing refactor.

        As far as I can tell, the specification does define implementation independent requirements
        as long as they are fully supported (5.12.2). Nevertheless, a dec2bin conversion function
        is definitely not going to be injective if you can be bothered to type out 30 decimal places;
        and is absolutely not going to be surjective if you don't type out those digits which is 99% of the time.

        What's the easy solution for real numbers then? Use hexadecimal floating point syntax!
        C++ has it! (https://en.cppreference.com/w/cpp/language/floating_literal)
        No rounding is needed for few digits so there isn't anything funky in the conversion!
        Oh wait, Rust doesn't support that. (https://github.com/rust-lang/rust/issues/1433 + others)

        It claims to have been fixed at https://github.com/rust-lang/rust/pull/12652,
        however it's hidden away as a syntax extension in some hexfloat crate I have yet to find.
        Pretty much avoids the problem imo.
        */
        #[allow(illegal_floating_point_literal_pattern)]
        match component.normal {
            vectp![-0.001..=0.001, -0.001..=0.001, 0.999..=1.001] => {
                // blue plane, positive z, left side
                z_vec.x = -component.width();
                h_b = -component.height();
            }
            vectp![-0.001..=0.001, 0.999..=1.001, -0.001..=0.001] => {
                // green plane, positive y, top side
                h_g = -component.height();
            }
            vectp![0.999..=1.001, -0.001..=0.001, -0.001..=0.001] => {
                // red plane, positive x, right side
                x_vec.x = component.width();
                h_r = -component.height();
            }
            _ => (),
        }
    }

    // no unary plus :(
    x_vec.y = (-h_r - h_g + h_b) / 2.0;
    y_vec.y = ( h_r - h_g + h_b) / 2.0;
    z_vec.y = ( h_r - h_g - h_b) / 2.0;

    (x_vec, y_vec, z_vec)
}
