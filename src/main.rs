extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate texture;

use rand::{thread_rng, Rng};

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;

use std::path::Path;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 700;

const UPS: u64 = 120;

struct Coordinate {
    x: f64,
    y: f64,
}

enum Direction {
    Left,
    Right,
    Still,
}

struct Player {
    coordinate: Coordinate,
    direction: Direction,
    size: f64,
    speed: f64,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_dead: bool,
}

impl Player {
    fn new(size: f64, speed: f64) -> Player {
        Player {
            coordinate: Coordinate {
                x: WIDTH as f64 / 2_f64 - size / 2_f64,
                y: HEIGHT as f64 - size - 10_f64,
            },
            direction: Direction::Still,
            size,
            speed,
            is_left_pressed: false,
            is_right_pressed: false,
            is_dead: false,
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = graphics::rectangle::square(
            self.coordinate.x as f64,
            self.coordinate.y as f64,
            self.size as f64,
        );

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        match self.direction {
            Direction::Right => self.move_right(args.dt),
            Direction::Left => self.move_left(args.dt),
            _ => {}
        }
    }

    fn move_right(&mut self, dt: f64) {
        if self.coordinate.x + self.speed*dt > WIDTH as f64 - self.size {
            self.coordinate.x = 0_f64;
        } else {
            self.coordinate.x += self.speed * dt;
        }
    }

    fn move_left(&mut self, dt: f64) {
        if self.coordinate.x - self.speed*dt < 0_f64 {
            self.coordinate.x = WIDTH as f64 - self.size;
        } else {
            self.coordinate.x -= self.speed * dt;
        }
    }

    fn button(&mut self, args: &ButtonArgs) {
        if args.button == Button::Keyboard(Key::A) {
            if args.state == ButtonState::Press {
                self.left_pressed();
            } else {
                self.left_released();
            }
        } else if args.button == Button::Keyboard(Key::D) {
            if args.state == ButtonState::Press {
                self.right_pressed();
            } else {
                self.right_released();
            }
        }
    }

    fn left_pressed(&mut self) {
        self.is_left_pressed = true;
        self.direction = Direction::Left;
    }

    fn left_released(&mut self) {
        self.is_left_pressed = false;
        self.reset_direction();
    }

    fn right_pressed(&mut self) {
        self.is_right_pressed = true;
        self.direction = Direction::Right;
    }

    fn right_released(&mut self) {
        self.is_right_pressed = false;
        self.reset_direction();
    }

    fn reset_direction(&mut self) {
        if self.is_left_pressed {
            self.direction = Direction::Left;
        } else if self.is_right_pressed {
            self.direction = Direction::Right;
        } else {
            self.direction = Direction::Still;
        }
    }

    fn collided(&mut self) {
        self.is_dead = true;
    }
}

struct Faller {
    coordinate: Coordinate,
    velocity: f64,
    size: f64,
    is_dead: bool,
}

impl Faller {
    fn render(&mut self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics;

        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        let square = graphics::rectangle::square(
            self.coordinate.x,
            self.coordinate.y,
            self.size,
        );

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;

            graphics::rectangle(BLUE, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.move_faller(args.dt);
    }

    fn move_faller(&mut self, dt: f64) {
        self.coordinate.y += self.velocity*dt;

        if self.coordinate.y > HEIGHT as f64 {
            self.is_dead = true;
        }
    }
}

fn are_colliding(coord_one: &Coordinate, size_one: f64, coord_two: &Coordinate, size_two: f64) -> bool {
    fn top_left(coord: &Coordinate, size: f64) -> Coordinate {
        Coordinate {
            x: coord.x,
            y: coord.y
        }
    }
    fn top_right(coord: &Coordinate, size: f64) -> Coordinate {
        Coordinate {
            x: coord.x+size,
            y: coord.y
        }
    }
    fn bottom_left(coord: &Coordinate, size: f64) -> Coordinate {
        Coordinate {
            x: coord.x,
            y: coord.y+size
        }
    }
    fn bottom_right(coord: &Coordinate, size: f64) -> Coordinate {
        Coordinate {
            x: coord.x+size,
            y: coord.y+size
        }
    }

    fn is_point_within_square(coord_one: &Coordinate, size_one: f64, coord_two: &Coordinate, size_two: f64) -> bool {
        coord_one.x >= coord_two.x && coord_one.x <= coord_two.x+size_two &&
            coord_one.y >= coord_two.y && coord_one.y <= coord_two.y+size_two
    }

    fn top_collision(coord_one: &Coordinate, size_one: f64, coord_two: &Coordinate, size_two: f64) -> bool {
        is_point_within_square(&top_left(&coord_one, size_one), size_one, &coord_two, size_two) ||
            is_point_within_square(&top_right(&coord_one, size_one), size_one, &coord_two, size_two)
    }

    fn bottom_collision(coord_one: &Coordinate, size_one: f64, coord_two: &Coordinate, size_two: f64) -> bool {
        is_point_within_square(&bottom_left(&coord_one, size_one), size_one, &coord_two, size_two) ||
            is_point_within_square(&bottom_right(&coord_one, size_one), size_one, &coord_two, size_two)
    }

    top_collision(&coord_one, size_one, &coord_two, size_two) || bottom_collision(&coord_one, size_one, &coord_two, size_two)
}

fn load_game_over_image() -> (graphics::Image, opengl_graphics::Texture) {
    let image = graphics::Image::new().rect([0.0, 0.0, 800.0, 120.0]);

    let texture = opengl_graphics::Texture::from_path(Path::new("../images/GameOver.png"), &texture::TextureSettings::new()).unwrap();

    (image, texture)
}

struct App {
    gl: GlGraphics,
    player: Player,
    fallers: Vec<Faller>,
    rng: rand::rngs::ThreadRng,
    spawn_percent_chance: f64,
    faller_size_min: f64,
    faller_size_max: f64,
    faller_size_offset: f64,
    faller_velocity_min: f64,
    faller_velocity_max: f64,
    is_game_over: bool,
    time_elapsed: f64,
}

impl App {
    fn new() -> App {
        App {
            gl: GlGraphics::new(OpenGL::V3_2),
            player: Player::new(30_f64, 500_f64),
            fallers: vec![],
            rng: thread_rng(),
            spawn_percent_chance: 1_f64,
            faller_size_min: 10_f64,
            faller_size_max: 50_f64,
            faller_size_offset: 0_f64,
            faller_velocity_min: 200_f64,
            faller_velocity_max: 500_f64,
            is_game_over: false,
            time_elapsed: 0_f64,
        }
    }

