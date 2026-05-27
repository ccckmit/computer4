use std::path::Path;
use std::process;

fn usage(prog: &str) -> ! {
    eprintln!("Usage:");
    eprintln!("  {prog} <file.wav>               # show WAV info");
    eprintln!("  {prog} <file.mp3>               # show MP3 info");
    eprintln!("  {prog} <in.mp3> -o <out.wav>    # decode MP3 → WAV");
    eprintln!("  {prog} <in.wav> -o <out.mp3>    # encode WAV → MP3");
    process::exit(1);
}

fn is_ext(name: &str, ext: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(&ext[1..]))
        .unwrap_or(false)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = &args[0];

    if args.len() == 2 {
        let path = Path::new(&args[1]);
        if is_ext(&args[1], ".wav") {
            match mp3rs::read_wav_header(path) {
                Ok((hdr, _)) => println!("WAV info: {hdr}"),
                Err(e) => { eprintln!("Error: {e}"); process::exit(1); }
            }
        } else if is_ext(&args[1], ".mp3") {
            match mp3rs::decode_mp3(path) {
                Ok((info, _, _, _)) => println!("MP3 info: {info}"),
                Err(e) => { eprintln!("Error: {e}"); process::exit(1); }
            }
        } else {
            usage(prog);
        }
    } else if args.len() == 4 && args[2] == "-o" {
        let in_path = Path::new(&args[1]);
        let out_path = Path::new(&args[3]);

        if is_ext(&args[1], ".mp3") && is_ext(&args[3], ".wav") {
            println!("Decoding {} → {}", args[1], args[3]);
            match mp3rs::decode_mp3(in_path) {
                Ok((info, pcm, sr, ch)) => {
                    let hdr = mp3rs::WavHeader {
                        channels: ch as u16,
                        sample_rate: sr as u32,
                        bits_per_sample: 16,
                        data_size: pcm.len() as u32 * 2,
                    };
                    mp3rs::write_wav(out_path, &hdr, &pcm).unwrap();
                    println!("Done. {info}");
                }
                Err(e) => { eprintln!("Decode error: {e}"); process::exit(1); }
            }
        } else if is_ext(&args[1], ".wav") && is_ext(&args[3], ".mp3") {
            println!("Encoding {} → {}", args[1], args[3]);
            match mp3rs::read_wav_header(in_path) {
                Ok((hdr, pcm)) => {
                    mp3rs::encode_mp3(out_path, &pcm, hdr.sample_rate, hdr.channels, 128)
                        .unwrap();
                    println!("Done.");
                }
                Err(e) => { eprintln!("Error: {e}"); process::exit(1); }
            }
        } else {
            usage(prog);
        }
    } else {
        usage(prog);
    }
}
