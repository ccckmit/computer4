//! world4/render/test_pong.rs
//! Pong closed-form controller with browser-based rendering.

use world4::{PongEnv, Env};
use std::thread;
use std::time::Duration;

const HTML: &str = include_str!("viewer_pong.html");

fn main() {
    let server = world4::render::server::RenderServer::start_with_html(8080, HTML);

    println!("  Waiting for browser connection (4s)...");
    thread::sleep(Duration::from_secs(4));

    let episodes = 10;
    let max_steps = 1000;

    println!("==================================================");
    println!("  Pong-v1  ·  Closed-Form Controller  (Rust)");
    println!("==================================================");

    for ep in 0..episodes {
        let mut env = PongEnv::new(max_steps, None);
        let (mut obs, _) = env.reset(Some(ep as u64 * 100));
        let mut steps = 0;

        for _ in 0..max_steps {
            let ball_y = obs[1];
            let paddle_y = obs[4];
            let action: usize = if ball_y > paddle_y + 0.05 { 1 } else { 0 };

            let result = env.step(action);
            steps += 1;

            let done = result.done();
            let reward = result.reward;
            let hits = result.info.get("hits").map_or("0".to_string(), |v| v.clone());
            let ball_x = result.observation[0];
            let ball_y = result.observation[1];
            let paddle_y = result.observation[4];
            obs = result.observation;

            let frame = format!(
                r#"{{"ball_x":{:.4},"ball_y":{:.4},"paddle_y":{:.4},"hits":{},"steps":{},"reward":{},"done":{}}}"#,
                ball_x, ball_y, paddle_y, hits, steps, reward,
                if done { "true" } else { "false" }
            );
            server.send(&frame);

            thread::sleep(Duration::from_millis(33));

            if done {
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