    fn reset(&mut self) {
        self.player = Player::new(30_f64, 500_f64);
        self.fallers = vec![];
        self.spawn_percent_chance = 1_f64;
        self.faller_size_offset = 0_f64;
        self.faller_size_min = 10_f64;
        self.faller_size_max = 50_f64;

        self.time_elapsed = 0_f64;

        self.is_game_over = false;
    }

    fn render(&mut self, args: &RenderArgs) {
        if !self.is_game_over {
            use graphics;

            const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

            self.gl.draw(args.viewport(), |c, gl| {
                graphics::clear(BLACK, gl);
            });

            self.player.render(&mut self.gl, args);

            for faller in &mut self.fallers {
                faller.render(&mut self.gl, args);
            }
        }
    }

    fn render_game_over(&mut self, args: &RenderArgs, game_over_image: &graphics::Image, game_over_texture: &opengl_graphics::Texture) {
        use graphics;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

        self.gl.draw(args.viewport(), |c, gl| {
            graphics::clear(WHITE, gl);

            game_over_image.draw(game_over_texture, &c.draw_state, c.transform, gl);
        });

        //println!("{}", self.time_elapsed);
    }

    fn update(&mut self, args: &UpdateArgs) {
        if self.player.is_dead {
            //self.reset();
            return
        }
        self.possibly_create_random_faller();
        self.spawn_percent_chance += 0.001_f64;
        self.faller_size_offset += 0.003_f64;

        self.player.update(&args);

        for faller in &mut self.fallers {
            faller.update(&args);
            if are_colliding(&self.player.coordinate, self.player.size, &faller.coordinate, faller.size) {
                faller.is_dead = true;
                self.player.collided();
            }
        }
        self.fallers.retain(|faller| !faller.is_dead);

        if self.player.is_dead {
            self.is_game_over = true;
        }

        self.time_elapsed += args.dt;
    }

    fn possibly_create_random_faller(&mut self) {
        if self.rng.gen_range(0_f64, 100_f64) < self.spawn_percent_chance {
            let size = self.rng.gen_range(self.faller_size_min+self.faller_size_offset, self.faller_size_max+self.faller_size_offset);
            self.fallers.push(Faller {
                coordinate: Coordinate {
                    x: self.rng.gen_range(0_f64, WIDTH as f64 -size),
                    y: -size
                },
                velocity: self.rng.gen_range(self.faller_velocity_min, self.faller_velocity_max),
                size: size,
                is_dead: false,
            })
        }
    }

    fn button(&mut self, args: &ButtonArgs) {
        self.player.button(args);
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new("Falling", [WIDTH, HEIGHT])
        .opengl(opengl)
        .vsync(true)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App::new();

    let game_over_image = graphics::Image::new().rect([0.0, 0.0, 800.0, 120.0]);

    let game_over_texture = opengl_graphics::Texture::from_path(Path::new("./images/GameOver.png"), &texture::TextureSettings::new()).unwrap();
    let mut events = Events::new(EventSettings::new().ups(UPS));
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            if app.is_game_over {
                app.render_game_over(&r, &game_over_image, &game_over_texture);
            } else {
                app.render(&r);
            }
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }

        if let Some(b) = e.button_args() {
            app.button(&b);
        }
    }
}
