use std::path::Path;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input> <output>", args[0]);
        eprintln!("  Supported: .mp3 → .wav, .wav → .mp3");
        std::process::exit(1);
    }

    let in_path = Path::new(&args[1]);
    let out_path = Path::new(&args[2]);
    let in_ext = in_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let out_ext = out_path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    match (in_ext.as_str(), out_ext.as_str()) {
        ("mp3", "wav") => {
            let (info, pcm, sr, ch) = mp3rs::decode_mp3(in_path).expect("decode failed");
            let hdr = mp3rs::WavHeader {
                channels: ch as u16,
                sample_rate: sr as u32,
                bits_per_sample: 16,
                data_size: pcm.len() as u32 * 2,
            };
            mp3rs::write_wav(out_path, &hdr, &pcm).expect("write wav failed");
            println!("Decoded {} → {} ({} kbps, {} Hz, {} ch)",
                     args[1], args[2], info.bitrate_kbps, sr, ch);
        }
        ("wav", "mp3") => {
            let (hdr, pcm) = mp3rs::read_wav_header(in_path).expect("read wav failed");
            mp3rs::encode_mp3(out_path, &pcm, hdr.sample_rate, hdr.channels, 128)
                .expect("encode failed");
            println!("Encoded {} → {} ({} Hz, {} ch, 128 kbps)",
                     args[1], args[2], hdr.sample_rate, hdr.channels);
        }
        _ => {
            eprintln!("Unsupported conversion: .{} → .{}", in_ext, out_ext);
            std::process::exit(1);
        }
    }
}
