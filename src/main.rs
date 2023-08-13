#![windows_subsystem = "windows"]

use std::f32::consts::TAU;

use macroquad::{
    prelude::{is_key_pressed, Circle, Color, KeyCode, Rect, Vec2, BLACK, BLUE, GRAY, GREEN, RED},
    shapes::{draw_circle, draw_rectangle_lines},
    text::{draw_text_ex, TextParams},
    time::get_fps,
    window::{clear_background, next_frame, screen_height, screen_width, Conf},
};

use rand::Rng;

const WINDOW_HEIGHT: f32 = 1024.0;
const WINDOW_WIDTH: f32 = 1024.0;

const NUMBER_OF_POINTS: f32 = 2000.0;
const POINT_RADIUS: f32 = 5.0;
const POINT_COLOR: Color = RED;

const QUADTREE_CAPACITY: f32 = 30.0;
const RANDOM_WALK_DISTANCE: f32 = 10.0;

#[derive(PartialEq, Copy, Clone, Debug)]
struct Particle {
    pos: Vec2,
    color: Color,
    radius: f32,
}

impl Particle {
    fn update_pos(&mut self, new_pos: Vec2) {
        self.pos = new_pos;
    }

    fn itersects(&self, other: &Particle) -> bool {
        (self.pos.x - other.pos.x) * (self.pos.x - other.pos.x)
            + (self.pos.y - other.pos.y) * (self.pos.y - other.pos.y)
            < (self.radius + other.radius) * (self.radius + other.radius)
    }
}

struct QuadTree {
    capacity: f32,
    is_full: bool,
    boundary: Rect,
    data: Vec<Particle>,
    northeast: Option<Box<QuadTree>>,
    northwest: Option<Box<QuadTree>>,
    southeast: Option<Box<QuadTree>>,
    southwest: Option<Box<QuadTree>>,
}

impl QuadTree {
    pub fn new(capacity: f32, boundary: Rect) -> QuadTree {
        return QuadTree {
            capacity: capacity,
            is_full: false,
            boundary: boundary,
            data: Vec::new(),
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
        };
    }

    pub fn insert(&mut self, point: Particle) {
        if self.is_full {
            if self.northeast.is_some() {
                let northeast = self.northeast.as_mut().unwrap();
                if northeast.contains(&point) {
                    northeast.insert(point);
                    return;
                }
            }
            if self.northwest.is_some() {
                let northwest = self.northwest.as_mut().unwrap();
                if northwest.contains(&point) {
                    northwest.insert(point);
                    return;
                }
            }
            if self.southeast.is_some() {
                let southeast = self.southeast.as_mut().unwrap();
                if southeast.contains(&point) {
                    southeast.insert(point);
                    return;
                }
            }
            if self.southwest.is_some() {
                let southwest = self.southwest.as_mut().unwrap();
                if southwest.contains(&point) {
                    southwest.insert(point);
                    return;
                }
            }
            return;
        }

        if self.data.len() as f32 >= self.capacity {
            self.is_full = true;
            let x = self.boundary.x;
            let y = self.boundary.y;
            let w_2 = self.boundary.clone().w / 2.0;
            let h_2 = self.boundary.clone().h / 2.0;

            self.northeast = Some(Box::new(QuadTree::new(
                self.capacity,
                Rect::new(x + w_2, y, w_2, h_2),
            )));
            self.northwest = Some(Box::new(QuadTree::new(
                self.capacity,
                Rect::new(x, y, w_2, h_2),
            )));
            self.southeast = Some(Box::new(QuadTree::new(
                self.capacity,
                Rect::new(x + w_2, y + h_2, w_2, h_2),
            )));
            self.southwest = Some(Box::new(QuadTree::new(
                self.capacity,
                Rect::new(x, y + h_2, w_2, h_2),
            )));

            // for index in 0..self.data.len() {
            //     self.insert(self.data[index]);
            // }
            return;
        }

        self.data.push(point);
    }

    fn contains(&self, point: &Particle) -> bool {
        self.boundary.contains(point.pos)
    }

