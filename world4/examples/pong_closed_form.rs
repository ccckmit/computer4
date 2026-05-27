//! world4/examples/pong_closed_form.rs - Pong closed-form controller example

use world4::{PongEnv, Env};

fn closed_form_action(obs: &[f32]) -> usize {
    let ball_y = obs[1];
    let paddle_y = obs[4];
    if ball_y > paddle_y + 0.05 {
        1
    } else if ball_y < paddle_y - 0.05 {
        0
    } else {
        0
    }
}

fn main() {
    println!("=== Pong Closed-Form Controller ===\n");

    let episodes = 10;
    let max_steps = 1000;

    let mut total_steps = 0;

    for ep in 0..episodes {
        let mut env = PongEnv::new(max_steps, None);
        let (mut obs, _) = env.reset(Some(ep));
        let mut steps = 0;

        for _ in 0..max_steps {
            let action = closed_form_action(&obs);
            let result = env.step(action);
            obs = result.observation;
            steps += 1;

            if result.terminated || result.truncated {
                break;
            }
        }

        total_steps += steps;
        println!("Episode {}: {} steps", ep + 1, steps);
    }

    println!("\nAverage steps: {:.1}", total_steps as f64 / episodes as f64);
    println!("Done!");
}