extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use piston::{
    Button, ButtonEvent, ButtonState, Key, MouseButton, MouseCursorEvent, MouseScrollEvent,
};

mod objects;
use objects::{check_collide, create_ball, gravitate, solve_collision, Ball};
mod fps;
use fps::FPSCounter;

struct BallMarker {
    point1: [f64; 2],
    point2: [f64; 2],
    radius: f64,
    visible: bool,
}
pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    balls: Vec<Ball>,
    marker: BallMarker,
    mode: GravityMode,
    fps: FPSCounter,
    gravity: f64,
    collisions: Vec<[f64; 2]>,
}

enum GravityMode {
    LinGrav = 1,
    RadGrav = 2,
}

fn mix_colors(color1: [f32; 4], color2: [f32; 4], gradient: f32) -> [f32; 4] {
    let mut result: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
    for i in 0..4 {
        let diff: f32 = (color1[i] - color2[i]).abs();
        if color1[i] > color2[i] {
            result[i] = color2[i] + (diff * gradient);
        } else {
            result[i] = color1[i] + (diff * gradient);
        }
    }

    return result;
}

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BACKGROUND: [f32; 4] = [0.1, 0.1, 0.1, 1.0];
        //const FOREGROUND: [f32; 4] = [0.3, 0.3, 0.3, 1.0];
        const BALL: [f32; 4] = [0.3, 0.3, 1.0, 1.0];
        const BALL_MARKER: [f32; 4] = [0.3, 0.3, 0.4, 0.3];

        //let square = rectangle::rectangle_by_corners(0.0, 300.0, 400.0*2.0, 310.0);

        fn draw_circle(
            color: [f32; 4],
            radius: f64,
            x: f64,
            y: f64,
            c: Context,
            gl: &mut GlGraphics,
        ) {
            let transform = c.transform.trans(x, y).rot_rad(0.0).trans(0.0, 0.0);

            circle_arc(
                color,
                radius * 2.0,
                0.0,
                360.0,
                graphics::rectangle::square(x, y, 0.0),
                transform,
                gl,
            );
        }

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BACKGROUND, gl);

            for b in self.balls.iter() {
                
                draw_circle(BALL, b.r, b.x, 300.0 - b.y, c, gl);
            }

            
            //}

            //draw marker
            if self.marker.visible {
                draw_circle(
                    BALL_MARKER,
                    self.marker.radius,
                    self.marker.point1[0],
                    self.marker.point1[1],
                    c,
                    gl,
                );
                draw_circle(
                    BALL_MARKER,
                    3.0,
                    self.marker.point2[0],
                    self.marker.point2[1],
                    c,
                    gl,
                );

                let diff_vec: [f64; 2] = [
                    self.marker.point1[0] - self.marker.point2[0],
                    self.marker.point1[1] - self.marker.point2[1],
                ];

                let iters = 6;
                for i in 1..iters {
                    draw_circle(
                        BALL_MARKER,
                        1.5,
                        self.marker.point2[0] + diff_vec[0] * (i as f64) / (iters as f64),
                        self.marker.point2[1] + diff_vec[1] * (i as f64) / (iters as f64),
                        c,
                        gl,
                    );
                }
            }
            println!("{}", self.fps.tick())
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        if matches!(self.mode, GravityMode::RadGrav) {
            gravitate(&mut self.balls, args.dt);
        } else {
            for a in 0..self.balls.len() {
                for b in 0..self.balls.len() {
                    let collision_result = check_collide(&self.balls[a], &self.balls[b]);
                    if collision_result.0 && a != b {
                        let mut ball2 = self.balls[b].copy();
                        let mut ball1 = &mut self.balls[a];

                        let collision_coords =
                            solve_collision(&mut ball1, &mut ball2, collision_result.1);

                        if collision_coords.is_some() {
                            self.collisions.push(collision_coords.unwrap());
                        }
                        self.balls[b] = ball2;
                    }
                }
            }
        }
        if matches!(self.mode, GravityMode::RadGrav) {
            for i in self.balls.iter_mut() {
                i.update_speed(args.dt, 0.0);
            }
        } else {
            for i in self.balls.iter_mut() {
                i.update_speed(args.dt, self.gravity);
            }
        }

        for b in self.balls.iter_mut() {
            b.next_step(args.dt);
        }

        if self.balls.len() > 0 && matches!(self.mode, GravityMode::RadGrav) {
            //refocus(&mut self.balls);
        }


    }

    fn add_ball(&mut self, spread_mode: bool) {
        let speed: [f64; 2] = [
            self.marker.point1[0] - self.marker.point2[0],
            self.marker.point2[1] - self.marker.point1[1],
        ];

        self.balls.push(create_ball(
            self.marker.point1[0],
            300.0 - self.marker.point1[1],
            speed[0] * 2.5,
            speed[1] * 2.5,
            self.marker.radius,
            1.0,
        ));
        if spread_mode {
            for i in 0..50 {
                self.balls.push(create_ball(
                    self.marker.point1[0] + (i as f64) * 0.3,
                    300.0 - self.marker.point1[1],
                    speed[0] * 2.5,
                    speed[1] * 2.5,
                    self.marker.radius,
                    1.0,
                ));
            }
        }

        self.marker.visible = false;
    }
    fn update_marker_point2(&mut self, x: f64, y: f64) {
        self.marker.point2 = [x / 2.0, y / 2.0];
    }
    fn update_marker_point1(&mut self, x: f64, y: f64) {
        self.marker.visible = true;
        self.marker.point1 = [x / 2.0, y / 2.0];
    }

    fn update_marker_radius(&mut self, amount: f64) {
        const MIN_SIZE: f64 = 1.0;
        const MAX_SIZE: f64 = 30.0;
        self.marker.radius -= amount;
        if self.marker.radius < MIN_SIZE {
            self.marker.radius = MIN_SIZE;
        } else if self.marker.radius > MAX_SIZE {
            self.marker.radius = MAX_SIZE;
        }
    }

    fn reset(&mut self) {
        self.balls = Vec::<Ball>::new();
    }

    fn delete_ball(&mut self) {
        self.balls.retain(|ball| {
            let delete = {
                let dis_x = ball.x - self.marker.point2[0];
                let dis_y = ball.y - (300.0 - self.marker.point2[1]);
                return f64::sqrt(f64::powf(dis_x, 2.0) + f64::powf(dis_y, 2.0)) > ball.r;
            };
        })
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl: OpenGL = OpenGL::V3_2;
    let left_mouse = Button::Mouse(MouseButton::Left);
    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("Physics", [800, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        balls: Vec::<Ball>::new(),
        marker: BallMarker {
            point1: [0.0, 0.0],
            point2: [0.0, 0.0],
            radius: 10.0,
            visible: false,
        },
        mode: GravityMode::LinGrav,
        fps: FPSCounter::new(),
        gravity: -400.0,
        collisions: Vec::<[f64; 2]>::new(),
    };

    let mut set_spawn_point: bool = false;
    let mut creating_ball: bool = false;
    let mut w_down = false;
    let mut s_down = false;
    let mut spread = false;

    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
            if w_down && creating_ball {
                app.update_marker_radius(-0.1);
            } else if s_down && creating_ball {
                app.update_marker_radius(0.1);
            }
        }

        if let Some(args) = e.mouse_cursor_args() {
            if set_spawn_point {
                app.update_marker_point1(args[0], args[1]);
                set_spawn_point = false;
            }
            app.update_marker_point2(args[0], args[1]);
        }
        if let Some(args) = e.button_args() {
            if args.button == left_mouse && args.state == ButtonState::Press {
                
                set_spawn_point = true;
                creating_ball = true;
            } else if args.button == left_mouse && args.state == ButtonState::Release {
                
                app.add_ball(spread);
            } else if args.button == Button::Keyboard(Key::R) && args.state == ButtonState::Press {
                
                app.reset();
            } else if args.button == Button::Keyboard(Key::S) && args.state == ButtonState::Press {
                
                s_down = true;
            } else if args.button == Button::Keyboard(Key::W) && args.state == ButtonState::Press {
                
                w_down = true;
            } else if args.button == Button::Keyboard(Key::S) && args.state == ButtonState::Release
            {
                
                s_down = false;
            } else if args.button == Button::Keyboard(Key::W) && args.state == ButtonState::Release
            {
                
                w_down = false;
            } else if args.button == Button::Keyboard(Key::D) && args.state == ButtonState::Press {
                
                app.mode = GravityMode::LinGrav;
            } else if args.button == Button::Keyboard(Key::A) && args.state == ButtonState::Press {
                
                app.mode = GravityMode::RadGrav;
            } else if args.button == Button::Mouse(MouseButton::Right)
                && args.state == ButtonState::Press
            {
                app.delete_ball();
                println!("pressed");
            } else if args.button == Button::Keyboard(Key::E) && args.state == ButtonState::Press {
                
                spread = true;
            } else if args.button == Button::Keyboard(Key::E) && args.state == ButtonState::Release
            {
                
                spread = false;
            }
        }
        if let Some(args) = e.mouse_scroll_args() {
            if creating_ball {
                app.update_marker_radius(args[1]);
            }
        }
    }
}
