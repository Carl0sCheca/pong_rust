use rand::prelude::Distribution;

const PADDLE_WIDTH: f32 = 30.0;
const PADDLE_HEIGHT: f32 = 100.0;
const PADDLE_SPEED: f32 = 500.0;
const BALL_SIZE: f32 = 20.0;
const BALL_SPEED: f32 = 400.0;

#[derive(Debug)]
pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn normalize(&self) -> Self {
        let norm: f32 = (self.x.powf(2.0) + self.y.powf(2.0)).sqrt();

        Self {
            x: self.x / norm,
            y: self.y / norm,
        }
    }
}

#[derive(Debug)]
pub struct Player {
    pub position: Vector2D,
    pub vertices: [[f32; 3]; 4],
    pub input: Input,
    pub points: u32,
}

impl Player {
    pub fn new(x: f32, y: f32, size: &winit::dpi::PhysicalSize<u32>) -> Self {
        Self {
            position: Vector2D::new(x, y),
            vertices: Player::move_vertices([x, y], size),
            input: Input::None,
            points: 0,
        }
    }

    pub fn move_position(&mut self, x: f32, y: f32, size: &winit::dpi::PhysicalSize<u32>) {
        self.position = Vector2D::new(x, y);
        self.vertices = Player::move_vertices([self.position.x, self.position.y], size);
    }

    pub fn move_vertices(to: [f32; 2], size: &winit::dpi::PhysicalSize<u32>) -> [[f32; 3]; 4] {
        let mut position: [[f32; 3]; 4] = [[0.0; 3]; 4];
        position[0] = crate::engine::Engine::screen_space_to_clip_space(&[to[0], to[1], 0.0], size);
        position[1] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0] + PADDLE_WIDTH, to[1], 0.0],
            size,
        );
        position[2] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0] + PADDLE_WIDTH, to[1] + PADDLE_HEIGHT, 0.0],
            size,
        );
        position[3] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0], to[1] + PADDLE_HEIGHT, 0.0],
            size,
        );

        position
    }
}

#[derive(Debug)]
pub struct Controller {
    size: winit::dpi::PhysicalSize<u32>,
    pub players: [Player; 2],
    pub ball: Ball,
}

#[derive(Debug)]
pub struct Ball {
    pub position: Vector2D,
    pub vertices: [[f32; 3]; 4],
    pub direction: Vector2D,
}

impl Ball {
    pub fn new(x: f32, y: f32, direction: Vector2D, size: &winit::dpi::PhysicalSize<u32>) -> Self {
        Self {
            position: Vector2D::new(x, y),
            vertices: Ball::move_vertices([x, y], size),
            direction: direction.normalize(),
        }
    }

    pub fn move_position(&mut self, x: f32, y: f32, size: &winit::dpi::PhysicalSize<u32>) {
        self.position = Vector2D::new(x, y);
        self.vertices = Ball::move_vertices([self.position.x, self.position.y], size)
    }

    pub fn move_vertices(to: [f32; 2], size: &winit::dpi::PhysicalSize<u32>) -> [[f32; 3]; 4] {
        let mut position: [[f32; 3]; 4] = [[0.0; 3]; 4];
        position[0] = crate::engine::Engine::screen_space_to_clip_space(&[to[0], to[1], 0.0], size);
        position[1] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0] + BALL_SIZE, to[1], 0.0],
            size,
        );
        position[2] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0] + BALL_SIZE, to[1] + BALL_SIZE, 0.0],
            size,
        );
        position[3] = crate::engine::Engine::screen_space_to_clip_space(
            &[to[0], to[1] + BALL_SIZE, 0.0],
            size,
        );

        position
    }
}

