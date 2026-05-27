//! world4/envs/pong.rs
//! Pong: single-player paddle-ball game.

use std::collections::HashMap;
use rand::{Rng, SeedableRng};

use crate::core::{Env, StepResult};
use crate::spaces::{Box, Discrete};

const BALL_SPEED: f32 = 0.015;
const PADDLE_SPEED: f32 = 0.04;
const PADDLE_HALF: f32 = 0.1;
const PADDLE_X: f32 = 0.04;

#[derive(Debug, Clone)]
pub struct PongEnv {
    max_steps: usize,
    render_mode: Option<String>,
    observation_space: Box,
    action_space: Discrete,
    ball_x: f32,
    ball_y: f32,
    ball_vx: f32,
    ball_vy: f32,
    paddle_y: f32,
    steps: usize,
    hits: usize,
    rng: rand::rngs::StdRng,
}

impl PongEnv {
    pub fn new(max_steps: usize, render_mode: Option<String>) -> Self {
        let low = vec![0.0, 0.0, -BALL_SPEED * 2.0, -BALL_SPEED * 2.0, 0.0];
        let high = vec![1.0, 1.0, BALL_SPEED * 2.0, BALL_SPEED * 2.0, 1.0];
        let observation_space = Box::new(low, high, Some(vec![5]), "f32", None);
        let action_space = Discrete::new(2, 0, None);

        PongEnv {
            max_steps,
            render_mode,
            observation_space,
            action_space,
            ball_x: 0.0,
            ball_y: 0.0,
            ball_vx: 0.0,
            ball_vy: 0.0,
            paddle_y: 0.5,
            steps: 0,
            hits: 0,
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    pub fn observation_space_ref(&self) -> &Box {
        &self.observation_space
    }

    pub fn action_space_ref(&self) -> &Discrete {
        &self.action_space
    }

    pub fn metadata(&self) -> HashMap<&str, &str> {
        let mut m = HashMap::new();
        m.insert("render_modes", "ansi");
        m
    }

    pub fn reward_range(&self) -> (f64, f64) {
        (-1.0, 1.0)
    }
}

impl Env<Vec<f32>, usize> for PongEnv {
    fn reset(&mut self, seed: Option<u64>) -> (Vec<f32>, HashMap<String, String>) {
        if let Some(s) = seed {
            self.rng = rand::rngs::StdRng::seed_from_u64(s);
        }

        let angle = self.rng.gen_range(-1.0f32..1.0) * std::f32::consts::FRAC_PI_4;
        self.ball_x = 0.5;
        self.ball_y = 0.5;
        self.ball_vx = -BALL_SPEED * angle.cos();
        self.ball_vy = BALL_SPEED * angle.sin();
        self.paddle_y = 0.5;
        self.steps = 0;
        self.hits = 0;

        let obs = vec![self.ball_x, self.ball_y, self.ball_vx, self.ball_vy, self.paddle_y];
        (obs, HashMap::new())
    }

    fn step(&mut self, action: usize) -> StepResult<Vec<f32>> {
        if action > 1 {
            panic!("Invalid action {}. Must be 0 (UP) or 1 (DOWN).", action);
        }

        self.steps += 1;

        match action {
            0 => self.paddle_y = (self.paddle_y - PADDLE_SPEED).max(0.0),
            1 => self.paddle_y = (self.paddle_y + PADDLE_SPEED).min(1.0),
            _ => unreachable!(),
        }

        self.ball_x += self.ball_vx;
        self.ball_y += self.ball_vy;

        if self.ball_y <= 0.0 {
            self.ball_y = 0.0;
            self.ball_vy = self.ball_vy.abs();
        }
        if self.ball_y >= 1.0 {
            self.ball_y = 1.0;
            self.ball_vy = -self.ball_vy.abs();
        }

        if self.ball_x >= 1.0 {
            self.ball_x = 1.0;
            self.ball_vx = -self.ball_vx.abs();
        }

        let hit = self.ball_x <= PADDLE_X + 0.02
            && self.ball_x >= PADDLE_X - 0.02
            && (self.ball_y - self.paddle_y).abs() <= PADDLE_HALF;
        if hit {
            self.ball_vx = self.ball_vx.abs();
            self.ball_x = PADDLE_X + 0.02;
            let spin = self.rng.gen_range(-0.005f32..0.005);
            self.ball_vy += spin;
            self.ball_vy = self.ball_vy.clamp(-BALL_SPEED, BALL_SPEED);
            self.hits += 1;
        }

        let terminated = self.ball_x < 0.0;
        let truncated = !terminated && self.steps >= self.max_steps;
        let reward = if terminated { -1.0 } else { 1.0 };

        let obs = vec![self.ball_x, self.ball_y, self.ball_vx, self.ball_vy, self.paddle_y];

        let mut info = HashMap::new();
        info.insert("steps".to_string(), self.steps.to_string());
        info.insert("hits".to_string(), self.hits.to_string());

        StepResult::new(obs, reward, terminated, truncated)
    }

    fn observation_space(&self) -> &dyn std::any::Any {
        &self.observation_space
    }

    fn action_space(&self) -> &dyn std::any::Any {
        &self.action_space
    }

    fn render(&mut self) -> Option<String> {
        const W: usize = 40;
        const H: usize = 16;

        let mut grid = vec![vec![' '; W]; H];

        for x in 0..W {
            grid[0][x] = '─';
            grid[H - 1][x] = '─';
        }
        for y in 0..H {
            grid[y][0] = '│';
            grid[y][W - 1] = '│';
        }

        grid[0][W - 1] = '┐';
        grid[H - 1][W - 1] = '┘';
        grid[0][0] = '┌';
        grid[H - 1][0] = '└';

        let px = ((PADDLE_X * (W - 3) as f32) as usize) + 1;
        let py_center = (self.paddle_y * (H - 2) as f32) as usize + 1;
        let phalf = (PADDLE_HALF * (H - 2) as f32).ceil() as usize;
        for dy in 0..=phalf {
            let y1 = py_center.saturating_sub(dy);
            let y2 = py_center.saturating_add(dy);
            if y1 > 0 && y1 < H - 1 { grid[y1][px] = '█'; }
            if y2 > 0 && y2 < H - 1 { grid[y2][px] = '█'; }
        }

        let bx = ((self.ball_x * (W - 3) as f32) as usize) + 1;
        let by = ((self.ball_y * (H - 2) as f32) as usize) + 1;
        if by > 0 && by < H - 1 && bx > 0 && bx < W - 1 {
            grid[by][bx] = '●';
        }

        let mut out = String::new();
        for row in &grid {
            out.push_str(&row.iter().collect::<String>());
            out.push('\n');
        }
        out.push_str(&format!("  hits={}  steps={}", self.hits, self.steps));

        print!("{}", out);
        Some(out)
    }
}