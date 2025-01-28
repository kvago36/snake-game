use rand::Rng;

use std::io::{self};

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::color::{BLACK, RED, GREEN};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::Button::{Keyboard};
use piston::event_loop::{EventSettings, Events};
use piston::{EventLoop, Key, PressEvent};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

const MATRIX_SIZE: usize = 27;

#[derive(Copy, Clone, Debug)]
enum Tie {
    Food,
    Particle,
}

enum Directions {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: u8,
    y: u8,
}

struct Board {
    field: [[Option<Tie>; MATRIX_SIZE]; MATRIX_SIZE],
}

impl Board {
    fn new() -> Board {
        Board {
            field: [[None; MATRIX_SIZE]; MATRIX_SIZE],
        }
    }

    fn update_board(&mut self, x: u8, y: u8, tie: Option<Tie>) {
        self.field[x as usize][y as usize] = tie;
    }

    fn generate_food(&mut self) -> Option<(u8, u8)> {
        let result = match self.find_empty_spot() {
            Some((row, col)) => {
                self.field[row as usize][col as usize] = Some(Tie::Food);
                Some((row, col))
            }
            None => None,
        };

        result
    }

    fn find_empty_spot(&self) -> Option<(u8, u8)> {
        let mut v = Vec::new();

        for row in 0..MATRIX_SIZE {
            for col in 0..MATRIX_SIZE {
                if let None = self.field[row][col] {
                    v.push((row as u8, col as u8))
                }
            }
        }

        if v.len() == 0 {
            return None;
        }

        let mut rng = rand::rng();

        let random_spot_index = rng.random_range(0..=v.len());

        Some(v[random_spot_index])
    }
}

struct Game {
    board: Board,
    is_finished: bool,
    direction: Directions,
    coordinates: Vec<Point>,
    snake_size: usize,
    is_paused: bool,
    food: Point,
}

impl Game {
    fn new() -> Game {
        let mut board = Board::new();

        board.update_board(6, 6, Some(Tie::Food));

        Game {
            board,
            is_finished: false,
            is_paused: false,
            direction: Directions::LEFT,
            coordinates: vec![Point{x:12, y: 12}, Point{x:13, y:12}, Point{x:14, y:12}],
            snake_size: 3,
            food: Point{x: 6, y: 6},
        }
    }

    fn pause(&mut self) {
        self.is_paused = !self.is_paused;
    }
    fn change_direction(&mut self, direction: Directions) {
        match self.direction {
            Directions::UP => {
                match direction {
                    Directions::LEFT => self.direction = Directions::LEFT,
                    Directions::RIGHT => self.direction = Directions::RIGHT,
                    _ => {}
                }
            }
            Directions::DOWN => {
                match direction {
                    Directions::LEFT => self.direction = Directions::LEFT,
                    Directions::RIGHT => self.direction = Directions::RIGHT,
                    _ => {}
                }
            }
            Directions::LEFT => {
                match direction {
                    Directions::UP => self.direction = Directions::UP,
                    Directions::DOWN => self.direction = Directions::DOWN,
                    _ => {}
                }
            }
            Directions::RIGHT => {
                match direction {
                    Directions::UP => self.direction = Directions::UP,
                    Directions::DOWN => self.direction = Directions::DOWN,
                    _ => {}
                }
            }
        }
    }

    fn eat_food(&mut self) {
        assert!(self.coordinates.len() > 1);

        let point = self.coordinates.last().unwrap();

        let new_tail = match self.direction {
            Directions::UP => Point{x: point.x, y: point.y - 1},
            Directions::DOWN => Point{x: point.x, y: point.y + 1},
            Directions::LEFT => Point{x: point.x + 1, y: point.y},
            Directions::RIGHT => Point{x: point.x - 1, y: point.y},
        };

        self.board.update_board(new_tail.x, new_tail.y, Some(Tie::Particle));

        self.snake_size += 1;
        self.coordinates.push(new_tail);

        if let Some((x, y)) = self.board.generate_food() {
            self.food = Point{x, y};
        } else {
            self.is_finished = true;
        }
    }

