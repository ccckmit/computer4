// aplayer4 — Audio player CLI

use std::time::Duration;

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use std::io::{stdout, Write};

use aplayer4::{Player, State};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <audio-file>", args[0]);
        std::process::exit(1);
    }

    let path = &args[1];
    let mut player = Player::new()?;
    player.load(path)?;
    player.play();

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let result = run_ui(&mut player, &mut stdout);

    execute!(stdout, LeaveAlternateScreen, Show)?;
    disable_raw_mode()?;

    result
}

fn run_ui(player: &mut Player, stdout: &mut impl Write) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let state = player.state();
        write!(stdout, "\r\x1b[2K")?;
        let icon = match state {
            State::Playing => "[> Playing]",
            State::Paused  => "[| Paused ]",
            State::Stopped => "[x Stopped]",
        };
        write!(
            stdout,
            "{}  Vol: {:.0}%  [Space] pause/resume  [+/-] vol  [q] quit",
            icon,
            player.volume() * 100.0,
        )?;
        stdout.flush()?;

        if state == State::Stopped {
            break;
        }

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char(' ') => {
                        match player.state() {
                            State::Playing => player.pause(),
                            _ => player.play(),
                        }
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        let v = (player.volume() + 0.1).min(1.0);
                        player.set_volume(v);
                    }
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        let v = (player.volume() - 0.1).max(0.0);
                        player.set_volume(v);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
