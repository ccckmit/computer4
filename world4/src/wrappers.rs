//! world4/wrappers.rs
//! Environment wrappers.

use std::any::Any;
use std::collections::HashMap;
use crate::core::{Env, StepResult};

pub struct TimeLimitWrapper<ObsType, ActType> {
    env: Box<dyn Env<ObsType, ActType>>,
    max_steps: usize,
    elapsed: usize,
}

impl<ObsType: 'static, ActType: 'static> TimeLimitWrapper<ObsType, ActType> {
    pub fn new(env: Box<dyn Env<ObsType, ActType>>, max_steps: usize) -> Self {
        TimeLimitWrapper {
            env,
            max_steps,
            elapsed: 0,
        }
    }
}

impl<ObsType: 'static, ActType: 'static> Env<ObsType, ActType> for TimeLimitWrapper<ObsType, ActType> {
    fn reset(&mut self, seed: Option<u64>) -> (ObsType, HashMap<String, String>) {
        self.elapsed = 0;
        self.env.reset(seed)
    }

    fn step(&mut self, action: ActType) -> StepResult<ObsType> {
        let result = self.env.step(action);
        self.elapsed += 1;

        if self.elapsed >= self.max_steps && !result.terminated {
            StepResult::new(result.observation, result.reward, result.terminated, true)
        } else {
            result
        }
    }

    fn observation_space(&self) -> &dyn Any {
        self.env.observation_space()
    }

    fn action_space(&self) -> &dyn Any {
        self.env.action_space()
    }

    fn render(&mut self) -> Option<String> {
        self.env.render()
    }

    fn close(&mut self) {
        self.env.close();
    }
}

pub struct RecordEpisodeWrapper<ObsType, ActType> {
    env: Box<dyn Env<ObsType, ActType>>,
    episodes: Vec<EpisodeData<ObsType, ActType>>,
    current_episode: Option<EpisodeData<ObsType, ActType>>,
}

struct EpisodeData<ObsType, ActType> {
    observations: Vec<ObsType>,
    actions: Vec<ActType>,
    rewards: Vec<f64>,
}

impl<ObsType: 'static, ActType: 'static> RecordEpisodeWrapper<ObsType, ActType> {
    pub fn new(env: Box<dyn Env<ObsType, ActType>>) -> Self {
        RecordEpisodeWrapper {
            env,
            episodes: Vec::new(),
            current_episode: None,
        }
    }

    pub fn get_episodes(&self) -> &Vec<EpisodeData<ObsType, ActType>> {
        &self.episodes
    }
}

impl<ObsType: 'static + Clone, ActType: 'static + Clone> Env<ObsType, ActType> for RecordEpisodeWrapper<ObsType, ActType> {
    fn reset(&mut self, seed: Option<u64>) -> (ObsType, HashMap<String, String>) {
        if let Some(ep) = self.current_episode.take() {
            self.episodes.push(ep);
        }
        self.current_episode = Some(EpisodeData {
            observations: Vec::new(),
            actions: Vec::new(),
            rewards: Vec::new(),
        });
        self.env.reset(seed)
    }

    fn step(&mut self, action: ActType) -> StepResult<ObsType> {
        let result = self.env.step(action.clone());

        if let Some(ref mut ep) = self.current_episode {
            ep.actions.push(action);
            ep.rewards.push(result.reward);

            if result.done() {
                let finished = EpisodeData {
                    observations: Vec::new(),
                    actions: ep.actions.clone(),
                    rewards: ep.rewards.clone(),
                };
                self.episodes.push(finished);
                self.current_episode = None;
            }
        }

        result
    }

    fn observation_space(&self) -> &dyn Any {
        self.env.observation_space()
    }

    fn action_space(&self) -> &dyn Any {
        self.env.action_space()
    }

    fn render(&mut self) -> Option<String> {
        self.env.render()
    }

    fn close(&mut self) {
        self.env.close();
    }
}