impl Controller {
    pub fn new(size: &winit::dpi::PhysicalSize<u32>) -> Self {
        let size = winit::dpi::PhysicalSize {
            width: size.width,
            height: size.height,
        };

        Self {
            size,
            players: [
                Player::new(
                    0.0,
                    (size.height as f32 / 2.0) - (PADDLE_HEIGHT / 2.0),
                    &size,
                ),
                Player::new(
                    size.width as f32 - PADDLE_WIDTH,
                    (size.height as f32 / 2.0) - (PADDLE_HEIGHT / 2.0),
                    &size,
                ),
            ],
            ball: Ball::new(
                (size.width as f32 / 2.0) - (BALL_SIZE / 2.0),
                (size.height as f32 / 2.0) - (BALL_SIZE / 2.0),
                Vector2D::new(
                    rand::distributions::Uniform::new(-1.0, 1.0).sample(&mut rand::thread_rng()),
                    rand::distributions::Uniform::new(-0.1, 0.1).sample(&mut rand::thread_rng()),
                ),
                &size,
            ),
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.input(0, dt);
        self.input(1, dt);
        self.ball_update(dt);
    }

    pub fn ball_update(&mut self, dt: f32) {
        self.ball.move_position(
            self.ball.position.x + BALL_SPEED * dt * self.ball.direction.x,
            self.ball.position.y + BALL_SPEED * dt * self.ball.direction.y,
            &self.size,
        );

        if self.ball.position.y + BALL_SIZE > self.size.height as f32 || self.ball.position.y < 0.0
        {
            self.ball.direction.y *= -1.0;
        }

        for player in &self.players {
            let anchor_correction = if self.ball.direction.x.is_sign_positive() {
                BALL_SIZE
            } else {
                0.0
            };

            if self.ball.position.x + anchor_correction >= player.position.x
                && self.ball.position.x <= player.position.x + PADDLE_WIDTH
                && self.ball.position.y >= player.position.y - BALL_SIZE
                && self.ball.position.y <= player.position.y + PADDLE_HEIGHT
            {
                self.ball.direction.x *= -1.0;

                let bounce_direction = self.ball.position.y - player.position.y;
                if (25.0..=55.0).contains(&bounce_direction) {
                    self.ball.direction.y = rand::distributions::Uniform::new(-0.05, 0.05)
                        .sample(&mut rand::thread_rng());
                } else if bounce_direction > 55.0 {
                    self.ball.direction.y = rand::distributions::Uniform::new(0.3, 1.0)
                        .sample(&mut rand::thread_rng());
                } else {
                    self.ball.direction.y = rand::distributions::Uniform::new(-1.0, -0.3)
                    .sample(&mut rand::thread_rng());
                }
                self.ball.direction = self.ball.direction.normalize();
            }
        }

        if self.ball.position.x + BALL_SIZE < 0.0 {
            self.ball = Ball::new(
                (self.size.width as f32 / 2.0) - (BALL_SIZE / 2.0),
                (self.size.height as f32 / 2.0) - (BALL_SIZE / 2.0),
                Vector2D::new(
                    rand::distributions::Uniform::new(0.0, 1.0).sample(&mut rand::thread_rng()),
                    rand::distributions::Uniform::new(-0.1, 0.1).sample(&mut rand::thread_rng()),
                ),
                &self.size,
            );

            self.players[1].points += 1;
            println!(
                "Player 2 scored, {}-{}",
                self.players[0].points, self.players[1].points
            );
        } else if self.ball.position.x > self.size.width as f32 {
            self.ball = Ball::new(
                (self.size.width as f32 / 2.0) - (BALL_SIZE / 2.0),
                (self.size.height as f32 / 2.0) - (BALL_SIZE / 2.0),
                Vector2D::new(
                    rand::distributions::Uniform::new(-1.0, 0.0).sample(&mut rand::thread_rng()),
                    rand::distributions::Uniform::new(-0.1, 0.1).sample(&mut rand::thread_rng()),
                ),
                &self.size,
            );
            self.players[0].points += 1;
            println!(
                "Player 1 scored, {}-{}",
                self.players[0].points, self.players[1].points
            );
        }
    }

    pub fn input(&mut self, player: usize, dt: f32) {
        match self.players[player].input {
            crate::pong::Input::Up => {
                self.players[player].move_position(
                    self.players[player].position.x,
                    self.players[player].position.y + (PADDLE_SPEED * dt),
                    &self.size,
                );

                if self.players[player].position.y + PADDLE_HEIGHT > self.size.height as f32 {
                    self.players[player].move_position(
                        self.players[player].position.x,
                        self.size.height as f32 - PADDLE_HEIGHT,
                        &self.size,
                    );
                }
            }
            crate::pong::Input::Down => {
                self.players[player].move_position(
                    self.players[player].position.x,
                    self.players[player].position.y - (PADDLE_SPEED * dt),
                    &self.size,
                );

                if self.players[player].position.y < 0.0 {
                    self.players[player].move_position(
                        self.players[player].position.x,
                        0.0,
                        &self.size,
                    );
                }
            }
            crate::pong::Input::None => (),
        }
    }
}

#[derive(Debug)]
pub enum Input {
    Up,
    Down,
    None,
}
