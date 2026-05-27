//! world4/core.rs
//! Core abstractions: the base Env trait and StepResult struct.

use std::any::Any;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct StepResult<ObsType> {
    pub observation: ObsType,
    pub reward: f64,
    pub terminated: bool,
    pub truncated: bool,
    pub info: std::collections::HashMap<String, String>,
}

impl<ObsType> StepResult<ObsType> {
    pub fn new(observation: ObsType, reward: f64, terminated: bool, truncated: bool) -> Self {
        StepResult {
            observation,
            reward,
            terminated,
            truncated,
            info: std::collections::HashMap::new(),
        }
    }

    pub fn done(&self) -> bool {
        self.terminated || self.truncated
    }

    pub fn into_tuple(self) -> (ObsType, f64, bool, bool, std::collections::HashMap<String, String>) {
        (self.observation, self.reward, self.terminated, self.truncated, self.info)
    }
}

impl<ObsType> Into<(ObsType, f64, bool, bool, std::collections::HashMap<String, String>)> for StepResult<ObsType> {
    fn into(self) -> (ObsType, f64, bool, bool, std::collections::HashMap<String, String>) {
        self.into_tuple()
    }
}

pub trait Env<ObsType: 'static, ActType> {
    fn reset(&mut self, seed: Option<u64>) -> (ObsType, std::collections::HashMap<String, String>);
    fn step(&mut self, action: ActType) -> StepResult<ObsType>;

    fn observation_space(&self) -> &dyn Any;
    fn action_space(&self) -> &dyn Any;

    fn render(&mut self) -> Option<String> { None }
    fn close(&mut self) {}
    fn seed(&mut self, seed: Option<u64>) -> u64 { seed.unwrap_or(0) }

    fn metadata(&self) -> std::collections::HashMap<String, String> {
        std::collections::HashMap::new()
    }

    fn reward_range(&self) -> (f64, f64) {
        (f64::NEG_INFINITY, f64::INFINITY)
    }
}

pub trait Space {
    fn sample(&mut self) -> Box<dyn std::any::Any>;
    fn contains(&self, x: &dyn std::any::Any) -> bool;
    fn shape(&self) -> Vec<usize>;
    fn dtype(&self) -> &str;
}

impl Debug for dyn Env<(), ()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Env")
    }
}