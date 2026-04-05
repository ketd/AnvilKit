//! Breakout — built with AnvilKit Canvas2D.
//! Cold-start agent test: can an AI write this game using only the public API?

use anvilkit::prelude::*;
use anvilkit::core::time::Time;
use anvilkit_app::{AnvilKitApp, GameCallbacks, GameConfig, GameContext};
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_render::renderer::canvas2d::{Canvas2D, Canvas2DRenderer};

// --- Layout ---
const W: f32 = 640.0;
const H: f32 = 480.0;
const PADDLE_W: f32 = 80.0;
const PADDLE_H: f32 = 12.0;
const PADDLE_Y: f32 = H - 40.0;
const PADDLE_SPEED: f32 = 500.0;
const BALL_SIZE: f32 = 10.0;
const BALL_SPEED: f32 = 300.0;
const BRICK_COLS: usize = 10;
const BRICK_ROWS: usize = 5;
const BRICK_W: f32 = (W - 20.0) / BRICK_COLS as f32;
const BRICK_H: f32 = 20.0;
const BRICK_TOP: f32 = 50.0;
const BRICK_PAD: f32 = 2.0;

// --- Colors ---
const BG: [f32; 4] = [0.05, 0.05, 0.12, 1.0];
const PADDLE: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
const BALL: [f32; 4] = [1.0, 0.3, 0.3, 1.0];
const TEXT: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const ROW_COLORS: [[f32; 4]; 5] = [
    [0.9, 0.2, 0.2, 1.0], // red
    [0.9, 0.6, 0.1, 1.0], // orange
    [0.9, 0.9, 0.2, 1.0], // yellow
    [0.2, 0.8, 0.3, 1.0], // green
    [0.2, 0.5, 0.9, 1.0], // blue
];

struct Brick {
    x: f32,
    y: f32,
    alive: bool,
    color: [f32; 4],
}

struct BreakoutGame {
    renderer: Option<Canvas2DRenderer>,
    paddle_x: f32,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    bricks: Vec<Brick>,
    score: u32,
    lives: u32,
    launched: bool,
    frame_count: u32,
}

impl BreakoutGame {
    fn new() -> Self {
        let mut bricks = Vec::new();
        let x_offset = 10.0;
        for row in 0..BRICK_ROWS {
            for col in 0..BRICK_COLS {
                bricks.push(Brick {
                    x: x_offset + col as f32 * BRICK_W + BRICK_PAD,
                    y: BRICK_TOP + row as f32 * BRICK_H + BRICK_PAD,
                    alive: true,
                    color: ROW_COLORS[row],
                });
            }
        }
        Self {
            renderer: None,
            paddle_x: W / 2.0 - PADDLE_W / 2.0,
            ball_x: W / 2.0,
            ball_y: PADDLE_Y - BALL_SIZE - 2.0,
            ball_vx: BALL_SPEED * 0.7,
            ball_vy: -BALL_SPEED,
            bricks,
            score: 0,
            lives: 3,
            launched: false,
            frame_count: 0,
        }
    }

    fn reset_ball(&mut self) {
        self.ball_x = self.paddle_x + PADDLE_W / 2.0;
        self.ball_y = PADDLE_Y - BALL_SIZE - 2.0;
        self.ball_vx = BALL_SPEED * 0.7;
        self.ball_vy = -BALL_SPEED;
        self.launched = false;
    }
}

