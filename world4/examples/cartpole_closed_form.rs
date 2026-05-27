//! world4/examples/cartpole_closed_form.rs - CartPole controller example

use world4::{CartPoleEnv, Env};

fn main() {
    println!("=== CartPole Closed-Form Controller ===\n");

    let episodes = 10;
    let max_steps = 500;

    for ep in 0..episodes {
        let mut env = CartPoleEnv::new(max_steps, None);
        let (mut obs, _) = env.reset(Some(ep));
        let mut steps = 0;

        for _ in 0..max_steps {
            let action = if obs[2] > 0.0 {
                if obs[3] > 0.01 { 1 } else { 0 }
            } else {
                if obs[3] < -0.01 { 0 } else { 1 }
            };

            let result = env.step(action);
            obs = result.observation;
            steps += 1;

            if result.terminated || result.truncated {
                println!("Episode {}: {} steps", ep, steps);
                break;
            }
        }
    }

    println!("\nDone!");
}