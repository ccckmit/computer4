//! world4/envs/cartpole.rs
//! CartPole-v1: balance a pole on a moving cart.

use std::collections::HashMap;
use rand::{Rng, SeedableRng};

use crate::core::{Env, StepResult};
use crate::spaces::{Box, Discrete};

const GRAVITY: f32 = 9.8;
const MASS_CART: f32 = 1.0;
const MASS_POLE: f32 = 0.1;
const TOTAL_MASS: f32 = MASS_CART + MASS_POLE;
const HALF_POLE_LENGTH: f32 = 0.5;
const POLE_MASS_LENGTH: f32 = MASS_POLE * HALF_POLE_LENGTH;
const FORCE_MAG: f32 = 10.0;
const TAU: f32 = 0.02;
const X_THRESHOLD: f32 = 2.4;
const THETA_THRESHOLD_RAD: f32 = 12.0 * std::f32::consts::PI / 180.0;

#[derive(Debug, Clone)]
pub struct CartPoleEnv {
    max_steps: usize,
    render_mode: Option<String>,
    observation_space: Box,
    action_space: Discrete,
    state: Option<Vec<f32>>,
    steps: usize,
    rng: rand::rngs::StdRng,
}

impl CartPoleEnv {
    pub fn new(max_steps: usize, render_mode: Option<String>) -> Self {
        let high = vec![
            X_THRESHOLD * 2.0,
            f32::MAX,
            THETA_THRESHOLD_RAD * 2.0,
            f32::MAX,
        ];
        let low = vec![
            -X_THRESHOLD * 2.0,
            -f32::MAX,
            -THETA_THRESHOLD_RAD * 2.0,
            -f32::MAX,
        ];
        let observation_space = Box::new(low, high, Some(vec![4]), "f32", None);
        let action_space = Discrete::new(2, 0, None);

        CartPoleEnv {
            max_steps,
            render_mode,
            observation_space,
            action_space,
            state: None,
            steps: 0,
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
        (0.0, 1.0)
    }
}

impl Env<Vec<f32>, usize> for CartPoleEnv {
    fn reset(&mut self, seed: Option<u64>) -> (Vec<f32>, HashMap<String, String>) {
        if let Some(s) = seed {
            self.rng = rand::rngs::StdRng::seed_from_u64(s);
        }

        let state: Vec<f32> = (0..4).map(|_| self.rng.gen_range(-0.05..0.05)).collect();
        self.state = Some(state.clone());
        self.steps = 0;

        (state, HashMap::new())
    }

    fn step(&mut self, action: usize) -> StepResult<Vec<f32>> {
        let state = self.state.take().expect("Call reset() before step()");

        if action > 1 {
            panic!("Invalid action {}. Must be 0 (left) or 1 (right).", action);
        }

        self.steps += 1;

        let mut x = state[0];
        let mut x_dot = state[1];
        let mut theta = state[2];
        let mut theta_dot = state[3];

        let force = if action == 1 { FORCE_MAG } else { -FORCE_MAG };
        let cos_theta = theta.cos();
        let sin_theta = theta.sin();

        let temp = (force + POLE_MASS_LENGTH * theta_dot * theta_dot * sin_theta) / TOTAL_MASS;
        let theta_acc = (GRAVITY * sin_theta - cos_theta * temp) /
            (HALF_POLE_LENGTH * (4.0 / 3.0 - MASS_POLE * cos_theta * cos_theta / TOTAL_MASS));
        let x_acc = temp - POLE_MASS_LENGTH * theta_acc * cos_theta / TOTAL_MASS;

        x += TAU * x_dot;
        x_dot += TAU * x_acc;
        theta += TAU * theta_dot;
        theta_dot += TAU * theta_acc;

        let new_state = vec![x, x_dot, theta, theta_dot];
        self.state = Some(new_state.clone());

        let terminated = x < -X_THRESHOLD || x > X_THRESHOLD ||
                         theta < -THETA_THRESHOLD_RAD || theta > THETA_THRESHOLD_RAD;
        let truncated = !terminated && self.steps >= self.max_steps;
        let reward = if terminated { 0.0 } else { 1.0 };

        let mut info = HashMap::new();
        info.insert("steps".to_string(), self.steps.to_string());
        info.insert("x".to_string(), format!("{:.3}", x));
        info.insert("theta_deg".to_string(), format!("{:.1}", theta.to_degrees()));

        StepResult::new(new_state, reward, terminated, truncated)
    }

    fn observation_space(&self) -> &dyn std::any::Any {
        &self.observation_space
    }

    fn action_space(&self) -> &dyn std::any::Any {
        &self.action_space
    }

    fn render(&mut self) -> Option<String> {
        let state = self.state.as_ref()?;

        let x = state[0];
        let theta = state[2];

        let width = 60;
        let cart_col = ((x + X_THRESHOLD) / (2.0 * X_THRESHOLD) * (width as f32 - 1.0)) as usize;
        let cart_col = std::cmp::max(0, std::cmp::min(width - 1, cart_col));

        let pole_tip_offset = (theta.sin() * 10.0) as isize;
        let pole_col = std::cmp::max(0, std::cmp::min(width - 1, (cart_col as isize + pole_tip_offset) as usize));

        let mut track = vec!['-'; width];
        track[cart_col] = '█';

        let mut pole_row = vec![' '; width];
        let start = std::cmp::min(cart_col, pole_col);
        let end = std::cmp::max(cart_col, pole_col);
        for col in start..=end {
            pole_row[col] = if theta > 0.0 { '/' } else { '\\' };
        }
        pole_row[pole_col] = 'O';

        let theta_deg = theta.to_degrees();
        let status = format!("  x={:+.3}  θ={:+.1}°  steps={}", x, theta_deg, self.steps);

        let out = format!(
            "┌{}┐\n│{}│\n│{}│\n└{}┘\n{}",
            "─".repeat(width),
            pole_row.iter().collect::<String>(),
            track.iter().collect::<String>(),
            "─".repeat(width),
            status
        );

        println!("{}", out);
        Some(out)
    }
}