    fn generate_food(&mut self) {
        if let Some((x, y)) = self.board.generate_food() {
            self.food = Point{x, y};
        } else {
            self.is_finished = true;
        }
    }

    fn update_position(&mut self) {}
}

pub struct App {
    gl: GlGraphics,
    game: Game,
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const SIZE: f64 = 10.0;

        let food = rectangle::square(self.game.food.x as f64 * SIZE, self.game.food.y as f64 * SIZE, SIZE);
        let snake: Vec<_> = self.game.coordinates.iter().map(
            |p| rectangle::square(p.x as f64 * SIZE, p.y as f64 * SIZE, SIZE)
        ).collect();

        let snake_head = self.game.coordinates.first().unwrap();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            for square in &snake {
                rectangle(
                    RED,
                    *square,
                    c.transform,
                    gl,
                );
            }

            let snake_head = rectangle::square(
                snake_head.x as f64 * SIZE,
                snake_head.y as f64 * SIZE,
            SIZE);

            rectangle(BLACK, snake_head, c.transform, gl);

            rectangle(RED, food, c.transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        let head = self.game.coordinates.first().unwrap();

        let new_head = match self.game.direction {
            Directions::UP => {
                if head.y == 0 {
                    Point { x: head.x, y: MATRIX_SIZE as u8 - 1 }
                } else {
                    Point { x: head.x, y: head.y - 1 }
                }
            }
            Directions::DOWN => {
                if head.y == MATRIX_SIZE as u8 - 1 {
                    Point { x: head.x, y: 0 }
                } else {
                    Point { x: head.x, y: head.y + 1 }
                }
            }
            Directions::LEFT => {
                if head.x == 0 {
                    Point { x: MATRIX_SIZE as u8 - 1, y: head.y }
                } else {
                    Point { x: head.x - 1, y: head.y }
                }
            }
            Directions::RIGHT => {
                if head.x == MATRIX_SIZE as u8 - 1 {
                    Point { x: 0, y: head.y }
                } else {
                    Point { x: head.x + 1, y: head.y }
                }
            }
        };

        self.game.coordinates.pop().unwrap();
        self.game.coordinates.insert(0, new_head);

        let last = self.game.coordinates.last().unwrap();

        self.game.board.update_board(last.x, last.y, None);

        match self.game.board.field[new_head.x as usize][new_head.y as usize] {
            Some(Tie::Food) => {
                println!("Food");
                self.game.eat_food();
            }
            Some(Tie::Particle) => {
                println!("end");
                self.game.is_finished = true;
            }
            _ => {}
        }

        self.game.board.update_board(new_head.x, new_head.y, Some(Tie::Particle));
    }
}

fn main() -> io::Result<()> {
    let opengl = OpenGL::V3_2;

    // Create a Glutin window.
    let mut window: Window = WindowSettings::new("spinning-square", [270, 270])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        game: Game::new(),
    };

    let mut events = EventSettings::new();

    events.set_ups(3);

    let mut events = Events::new(events);
    while let Some(e) = events.next(&mut window) {
        if app.game.is_finished {
            break;
        }

        if let Some(Keyboard(key)) = e.press_args() {
            // println!("Key pressed: {:?}", key);

            match key {
                Key::W | Key::Up => app.game.change_direction(Directions::UP),
                Key::A | Key::Left => app.game.change_direction(Directions::LEFT),
                Key::S | Key::Down => app.game.change_direction(Directions::DOWN),
                Key::D | Key::Right => app.game.change_direction(Directions::RIGHT),
                Key::Space => app.game.pause(),
                _ => println!("Other key pressed"),
            }
        }

        if app.game.is_paused {
            continue;
        }

        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }

    println!("Hello, world!");

    Ok(())
}
