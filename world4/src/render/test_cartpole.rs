//! world4/render/test_cartpole.rs
//! CartPole-v1 closed-form controller with browser-based rendering.

use world4::{CartPoleEnv, Env};
use std::thread;
use std::time::Duration;

fn main() {
    let server = world4::render::server::RenderServer::start(8080);

    println!("  Waiting for browser connection (4s)...");
    thread::sleep(Duration::from_secs(4));

    let episodes = 10;
    let max_steps = 500;

    println!("==================================================");
    println!("  CartPole-v1  ·  Closed-Form Controller  (Rust)");
    println!("==================================================");

    for ep in 0..episodes {
        let mut env = CartPoleEnv::new(max_steps, None);
        let (obs, _) = env.reset(Some(ep as u64 * 100));
        let mut obs_arr = [0.0f32; 4];
        for (i, &v) in obs.iter().enumerate() {
            obs_arr[i] = v;
        }
        let mut steps = 0;

        for _ in 0..max_steps {
            let theta = obs_arr[2];
            let theta_dot = obs_arr[3];
            let action: usize = if theta > 0.0 {
                if theta_dot > 0.01 { 1 } else { 0 }
            } else {
                if theta_dot < -0.01 { 0 } else { 1 }
            };

            let result = env.step(action);
            for (i, &v) in result.observation.iter().enumerate() {
                obs_arr[i] = v;
            }
            steps += 1;

            let frame = format!(
                r#"{{"x":{:.4},"theta":{:.4},"steps":{},"reward":1,"done":false}}"#,
                obs_arr[0], obs_arr[2], steps
            );
            server.send(&frame);

            thread::sleep(Duration::from_millis(33));

            if result.terminated || result.truncated {
                break;
            }
        }

        println!("  Episode {}: {} steps", ep + 1, steps);
    }

    println!("==================================================");
    println!("  Done! Press Ctrl+C to stop.");
    println!("==================================================");

    loop {
        thread::sleep(Duration::from_secs(1));
    }
}