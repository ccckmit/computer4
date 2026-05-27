//! world4/src/render/test_frozen_lake.rs
//! FrozenLake Q-Learning with browser-based rendering.

use world4::{FrozenLakeEnv, Env};
use std::thread;
use std::time::Duration;

const HTML: &str = include_str!("viewer_frozen_lake.html");

const GRID: &[&str] = &["SFFF", "FHFH", "FFFH", "HFFG"];
const GRID_SIZE: usize = 4;

fn build_cells() -> Vec<char> {
    GRID.iter().flat_map(|row| row.chars()).collect()
}

fn main() {
    let server = world4::render::server::RenderServer::start_with_html(8080, HTML);

    println!("  Waiting for browser connection (4s)...");
    thread::sleep(Duration::from_secs(4));

    println!("==================================================");
    println!("  FrozenLake-v1  ·  Q-Learning  (Rust)");
    println!("==================================================\n");
    println!("Training Q-table...");
    println!("(Visualization shows final evaluation episodes)\n");

    let mut q = vec![vec![0.0f64; 4]; 16];

    let episodes = 2000;
    let alpha = 0.1;
    let gamma = 0.99;
    let epsilon = 1.0;
    let epsilon_decay = 0.995;
    let epsilon_min = 0.01;

    let mut eps = epsilon;
    for ep in 0..episodes {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep));

        for _ in 0..100 {
            let action = if rand::random::<f64>() < eps {
                rand::random::<usize>() % 4
            } else {
                let mut best = 0;
                let mut best_val = q[obs][0];
                for a in 1..4 {
                    if q[obs][a] > best_val {
                        best_val = q[obs][a];
                        best = a;
                    }
                }
                best
            };

            let result = env.step(action);
            let next_obs = result.observation;
            let reward = result.reward;

            let best_next = q[next_obs].iter().fold(0.0f64, |m, &v| m.max(v));
            q[obs][action] += alpha * (reward + gamma * best_next - q[obs][action]);

            obs = next_obs;

            if result.terminated || result.truncated {
                break;
            }
        }

        eps = eps.max(epsilon_min) * epsilon_decay;

        if ep % 500 == 0 {
            println!("  Episode {:5} | epsilon: {:.3}", ep, eps);
        }
    }

    println!("\nTraining complete! Win rate evaluation:\n");

    let mut wins = 0;
    for ep in 0..200 {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep));

        for _ in 0..100 {
            let action = {
                let mut best = 0;
                let mut best_val = q[obs][0];
                for a in 1..4 {
                    if q[obs][a] > best_val {
                        best_val = q[obs][a];
                        best = a;
                    }
                }
                best
            };

            let result = env.step(action);
            obs = result.observation;

            if result.terminated || result.truncated {
                if result.reward > 0.0 {
                    wins += 1;
                }
                break;
            }
        }
    }

    println!("  Win rate: {}/200 = {:.1}%\n", wins, wins as f64 / 2.0);
    println!("==================================================");
    println!("  Running visualization episodes...\n");

    let eval_episodes = 5;
    let cells = build_cells();

    for ep in 0..eval_episodes {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep as u64 * 1000));
        let mut total_reward = 0.0;
        let mut steps = 0;
        let mut done = false;

        for _ in 0..200 {
            let action = {
                let mut best = 0;
                let mut best_val = q[obs][0];
                for a in 1..4 {
                    if q[obs][a] > best_val {
                        best_val = q[obs][a];
                        best = a;
                    }
                }
                best
            };

            let result = env.step(action);
            obs = result.observation;
            total_reward += result.reward;
            steps += 1;
            done = result.done();

            let frame = serde_json::json!({
                "agent_pos": obs,
                "cells": cells,
                "steps": steps,
                "reward": result.reward,
                "total_reward": total_reward,
                "done": done
            });
            server.send(&frame.to_string());

            thread::sleep(Duration::from_millis(200));

            if done {
                break;
            }
        }

        println!("  Eval Episode {}: {} steps, reward: {:.2}", ep + 1, steps, total_reward);

        thread::sleep(Duration::from_secs(1));
    }

    println!("\n==================================================");
    println!("  Done! Press Ctrl+C to stop.");
    println!("==================================================");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}