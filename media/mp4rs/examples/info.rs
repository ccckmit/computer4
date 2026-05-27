use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <file.mp4>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    match mp4rs::open(path) {
        Ok(info) => {
            print!("{info}");
            if info.tracks.is_empty() {
                return;
            }
            // Print AVC config if available
            #[allow(unused_variables)]
            for (i, tr) in info.tracks.iter().enumerate() {
                match mp4rs::avc_config(path, i) {
                    Ok(Some(avc)) => {
                        println!("  Track {} AVC: {} SPS, {} PPS", i, avc.sps_list.len(), avc.pps_list.len());
                    }
                    _ => {}
                }
            }
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}
