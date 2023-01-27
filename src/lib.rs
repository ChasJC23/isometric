use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::ops::Deref;
use std::rc::Rc;

use quick_xml::reader::Reader;
use quick_xml::writer::Writer;
use rand;
use config::Config;
use itertools::Itertools;

use crate::iter::ObjectSvgIter;
use crate::shapes::{Shape, Polygonal};
use crate::vector::{Vec2, Vec3};

#[cfg(test)] #[macro_use]
extern crate assert_matches;

pub mod shapes;
pub mod path;
pub mod parser;
pub mod vector;
pub mod iter;
pub mod num;

fn run<I: BufRead, O: Write>(mut reader: Reader<I>, mut writer: Writer<O>, settings: Config) {

    let shapes = parser::parse_shapes(&mut reader);
    let cube = shapes[255].unwrap();
    let (x_vec, y_vec, z_vec) = dimensions_from_cube(cube.borrow_mut().deref());

    let grid_size: Vec3<_> = settings.get::<(_, _, _)>("grid_size").unwrap().into();
    let mut grid = vec![vec![vec![0u8;grid_size.z];grid_size.y];grid_size.x];

    let connections = settings.get::<HashMap<String, Vec<(usize, usize, usize)>>>("equalities").unwrap();
    let connections: HashMap<String, Vec<Vec3<usize>>> = connections.iter().map(|pair| {
        let (key, arr) = pair;
        let arr = arr.iter().map(|e| Vec3::from(*e)).collect_vec();
        (key.clone(), arr)
    }).collect();

    for x in 0..grid_size.x {
        for y in 0..grid_size.y {
            for z in 0..grid_size.z {
                grid[x][y][z] = rand::random();
            }
        }
    }

    let (shapes, image_width, image_height) = get_objects(grid, shapes, x_vec, y_vec, z_vec, &connections.into_values().collect_vec());

    println!("{}", shapes.len());

    let light_vector = vect![0.3, 0.7, 0.5].normalise();
    let scene_colour = vect![0.6, 0.2, 0.9];

    for event in ObjectSvgIter::from_vec(&shapes, image_width, image_height, &light_vector, &scene_colour) {
        writer.write_event(event).expect("TODO: panic message");
    }
}

fn get_objects(grid: Vec<Vec<Vec<u8>>>, shapes: [Option<Rc<RefCell<Shape>>>; 256], x_vec: Vec2<f64>, y_vec: Vec2<f64>, z_vec: Vec2<f64>, connections: &[Vec<Vec3<usize>>]) -> (Vec<Rc<RefCell<Shape>>>, f64, f64) {

    // TODO: should probably put this elsewhere huh
    let cube = shapes[255].unwrap().borrow_mut();
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

                    let find_connection = | connections: &[Vec<_>], endpoint | {
                        for connection in connections {
                            if connection.contains(&endpoint) {
                                return Some(connection);
                            }
                        };
                        None
                    };

                    // why did I do this to myself? I literally just made it harder for me
                    // to interpret now that it's been >3 months since I wrote this
                    let mut shape_cell = {
                        if let Some(connection) = find_connection(connections, vect![x, y, z]) {
                            'a: {
                                for (existing_shape, pos) in to_draw {
                                    if connection.contains(&pos) {
                                        match existing_shape {
                                            Some(s) => break 'a s.clone(),
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
                    let mut shape = shape_cell.borrow_mut();

                    // the centre of the shape might not be the same as the centre of the encapsulating cube
                    let offset = (shape.centre() - centre_reference + shape_size / 2.0) % shape_size - shape_size / 2.0;

                    shape.move_to(centre + offset);

                    for (opt_old_shape, old_pos) in &mut to_draw {
                        match opt_old_shape {
                            Some(old_shape) => {
                                let mut delete_this = false;
                                old_shape.replace_with(|s| match s.del_if_obscured_by(shape.deref()) {
                                    Some(s) => s,
                                    None => {
                                        delete_this = true;
                                        Shape::new(vec![])
                                    },
                                });
                                if delete_this {
                                    *opt_old_shape = None
                                }
                            },
                            None => (),
                        }
                    }

                    to_draw.push((Some(shape_cell), vect![x, y, z]));
                }
            }
        }
    }

    (
        to_draw.iter()
            .map(|e| e.0.clone())
            .filter(|e| e.is_some())
            .map(|e| e.unwrap())
            .collect(),
        board_width,
        board_height
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
            },
            vectp![-0.001..=0.001, 0.999..=1.001, -0.001..=0.001] => {
                // green plane, positive y, top side
                h_g = -component.height();
            },
            vectp![0.999..=1.001, -0.001..=0.001, -0.001..=0.001] => {
                // red plane, positive x, right side
                x_vec.x = component.width();
                h_r = -component.height();
            },
            _ => (),
        }
    }

    // no unary plus :(
    x_vec.y = (-h_r - h_g + h_b) / 2.0;
    y_vec.y = ( h_r - h_g + h_b) / 2.0;
    z_vec.y = ( h_r - h_g - h_b) / 2.0;

    (x_vec, y_vec, z_vec)
}
