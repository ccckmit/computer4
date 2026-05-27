//! world4/examples/frozen_lake_example.rs - FrozenLake Q-Learning example

use world4::{FrozenLakeEnv, Env};

fn main() {
    println!("=== world4 · FrozenLake-v1 · Q-Learning ===\n");

    let mut q = vec![vec![0.0f64; 4]; 16];

    let episodes = 1000;
    let alpha = 0.8;
    let gamma = 0.95;
    let epsilon = 0.1;

    for ep in 0..episodes {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep));

        for _ in 0..200 {
            let action = if rand::random::<f64>() < epsilon {
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

        if ep % 200 == 0 {
            println!("Episode {:5}", ep);
        }
    }

    println!("\nTraining complete!");
    println!("\nEvaluating greedy policy...");

    let mut wins = 0;
    for ep in 0..100 {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep));

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

            if result.terminated || result.truncated {
                if result.reward > 0.0 {
                    wins += 1;
                }
                break;
            }
        }
    }

    println!("Win rate: {}/100 = {:.1}%", wins, wins as f64);
}