impl GameCallbacks for BreakoutGame {
    fn init(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let Some(format) = ctx.render_app.surface_format() else { return };
        self.renderer = Some(Canvas2DRenderer::new(device, format));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let input = ctx.app.world().resource::<InputState>();
        let dt = ctx.app.world().resource::<Time>().delta_seconds();

        if self.lives == 0 { return; }

        // Paddle movement
        if input.is_key_pressed(KeyCode::Left) || input.is_key_pressed(KeyCode::A) {
            self.paddle_x = (self.paddle_x - PADDLE_SPEED * dt).max(0.0);
        }
        if input.is_key_pressed(KeyCode::Right) || input.is_key_pressed(KeyCode::D) {
            self.paddle_x = (self.paddle_x + PADDLE_SPEED * dt).min(W - PADDLE_W);
        }

        // Launch ball
        if !self.launched {
            self.ball_x = self.paddle_x + PADDLE_W / 2.0;
            self.ball_y = PADDLE_Y - BALL_SIZE - 2.0;
            if input.is_key_just_pressed(KeyCode::Space) {
                self.launched = true;
            }
            return;
        }

        // Ball movement
        self.ball_x += self.ball_vx * dt;
        self.ball_y += self.ball_vy * dt;

        // Wall collisions
        if self.ball_x <= 0.0 { self.ball_x = 0.0; self.ball_vx = self.ball_vx.abs(); }
        if self.ball_x + BALL_SIZE >= W { self.ball_x = W - BALL_SIZE; self.ball_vx = -self.ball_vx.abs(); }
        if self.ball_y <= 0.0 { self.ball_y = 0.0; self.ball_vy = self.ball_vy.abs(); }

        // Ball falls below paddle
        if self.ball_y > H {
            self.lives -= 1;
            self.reset_ball();
            return;
        }

        // Paddle collision
        if self.ball_vy > 0.0
            && self.ball_y + BALL_SIZE >= PADDLE_Y
            && self.ball_y + BALL_SIZE <= PADDLE_Y + PADDLE_H
            && self.ball_x + BALL_SIZE >= self.paddle_x
            && self.ball_x <= self.paddle_x + PADDLE_W
        {
            self.ball_vy = -self.ball_vy.abs();
            // Angle based on hit position
            let hit = (self.ball_x + BALL_SIZE / 2.0 - self.paddle_x) / PADDLE_W;
            self.ball_vx = BALL_SPEED * (hit - 0.5) * 2.0;
        }

        // Brick collisions
        let bx = self.ball_x;
        let by = self.ball_y;
        let bs = BALL_SIZE;
        for brick in &mut self.bricks {
            if !brick.alive { continue; }
            let bw = BRICK_W - BRICK_PAD * 2.0;
            let bh = BRICK_H - BRICK_PAD * 2.0;
            if bx + bs > brick.x && bx < brick.x + bw
                && by + bs > brick.y && by < brick.y + bh
            {
                brick.alive = false;
                self.score += 10;
                // Simple reflection: flip vy
                self.ball_vy = -self.ball_vy;
                break; // one brick per frame
            }
        }
    }

    fn render(&mut self, ctx: &mut GameContext) {
        let Some(ref mut renderer) = self.renderer else { return };
        let Some(mut c) = Canvas2D::begin(ctx.render_app, renderer) else { return };

        c.clear(BG);

        // Bricks
        for brick in &self.bricks {
            if !brick.alive { continue; }
            let bw = BRICK_W - BRICK_PAD * 2.0;
            let bh = BRICK_H - BRICK_PAD * 2.0;
            c.draw_rect(brick.x, brick.y, bw, bh, brick.color);
        }

        // Paddle
        c.draw_rect(self.paddle_x, PADDLE_Y, PADDLE_W, PADDLE_H, PADDLE);

        // Ball
        c.draw_rect(self.ball_x, self.ball_y, BALL_SIZE, BALL_SIZE, BALL);

        // HUD
        c.draw_text(10.0, 10.0, &format!("Score: {}  Lives: {}", self.score, self.lives), 20.0, TEXT);

        if !self.launched && self.lives > 0 {
            c.draw_text(W / 2.0 - 90.0, H / 2.0, "Press SPACE to launch", 20.0, TEXT);
        }
        if self.lives == 0 {
            c.draw_text(W / 2.0 - 60.0, H / 2.0, "GAME OVER", 32.0, [1.0, 0.3, 0.3, 1.0]);
        }
        if self.bricks.iter().all(|b| !b.alive) {
            c.draw_text(W / 2.0 - 50.0, H / 2.0, "YOU WIN!", 32.0, [0.3, 1.0, 0.3, 1.0]);
        }

        // Capture first frame for AI agent visual feedback
        self.frame_count += 1;
        if self.frame_count == 3 {
            println!(">>> Capturing frame to screenshots/breakout_frame.png");
            c.capture_frame("screenshots/breakout_frame.png");
            println!(">>> Capture done");
        }

        c.finish();
    }
}

fn main() {
    println!("Breakout — powered by AnvilKit Canvas2D");
    println!("  A/D or Left/Right = move paddle, Space = launch ball");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins::new().with_window(
        WindowConfig::new().with_title("Breakout").with_size(W as u32, H as u32),
    ));

    let config = GameConfig::new("Breakout").with_size(W as u32, H as u32);
    AnvilKitApp::run(config, app, BreakoutGame::new());
}
