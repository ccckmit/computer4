use std::path::Path;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.mp3|file.wav>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext.to_lowercase().as_str() {
        "mp3" => {
            match mp3rs::decode_mp3(path) {
                Ok((info, _, sr, ch)) => {
                    println!("MP3 file: {}", path.display());
                    println!("  bitrate: {} kbps", info.bitrate_kbps);
                    println!("  sample rate: {} Hz", sr);
                    println!("  channels: {}", ch);
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        "wav" => {
            match mp3rs::read_wav_header(path) {
                Ok((hdr, pcm)) => {
                    let dur = pcm.len() as f64 / hdr.sample_rate as f64 / hdr.channels as f64;
                    println!("WAV file: {}", path.display());
                    println!("  channels: {}", hdr.channels);
                    println!("  sample rate: {} Hz", hdr.sample_rate);
                    println!("  bits: {}", hdr.bits_per_sample);
                    println!("  samples: {}", pcm.len() / hdr.channels as usize);
                    println!("  duration: {:.2}s", dur);
                }
                Err(e) => eprintln!("Error: {e}"),
            }
        }
        _ => eprintln!("Unsupported file: {}", path.display()),
    }
}