    fn query(&self, range: Circle) -> Vec<Particle> {
        let mut res = Vec::new();

        if !range.overlaps_rect(&self.boundary) {
            return res;
        }

        for p in self.data.iter() {
            if range.contains(&p.pos) {
                res.push(p.clone());
            }
        }

        if let Some(v) = &self.northwest {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.northeast {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.southwest {
            res.extend(v.query(range));
        }
        if let Some(v) = &self.southeast {
            res.extend(v.query(range));
        }

        res
    }

    pub fn display(&mut self, thickness: f32, color: Color) {
        draw_rectangle_lines(
            self.boundary.x,
            self.boundary.y,
            self.boundary.w,
            self.boundary.h,
            thickness,
            color,
        );
        if self.is_full {
            self.northeast.as_mut().unwrap().display(thickness, color);
            self.northwest.as_mut().unwrap().display(thickness, color);
            self.southeast.as_mut().unwrap().display(thickness, color);
            self.southwest.as_mut().unwrap().display(thickness, color);
        }
    }
}

fn generate_random_points(number_of_points: f32) -> Vec<Particle> {
    let mut points: Vec<Particle> = Vec::new();
    let mut rng = rand::thread_rng();

    let mut rand_x = screen_width() / 2.0;
    let mut rand_y = screen_height() / 2.0;

    for _ in 0..number_of_points as i32 {
        points.push(Particle {
            pos: Vec2 {
                x: rand_x,
                y: rand_y,
            },
            color: POINT_COLOR,
            radius: POINT_RADIUS,
        });
        rand_x =
            (rand_x + rng.gen_range(-RANDOM_WALK_DISTANCE..RANDOM_WALK_DISTANCE) + screen_width())
                % screen_width();
        rand_y =
            (rand_y + rng.gen_range(-RANDOM_WALK_DISTANCE..RANDOM_WALK_DISTANCE) + screen_width())
                % screen_width();
    }
    points
}

fn draw_points(points: &Vec<Particle>) {
    for point in points {
        draw_circle(point.pos.x, point.pos.y, point.radius, point.color);
    }
}

fn move_points(points: &mut Vec<Particle>) {
    let mut rng = rand::thread_rng();

    for point in points {
        let angle = rng.gen_range(0.0..TAU);
        point.update_pos(Vec2 {
            x: (point.pos.x + RANDOM_WALK_DISTANCE * angle.cos() + screen_width()) % screen_width(),
            y: (point.pos.y + RANDOM_WALK_DISTANCE * angle.sin() + screen_height())
                % screen_height(),
        });
    }
}

// fn check_overlap(points: &mut Vec<Particle>) {
//     let len = points.len();
//     'outer: for index_1 in 0..len {
//         for index_2 in 0..len {
//             if index_1 != index_2 {
//                 if points[index_1].itersects(&points[index_2]) {
//                     points[index_1].color = BLUE;
//                     points[index_2].color = BLUE;
//                     continue 'outer;
//                 }
//             }
//             points[index_1].color = RED;
//         }
//     }
// }

fn check_overlap(points: &mut Vec<Particle>, quadtree: QuadTree) {
    for index in 0..points.len() {
        let overlap = quadtree.query(Circle {
            x: points[index].pos.x,
            y: points[index].pos.y,
            r: 2.0 * points[index].radius,
        });
        if overlap.len() > 1 {
            points[index].color = BLUE;
            // for index_2 in 0..overlap.len() {
            //     overlap[index_2].color = BLUE;
            // }
        } else {
            points[index].color = RED;
        }
    }
}

fn build_quadtree(points: &mut Vec<Particle>) -> QuadTree {
    let mut quadtree = QuadTree::new(
        QUADTREE_CAPACITY,
        Rect::new(0.0, 0.0, screen_width(), screen_height()),
    );
    for index in 0..points.len() {
        quadtree.insert(points[index]);
    }
    quadtree
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Quadtree Visualizer".to_owned(),
        window_width: WINDOW_WIDTH as i32,
        window_height: WINDOW_HEIGHT as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut points = generate_random_points(NUMBER_OF_POINTS);

    loop {
        clear_background(GRAY);

        if is_key_pressed(KeyCode::Space) {
            points = generate_random_points(NUMBER_OF_POINTS);
        }

        move_points(&mut points);

        let mut quadtree = build_quadtree(&mut points);
        quadtree.display(4.0, GREEN);

        // check_overlap(&mut points);
        check_overlap(&mut points, quadtree);

        draw_points(&points);

        let fps_text = format!("{}", get_fps());
        draw_text_ex(
            &fps_text,
            screen_width() as f32 - 45.0,
            screen_height() as f32 - 14.0,
            TextParams {
                font_size: 30u16,
                color: BLACK,
                ..Default::default()
            },
        );

        // let quadtree_text = format!("{:?}", quadtree.boundary);
        // draw_text_ex(
        //     &quadtree_text,
        //     45.0,
        //     WINDOW_HEIGHT as f32 - 30.0,
        //     TextParams {
        //         font_size: 30u16,
        //         color: BLACK,
        //         ..Default::default()
        //     },
        // );

        next_frame().await
    }
}

// fps counter
// let fps_text = format!("{}", get_fps());
// draw_text_ex(
//     &fps_text,
//     WINDOW_WIDTH as f32 - 45.0,
//     WINDOW_HEIGHT as f32 - 14.0,
//     TextParams {
//         font_size: 30u16,
//         color: BLACK,
//         ..Default::default()
//     },
// );
