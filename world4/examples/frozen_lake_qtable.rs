//! world4/examples/frozenlake_qtable.rs - FrozenLake Q-Table example

use world4::{FrozenLakeEnv, Env};

fn main() {
    println!("=== FrozenLake Q-Table ===\n");

    let mut q = vec![vec![0.0f64; 4]; 16];

    let episodes = 2000;
    let alpha = 0.1;
    let gamma = 0.99;
    let epsilon = 1.0;
    let epsilon_decay = 0.995;
    let epsilon_min = 0.01;

    for ep in 0..episodes {
        let mut env = FrozenLakeEnv::new("4x4", None, true, None);
        let (mut obs, _) = env.reset(Some(ep));

        for _ in 0..100 {
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

        let epsilon = epsilon.max(epsilon_min) * epsilon_decay;

        if ep % 500 == 0 {
            println!("Episode {:5} | epsilon: {:.3}", ep, epsilon);
        }
    }

    println!("\nTraining complete!");

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

    println!("Win rate: {}/200 = {:.1}%", wins, wins as f64 / 2.0);
}