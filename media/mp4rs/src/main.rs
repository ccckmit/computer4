use std::path::Path;
use std::process;

fn usage(prog: &str) -> ! {
    eprintln!("Usage:");
    eprintln!("  {prog} <file.mp4>                               # show MP4 info");
    eprintln!("  {prog} <file.mp4> <track> <frame> <output>      # extract raw frame");
    eprintln!("  {prog} <file.mp4> annex-b <track> <frame> <out> # extract as Annex-B");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let prog = &args[0];

    if args.len() == 2 {
        let path = Path::new(&args[1]);
        match mp4rs::open(path) {
            Ok(info) => print!("{info}"),
            Err(e) => { eprintln!("Error: {e}"); process::exit(1); }
        }
    } else if args.len() == 6 && args[2] == "annex-b" {
        let path = Path::new(&args[1]);
        let track: usize = args[3].parse().unwrap_or_else(|_| usage(prog));
        let frame: u32 = args[4].parse().unwrap_or_else(|_| usage(prog));
        let output = Path::new(&args[5]);

        let avc = mp4rs::avc_config(path, track).unwrap_or_else(|e| {
            eprintln!("Error reading AVC config: {e}"); process::exit(1);
        }).unwrap_or_else(|| {
            eprintln!("No AVC config found for track {track}"); process::exit(1);
        });

        let (frame_data, _) = mp4rs::read_frame_annex_b(path, track, frame)
            .unwrap_or_else(|e| { eprintln!("Error: {e}"); process::exit(1); });

        let stream = mp4rs::build_annex_b_stream(&avc, &frame_data);
        std::fs::write(output, &stream).unwrap_or_else(|e| { eprintln!("Write error: {e}"); process::exit(1); });
        println!("Extracted frame {frame} (track {track}) to {}", output.display());
    } else if args.len() == 5 {
        let path = Path::new(&args[1]);
        let track: usize = args[2].parse().unwrap_or_else(|_| usage(prog));
        let frame: u32 = args[3].parse().unwrap_or_else(|_| usage(prog));
        let output = Path::new(&args[4]);

        let (raw, _) = mp4rs::read_frame(path, track, frame)
            .unwrap_or_else(|e| { eprintln!("Error: {e}"); process::exit(1); });
        std::fs::write(output, &raw).unwrap_or_else(|e| { eprintln!("Write error: {e}"); process::exit(1); });
        println!("Extracted frame {frame} (track {track}) to {}", output.display());
    } else {
        usage(prog);
    }
}
