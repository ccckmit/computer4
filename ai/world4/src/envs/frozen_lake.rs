//! world4/envs/frozen_lake.rs
//! FrozenLake-v1: a grid-world where the agent navigates a frozen lake.

use std::collections::HashMap;
use rand::{Rng, SeedableRng};

use crate::core::{Env, StepResult};
use crate::spaces::Discrete;

const MAPS: &[(&str, &[&str])] = &[
    ("4x4", &["SFFF", "FHFH", "FFFH", "HFFG"]),
    ("8x8", &[
        "SFFFFFFF",
        "FFFFFFFF",
        "FFFHFFFF",
        "FFFFFHFF",
        "FFFHFFFF",
        "FHHFFFHF",
        "FHFFHFHF",
        "FFFHFFFG",
    ]),
];

const ACTIONS: [(i32, i32); 4] = [
    (0, -1),  // LEFT
    (1, 0),   // DOWN
    (0, 1),   // RIGHT
    (-1, 0),  // UP
];

const ACTION_NAMES: [&str; 4] = ["LEFT", "DOWN", "RIGHT", "UP"];

#[derive(Debug, Clone)]
pub struct FrozenLakeEnv {
    desc: Vec<Vec<u8>>,
    nrow: usize,
    ncol: usize,
    is_slippery: bool,
    start: (usize, usize),
    holes: Vec<(usize, usize)>,
    goal: (usize, usize),
    observation_space: Discrete,
    action_space: Discrete,
    max_steps: usize,
    agent_pos: (usize, usize),
    steps: usize,
    rng: rand::rngs::StdRng,
}

impl FrozenLakeEnv {
    pub fn new(map_name: &str, custom_map: Option<&[&str]>, is_slippery: bool, max_steps: Option<usize>) -> Self {
        let desc: Vec<Vec<u8>> = if let Some(custom) = custom_map {
            custom.iter().map(|s| s.as_bytes().to_vec()).collect()
        } else {
            let map = MAPS.iter().find(|(name, _)| *name == map_name).unwrap();
            map.1.iter().map(|s| s.as_bytes().to_vec()).collect()
        };

        let nrow = desc.len();
        let ncol = desc[0].len();

        let (start, holes, goal) = Self::find_positions(&desc);

        let n_states = nrow * ncol;
        let max_steps = max_steps.unwrap_or(100 * nrow);

        FrozenLakeEnv {
            desc,
            nrow,
            ncol,
            is_slippery,
            start,
            holes,
            goal,
            observation_space: Discrete::new(n_states, 0, None),
            action_space: Discrete::new(4, 0, None),
            max_steps,
            agent_pos: start,
            steps: 0,
            rng: rand::rngs::StdRng::from_entropy(),
        }
    }

    fn find_positions(desc: &[Vec<u8>]) -> ((usize, usize), Vec<(usize, usize)>, (usize, usize)) {
        let mut start = (0, 0);
        let mut holes = Vec::new();
        let mut goal = (0, 0);

        for (r, row) in desc.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                match cell {
                    b'S' => start = (r, c),
                    b'H' => holes.push((r, c)),
                    b'G' => goal = (r, c),
                    _ => {}
                }
            }
        }
        (start, holes, goal)
    }

    fn pos_to_state(&self, pos: (usize, usize)) -> usize {
        pos.0 * self.ncol + pos.1
    }

    pub fn observation_space_ref(&self) -> &Discrete {
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
}

impl Env<usize, usize> for FrozenLakeEnv {
    fn reset(&mut self, seed: Option<u64>) -> (usize, HashMap<String, String>) {
        if let Some(s) = seed {
            self.rng = rand::rngs::StdRng::seed_from_u64(s);
        }
        self.agent_pos = self.start;
        self.steps = 0;
        let obs = self.pos_to_state(self.agent_pos);
        let mut info = HashMap::new();
        info.insert("pos".to_string(), format!("{:?}", self.agent_pos));
        (obs, info)
    }

    fn step(&mut self, action: usize) -> StepResult<usize> {
        if action >= 4 {
            panic!("Invalid action {}. Must be in [0, 3].", action);
        }

        self.steps += 1;

        let action = if self.is_slippery {
            let candidates = [(action + 3) % 4, action, (action + 1) % 4];
            let idx = self.rng.gen_range(0..3);
            candidates[idx]
        } else {
            action
        };

        let (dr, dc) = ACTIONS[action];
        let (r, c) = self.agent_pos;
        let nr = std::cmp::min(std::cmp::max(r as i32 + dr, 0), self.nrow as i32 - 1) as usize;
        let nc = std::cmp::min(std::cmp::max(c as i32 + dc, 0), self.ncol as i32 - 1) as usize;
        self.agent_pos = (nr, nc);

        let cell = self.desc[nr][nc] as char;
        let terminated = cell == 'H' || cell == 'G';
        let truncated = !terminated && self.steps >= self.max_steps;
        let reward = if cell == 'G' { 1.0 } else { 0.0 };

        let obs = self.pos_to_state(self.agent_pos);
        let mut info = HashMap::new();
        info.insert("pos".to_string(), format!("{:?}", self.agent_pos));
        info.insert("cell".to_string(), cell.to_string());
        info.insert("action_taken".to_string(), ACTION_NAMES[action].to_string());
        info.insert("steps".to_string(), self.steps.to_string());

        StepResult::new(obs, reward, terminated, truncated)
    }

    fn observation_space(&self) -> &dyn std::any::Any {
        &self.observation_space
    }

    fn action_space(&self) -> &dyn std::any::Any {
        &self.action_space
    }

    fn render(&mut self) -> Option<String> {
        let (ar, ac) = self.agent_pos;
        let mut lines = Vec::new();

        for (r, row) in self.desc.iter().enumerate() {
            let line: String = row.iter().enumerate().map(|(c, &cell)| {
                if (r, c) == (ar, ac) {
                    "\x1b[93mA\x1b[0m".to_string()
                } else {
                    match cell {
                        b'S' => "\x1b[94mS\x1b[0m".to_string(),
                        b'F' => "\x1b[97mF\x1b[0m".to_string(),
                        b'H' => "\x1b[91mH\x1b[0m".to_string(),
                        b'G' => "\x1b[92mG\x1b[0m".to_string(),
                        _ => " ".to_string(),
                    }
                }
            }).collect();
            lines.push(line);
        }

        let out = lines.join("\n");
        println!("{}", out);
        Some(out)
    }
}