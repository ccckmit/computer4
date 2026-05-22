use ffmpeg_next as ffmpeg;
use ffmpeg::format::input;
use ffmpeg::media::Type;
use ffmpeg::software::scaling::{context::Context as ScaleContext, flag::Flags};
use ffmpeg::util::format::pixel::Pixel;
use ffmpeg::util::frame::video::Video;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

fn save_ppm(frame: &Video, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride(0) as usize;
    let data = frame.data(0);
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    // PPM header (binary P6)
    write!(writer, "P6\n{} {}\n255\n", width, height)?;
    // Write pixel rows respecting stride (might have padding)
    for y in 0..height {
        let start = y * stride;
        let end = start + width * 3; // 3 bytes per pixel for RGB24
        writer.write_all(&data[start..end])?;
    }
    writer.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialise FFmpeg libraries
    ffmpeg::init().expect("Failed to init ffmpeg");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input.mpg> [frame_index output.ppm]", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];

    // Open input file
    let mut ictx = input(&input_path)?;
    // Find the best video stream
    let video_stream = ictx
        .streams()
        .best(Type::Video)
        .ok_or("No video stream found")?;
    let video_index = video_stream.index();

    // Set up decoder from stream parameters
    let codec_params = video_stream.parameters();
    let mut decoder = ffmpeg::codec::context::Context::from_parameters(codec_params)?
        .decoder()
        .video()?;

    // Prepare a scaler converting the decoded format to RGB24
    let mut scaler = ScaleContext::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::RGB24,
        decoder.width(),
        decoder.height(),
        Flags::BILINEAR,
    )?;

    // If only the input file is given, print basic info
    if args.len() == 2 {
        println!(
            "Video stream: {}x{}, pixel format: {:?}",
            decoder.width(),
            decoder.height(),
            decoder.format()
        );
        return Ok(());
    }

    // Parse frame index and output path
    if args.len() < 4 {
        eprintln!("When extracting a frame, provide both <frame_index> and <output.ppm>");
        std::process::exit(1);
    }
    let target_frame: usize = args[2]
        .parse()
        .map_err(|_| "Invalid frame index (must be an integer)")?;
    let output_path = Path::new(&args[3]);

    let mut frame_counter: usize = 0;
    let mut saved = false;
    let mut decoded = Video::empty();

    // Iterate over packets, feed decoder and check frames
    for (stream, packet) in ictx.packets() {
        if stream.index() == video_index {
            decoder.send_packet(&packet)?;
            // Pull all available decoded frames
            while decoder.receive_frame(&mut decoded).is_ok() {
                if frame_counter == target_frame {
                    let mut rgb = Video::empty();
                    scaler.run(&decoded, &mut rgb)?;
                    save_ppm(&rgb, output_path).expect("Failed to save PPM");
                    saved = true;
                    break;
                }
                frame_counter += 1;
            }
            if saved {
                break;
            }
        }
    }
    // If not yet saved, flush decoder to get remaining frames
    if !saved {
        decoder.send_eof()?;
        while decoder.receive_frame(&mut decoded).is_ok() {
            if frame_counter == target_frame {
                let mut rgb = Video::empty();
                scaler.run(&decoded, &mut rgb)?;
                save_ppm(&rgb, output_path).expect("Failed to save PPM");
                saved = true;
                break;
            }
            frame_counter += 1;
        }
    }

    if !saved {
        eprintln!("Requested frame {} not found in the file", target_frame);
    }
    Ok(())
}
