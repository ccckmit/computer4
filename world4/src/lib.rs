//! world4 library - A lightweight RL environment framework inspired by OpenAI Gym.

pub mod core;
pub mod spaces;
pub mod envs;
pub mod wrappers;
pub mod utils;
pub mod render;

pub use core::{Env, StepResult};
pub use spaces::{Discrete, Box as SpaceBox};
pub use envs::{FrozenLakeEnv, CartPoleEnv, PongEnv};
pub use wrappers::{TimeLimitWrapper, RecordEpisodeWrapper};
pub use utils::{registry, run_random_agent};

pub fn make(id: &str) -> Option<Box<dyn Env<usize, usize>>> {
    match id {
        "FrozenLake-v0" => Some(Box::new(FrozenLakeEnv::new("4x4", None, false, None))),
        "FrozenLake-v1" => Some(Box::new(FrozenLakeEnv::new("4x4", None, true, None))),
        "FrozenLake8x8-v1" => Some(Box::new(FrozenLakeEnv::new("8x8", None, true, None))),
        "CartPole-v1" => None,
        _ => None,
    }
}

const VERSION: &str = "0.1.0";