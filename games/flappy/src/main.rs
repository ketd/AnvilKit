//! Flappy Bird — built with AnvilKit's Canvas2D API.
//!
//! This game was written as an AI-friendliness validation test.
//! The entire rendering uses Canvas2D (zero wgpu knowledge required).

use anvilkit::prelude::*;
use anvilkit_app::{AnvilKitApp, GameCallbacks, GameConfig, GameContext};
use anvilkit_input::prelude::{InputState, KeyCode};
use anvilkit_render::renderer::canvas2d::{Canvas2D, Canvas2DRenderer};
use anvilkit::core::time::Time;

// --- Constants ---
const BIRD_X: f32 = 120.0;
const BIRD_SIZE: f32 = 24.0;
const GRAVITY: f32 = 800.0;
const JUMP_VELOCITY: f32 = -320.0;
const PIPE_WIDTH: f32 = 60.0;
const PIPE_GAP: f32 = 150.0;
const PIPE_SPEED: f32 = 180.0;
const PIPE_SPACING: f32 = 280.0;

// --- Colors ---
const SKY: [f32; 4] = [0.3, 0.6, 0.9, 1.0];
const BIRD_COLOR: [f32; 4] = [1.0, 0.85, 0.0, 1.0];
const PIPE_COLOR: [f32; 4] = [0.2, 0.75, 0.2, 1.0];
const GROUND_COLOR: [f32; 4] = [0.55, 0.35, 0.15, 1.0];
const TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

struct Pipe {
    x: f32,
    gap_y: f32, // center of the gap
    scored: bool,
}

struct FlappyGame {
    canvas_renderer: Option<Canvas2DRenderer>,
    bird_y: f32,
    bird_vy: f32,
    pipes: Vec<Pipe>,
    score: u32,
    alive: bool,
    started: bool,
}

impl FlappyGame {
    fn new() -> Self {
        Self {
            canvas_renderer: None,
            bird_y: 300.0,
            bird_vy: 0.0,
            pipes: Vec::new(),
            score: 0,
            alive: true,
            started: false,
        }
    }

    fn reset(&mut self) {
        self.bird_y = 300.0;
        self.bird_vy = 0.0;
        self.pipes.clear();
        self.score = 0;
        self.alive = true;
        self.started = false;
    }

    fn spawn_pipes(&mut self, screen_w: f32, screen_h: f32) {
        let last_x = self.pipes.last().map(|p| p.x).unwrap_or(screen_w);
        while last_x + PIPE_SPACING < screen_w + 600.0 {
            let gap_y = 150.0 + (self.pipes.len() as f32 * 73.0 % (screen_h - 300.0));
            let x = self.pipes.last().map(|p| p.x + PIPE_SPACING).unwrap_or(screen_w + 100.0);
            self.pipes.push(Pipe { x, gap_y, scored: false });
        }
    }
}

impl GameCallbacks for FlappyGame {
    fn init(&mut self, ctx: &mut GameContext) {
        let Some(device) = ctx.render_app.render_device() else { return };
        let Some(format) = ctx.render_app.surface_format() else { return };
        self.canvas_renderer = Some(Canvas2DRenderer::new(device, format));
    }

    fn update(&mut self, ctx: &mut GameContext) {
        let input = ctx.app.world().resource::<InputState>();
        let dt = ctx.app.world().resource::<Time>().delta_seconds();

        // Jump on Space or restart on R
        if input.is_key_just_pressed(KeyCode::Space) {
            if self.alive {
                self.started = true;
                self.bird_vy = JUMP_VELOCITY;
            }
        }
        if input.is_key_just_pressed(KeyCode::R) {
            self.reset();
            return;
        }

        if !self.started || !self.alive { return; }

        // Bird physics
        self.bird_vy += GRAVITY * dt;
        self.bird_y += self.bird_vy * dt;

        // Move pipes
        for pipe in &mut self.pipes {
            pipe.x -= PIPE_SPEED * dt;
        }
        self.pipes.retain(|p| p.x + PIPE_WIDTH > -50.0);

        // Spawn new pipes
        let (w, h) = ctx.render_app.window_state().size();
        self.spawn_pipes(w as f32, h as f32);

        // Collision: ground/ceiling
        let ground = h as f32 - 60.0;
        if self.bird_y + BIRD_SIZE > ground || self.bird_y < 0.0 {
            self.alive = false;
        }

        // Collision: pipes + scoring
        let bird_rect = (BIRD_X, self.bird_y, BIRD_X + BIRD_SIZE, self.bird_y + BIRD_SIZE);
        for pipe in &mut self.pipes {
            let top_pipe = (pipe.x, 0.0, pipe.x + PIPE_WIDTH, pipe.gap_y - PIPE_GAP / 2.0);
            let bot_pipe = (pipe.x, pipe.gap_y + PIPE_GAP / 2.0, pipe.x + PIPE_WIDTH, ground);

            if rects_overlap(bird_rect, top_pipe) || rects_overlap(bird_rect, bot_pipe) {
                self.alive = false;
            }

            if !pipe.scored && pipe.x + PIPE_WIDTH < BIRD_X {
                pipe.scored = true;
                self.score += 1;
            }
        }
    }

    fn render(&mut self, ctx: &mut GameContext) {
        let Some(ref mut renderer) = self.canvas_renderer else { return };
        let Some(mut canvas) = Canvas2D::begin(ctx.render_app, renderer) else { return };

        let w = canvas.width();
        let h = canvas.height();
        let ground = h - 60.0;

        // Background
        canvas.clear(SKY);

        // Pipes
        for pipe in &self.pipes {
            let top_h = pipe.gap_y - PIPE_GAP / 2.0;
            let bot_y = pipe.gap_y + PIPE_GAP / 2.0;
            canvas.draw_rect(pipe.x, 0.0, PIPE_WIDTH, top_h, PIPE_COLOR);
            canvas.draw_rect(pipe.x, bot_y, PIPE_WIDTH, ground - bot_y, PIPE_COLOR);
        }

        // Ground
        canvas.draw_rect(0.0, ground, w, 60.0, GROUND_COLOR);

        // Bird
        canvas.draw_rect(BIRD_X, self.bird_y, BIRD_SIZE, BIRD_SIZE, BIRD_COLOR);

        // Score
        canvas.draw_text(w / 2.0 - 20.0, 30.0, &format!("{}", self.score), 48.0, TEXT_COLOR);

        // Status text
        if !self.started {
            canvas.draw_text(w / 2.0 - 100.0, h / 2.0, "Press SPACE to start", 24.0, TEXT_COLOR);
        } else if !self.alive {
            canvas.draw_text(w / 2.0 - 80.0, h / 2.0, "GAME OVER — R to restart", 24.0, TEXT_COLOR);
        }

        canvas.finish();
    }
}

fn rects_overlap(a: (f32, f32, f32, f32), b: (f32, f32, f32, f32)) -> bool {
    a.0 < b.2 && a.2 > b.0 && a.1 < b.3 && a.3 > b.1
}

fn main() {
    println!("Flappy Bird — powered by AnvilKit Canvas2D");
    println!("  SPACE = jump, R = restart");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins::new().with_window(
        WindowConfig::new().with_title("Flappy Bird").with_size(480, 640),
    ));

    let config = GameConfig::new("Flappy Bird").with_size(480, 640);
    AnvilKitApp::run(config, app, FlappyGame::new());
}
