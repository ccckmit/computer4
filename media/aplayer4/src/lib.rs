// aplayer4 — Audio player library

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    _stream: OutputStream,
    handle: OutputStreamHandle,
    sink: Option<Sink>,
    volume: f32,
}

impl Player {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (_stream, handle) = OutputStream::try_default()?;
        Ok(Player { _stream, handle, sink: None, volume: 1.0 })
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
        self.stop();
        let file = File::open(path.as_ref())?;
        let source = Decoder::new(BufReader::new(file))?;
        let sink = Sink::try_new(&self.handle)?;
        sink.set_volume(self.volume);
        sink.append(source);
        self.sink = Some(sink);
        Ok(())
    }

    pub fn play(&self) {
        if let Some(ref sink) = self.sink {
            if sink.is_paused() {
                sink.play();
            }
        }
    }

    pub fn pause(&self) {
        if let Some(ref sink) = self.sink {
            sink.pause();
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
    }

    pub fn state(&self) -> State {
        match self.sink {
            None => State::Stopped,
            Some(ref s) => {
                if s.empty() {
                    State::Stopped
                } else if s.is_paused() {
                    State::Paused
                } else {
                    State::Playing
                }
            }
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.0);
        if let Some(ref sink) = self.sink {
            sink.set_volume(self.volume);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_player() -> Result<Player, Box<dyn std::error::Error>> {
        Player::new()
    }

    #[test]
    fn test_new_player() {
        match new_player() {
            Ok(_) => {} // audio device available
            Err(_) => {
                eprintln!("Skipping test: no audio device");
            }
        }
    }

    #[test]
    fn test_load_nonexistent_file() {
        let mut player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        let result = player.load("/tmp/__nonexistent_audio_file__.wav");
        assert!(result.is_err());
    }

    #[test]
    fn test_state_defaults_to_stopped() {
        let player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        assert_eq!(player.state(), State::Stopped);
    }

    #[test]
    fn test_volume_default() {
        let player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        assert!((player.volume() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_set_volume() {
        let mut player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        player.set_volume(0.5);
        assert!((player.volume() - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_set_volume_clamps() {
        let mut player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        player.set_volume(2.0);
        assert!((player.volume() - 1.0).abs() < 1e-6);
        player.set_volume(-1.0);
        assert!((player.volume() - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_stop_before_load_is_safe() {
        let mut player = match new_player() {
            Ok(p) => p,
            Err(_) => {
                eprintln!("Skipping test: no audio device");
                return;
            }
        };
        player.stop();
        assert_eq!(player.state(), State::Stopped);
    }
}
