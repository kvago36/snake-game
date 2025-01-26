use rand::Rng;
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use std::io::{self, Write};

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use graphics::color::{BLACK, RED, GREEN};
use graphics::types::{Rectangle, Scalar};
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;

#[derive(Copy, Clone)]
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

#[derive(Clone, Copy)]
struct Point {
    x: u8,
    y: u8,
}

struct Board {
    field: [[Option<Tie>; 27]; 27],
}

impl Board {
    fn new() -> Board {
        Board {
            field: [[None; 27]; 27],
        }
    }

    fn update_board(&mut self, x: u8, y: u8, tie: Tie) {
        self.field[x as usize][y as usize] = Some(tie);
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

        for row in 0..9 {
            for col in 0..9 {
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
    food: Point,
}

impl Game {
    fn new() -> Game {
        Game {
            board: Board::new(),
            is_finished: false,
            direction: Directions::LEFT,
            coordinates: vec![Point{x:12, y: 12}, Point{x:13, y:12}, Point{x:14, y:12}],
            snake_size: 3,
            food: Point{x: 6, y: 6},
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

        self.board.update_board(new_tail.x, new_tail.y, Tie::Particle);

        self.snake_size += 1;
        self.coordinates.push(new_tail);
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

        let food = rectangle::square(self.game.food.x as f64, self.game.food.y as f64, SIZE);
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
        // rotation: 0.0,
    };

    // Включаем "сырой режим" для терминала
    enable_raw_mode()?;
    println!("Нажмите клавиши (q для выхода):");

    loop {
        // Проверяем, есть ли событие ввода
        if event::poll(std::time::Duration::from_millis(500))? {
            // Считываем событие
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('q') => {
                        println!("Выход из программы.");
                        break;
                    }
                    KeyCode::Char(c) => println!("Нажата клавиша: {}", c),
                    KeyCode::Up => println!("Вверх ↑"),
                    KeyCode::Down => println!("Вниз ↓"),
                    KeyCode::Left => println!("Влево ←"),
                    KeyCode::Right => println!("Вправо →"),
                    _ => println!("Другая клавиша: {:?}", key_event.code),
                }
            }
        }
    }

    // Отключаем "сырой режим"
    disable_raw_mode()?;

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        // if let Some(args) = e.update_args() {
        //     app.update(&args);
        // }
    }

    println!("Hello, world!");

    Ok(())
}
