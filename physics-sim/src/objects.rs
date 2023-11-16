use std::f64::consts::PI;

const RIGHT_WALL: f64 = 400.0;
const G: f64 = 50.0;

pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub u: f64,
    pub v: f64,
    pub r: f64,
    pub elas: f64,
    pub m: f64,
}

struct Vector2 {
    pub x: f64,
    pub y: f64,
}

static mut COL_COUNT: u32 = 0;

fn moving_apart(ball: &Ball, other: &Ball, distance: f64) -> bool {
    let tiny_step = 0.001;
    let sim_distance_x = (other.x + other.u * tiny_step) - (ball.x + ball.u * tiny_step);
    let sim_distance_y = (other.y + other.v * tiny_step) - (ball.y + ball.v * tiny_step);
    return f64::sqrt(f64::powf(sim_distance_x, 2.0) + f64::powf(sim_distance_y, 2.0)) > distance;
}

impl Ball {
    pub fn update_speed(&mut self, dt: f64, mut gravity: f64) {
        let touched_ground = self.y <= self.r;
        self.flip_speed(
            self.x - self.r < 0.0 || self.x + self.r >= RIGHT_WALL,
            touched_ground,
        );
        if touched_ground {
            gravity = 0.0;
        }
        self.accelerate(&0.0, &gravity, dt);
    }

    fn accelerate(&mut self, x_a: &f64, y_a: &f64, dt: f64) {
        (*self).u += x_a * dt;
        (*self).v += y_a * dt;
    }
    fn flip_speed(&mut self, x_flip: bool, y_flip: bool) {
        if x_flip {
            self.u = 0.0 - self.u;

            self.u *= self.elas;
            self.v *= self.elas;
        }
        if y_flip && self.v < 0.0 {
            self.v = 0.0 - self.v;

            self.u *= self.elas;
            self.v *= self.elas;
        }
    }

    pub fn next_step(&mut self, dt: f64) {
        (*self).x += self.u * dt;
        (*self).y += self.v * dt;
    }

    pub fn copy(&self) -> Ball {
        Ball {
            x: self.x,
            y: self.y,
            u: self.u,
            v: self.v,
            r: self.r,
            elas: self.elas,
            m: self.m,
        }
    }

    pub fn get_energy(&self, g: f64) -> f64 {
        return 0.5
            * self.m
            * f64::powf(
                f64::sqrt(f64::powf(self.u, 2.0) + f64::powf(self.v, 2.0)),
                2.0,
            )
            + self.m * -g * self.y;
    }
}

pub fn create_ball(x: f64, y: f64, u: f64, v: f64, r: f64, elas: f64) -> Ball {
    return Ball {
        x: x,
        y: y,
        u: u,
        v: v,
        r: r,
        elas: elas,
        m: r * r * PI,
    };
}

pub fn solve_collision(ball1: &mut Ball, ball2: &mut Ball, distance: f64) -> Option<[f64; 2]> {
    if moving_apart(&ball1, &ball2, distance) {
        return None;
    }

    //vector normal to the collison point
    let mut collision = Vector2 {
        x: (ball2.x - ball1.x),
        y: (ball2.y - ball1.y),
    };

    //normalize collison normal
    collision.x /= distance;
    collision.y /= distance;

    // Get the components of the velocity vectors which are parallel to the collision.
    // The perpendicular component remains the same for both fish

    //double aci = a.velocity().dot(collision);
    //double bci = b.velocity().dot(collision);
    let aci = ball1.u * collision.x + ball1.v * collision.y;
    let bci = ball2.u * collision.x + ball2.v * collision.y;

    // Solve for the new velocities using the 1-dimensional elastic collision equations.
    // Turns out it's really simple when the masses are the same.

    // Replace the collision velocity components with the new ones
    //a.velocity() += (acf - aci) * collision;
    ball1.u += ((bci - aci)
        - (((ball1.m - ball2.m) / (ball1.m + ball2.m)) * (bci - aci)
            + ((2.0 * ball2.m) / (ball1.m + ball2.m)) * (aci - bci)))
        * collision.x
        * 0.5;
    ball1.v += ((bci - aci)
        - (((ball1.m - ball2.m) / (ball1.m + ball2.m)) * (bci - aci)
            + ((2.0 * ball2.m) / (ball1.m + ball2.m)) * (aci - bci)))
        * collision.y
        * 0.5;
    //b.velocity() += (bcf - bci) * collision;
    ball2.u += ((aci - bci)
        - (((ball2.m - ball1.m) / (ball1.m + ball2.m)) * (aci - bci)
            + ((2.0 * ball1.m) / (ball1.m + ball2.m)) * (bci - aci)))
        * collision.x
        * 0.5;
    ball2.v += ((aci - bci)
        - (((ball2.m - ball1.m) / (ball1.m + ball2.m)) * (aci - bci)
            + ((2.0 * ball1.m) / (ball1.m + ball2.m)) * (bci - aci)))
        * collision.y
        * 0.5;

    ball1.u *= ball1.elas;
    ball1.v *= ball1.elas;

    ball2.u *= ball2.elas;
    ball2.v *= ball2.elas;

    unsafe {
        if COL_COUNT % 10000 == 0 {
            println!("Collison: {}", COL_COUNT);
        }
        COL_COUNT += 1;
    }

    let mut result = [0.0, 0.0];
    if ball1.x < ball2.x {
        result[0] = ball1.x + ball1.r;
    } else {
        result[0] = ball2.x + ball2.r;
    }
    if ball1.y < ball2.y {
        result[1] = ball1.y + ball1.r;
    } else {
        result[1] = ball2.y + ball2.r;
    }
    return Some([collision.x, collision.y]);
}

pub fn check_collide(ball1: &Ball, ball2: &Ball) -> (bool, f64) {
    if ball1.x == ball2.x && ball1.y == ball2.y {
        return (false, 0.0);
    }

    let distance = f64::sqrt(f64::powf(ball2.x - ball1.x, 2.0) + f64::powf(ball2.y - ball1.y, 2.0));

    if distance < (ball2.r + ball1.r) {
        return (true, distance);
    } else {
        return (false, 0.0);
    }
}

fn add_gravity(ball1: &mut Ball, ball2: &Ball, dt: f64) {
    let mut to_other = Vector2 {
        x: (ball2.x - ball1.x),
        y: ball2.y - ball1.y,
    };
    let mut distance =
        f64::sqrt(f64::powf(ball2.x - ball1.x, 2.0) + f64::powf(ball2.y - ball1.y, 2.0));
    to_other.x /= distance;
    to_other.y /= distance;

    if distance < 0.02 {
        distance = 0.02;
    }
    let acc: f64 = (G * ball2.m) / (distance * distance) * dt;

    ball1.u += to_other.x * acc;
    ball1.v += to_other.y * acc;
}

pub fn gravitate(balls: &mut Vec<Ball>, dt: f64) {
    for a in 0..balls.len() {
        let mut this_ball = balls[a].copy();
        for b in 0..balls.len() {
            if a != b {
                add_gravity(&mut this_ball, &balls[b], dt);
            }
        }
        balls[a] = this_ball;
    }
}

pub fn refocus(balls: &mut Vec<Ball>) {
    let x = balls[0].x - 200.0;
    let y = balls[0].y - 150.0;

    for b in balls.iter_mut() {
        b.x -= x;
        b.y -= y;
    }
}
