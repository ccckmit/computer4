// Simplified JPEG/PPM utility using the `image` crate
// Supports encoding a PPM (P6) image to JPEG and decoding JPEG back to PPM.

use std::env;
use std::path::Path;
use std::process;
use image::{io::Reader as ImageReader, ImageFormat};

fn usage(prog: &str) -> ! {
    eprintln!("Usage: {} <encode|decode> <input> <output>", prog);
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        usage(&args[0]);
    }

    let mode = &args[1];
    let input_path = Path::new(&args[2]);
    let output_path = Path::new(&args[3]);

    match mode.as_str() {
        "encode" => encode_ppm_to_jpeg(input_path, output_path),
        "decode" => decode_jpeg_to_ppm(input_path, output_path),
        _ => {
            eprintln!("Unknown mode: {}. Use 'encode' or 'decode'.", mode);
            process::exit(1);
        }
    }
}

fn encode_ppm_to_jpeg(input: &Path, output: &Path) {
    // Load PPM (PNM) image using the `image` crate.
    let img = ImageReader::open(input)
        .expect("Failed to open input file")
        .with_guessed_format()
        .expect("Failed to guess image format")
        .decode()
        .expect("Failed to decode PPM image");

    // Save the image as JPEG.
    img.save_with_format(output, ImageFormat::Jpeg)
        .expect("Failed to write JPEG file");
    println!("Encoded {} -> {}", input.display(), output.display());
}

fn decode_jpeg_to_ppm(input: &Path, output: &Path) {
    // Load JPEG image.
    let img = ImageReader::open(input)
        .expect("Failed to open input file")
        .with_guessed_format()
        .expect("Failed to guess image format")
        .decode()
        .expect("Failed to decode JPEG image");

    // Convert to raw RGB8 buffer.
    let rgb = img.to_rgb8();

    // Write a simple PPM (P6) file.
    use std::io::Write;
    let mut file = std::fs::File::create(output).expect("Failed to create output file");
    // Header.
    writeln!(file, "P6").unwrap();
    writeln!(file, "{} {}", rgb.width(), rgb.height()).unwrap();
    writeln!(file, "255").unwrap();
    // Pixel data.
    file.write_all(&rgb).expect("Failed to write pixel data");
    println!("Decoded {} -> {}", input.display(), output.display());
}
