use std::env;
use std::fs;
use std::path::Path;

use jpeg::{JpegEncoder, JpegDecoder, rgb_to_ycbcr, ycbcr_to_rgb};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <encode|decode> <input> <output>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let input_path = &args[2];
    let output_path = &args[3];

    match mode.as_str() {
        "encode" => ppm_to_jpeg(input_path, output_path),
        "decode" => jpeg_to_ppm(input_path, output_path),
        _ => {
            eprintln!("Unknown mode: {}. Use 'encode' or 'decode'.", mode);
            std::process::exit(1);
        }
    }
}

fn jpeg_to_ppm(input_path: &str, output_path: &str) {
    let data = fs::read(input_path).expect("Failed to read input JPEG file");

    let mut decoder = JpegDecoder::new();
    let rgb = decoder.decode_to_rgb(&data).expect("Failed to decode JPEG");

    let width = decoder.get_width();
    let height = decoder.get_height();

    let header = format!("P6\n{} {}\n255\n", width, height);
    let mut ppm = header.as_bytes().to_vec();
    ppm.extend_from_slice(&rgb);

    fs::write(output_path, &ppm).expect("Failed to write PPM file");
    eprintln!("Decoded {} -> {} ({}x{})", input_path, output_path, width, height);
}

fn ppm_to_jpeg(input_path: &str, output_path: &str) {
    let data = fs::read(input_path).expect("Failed to read input PPM file");

    let (width, height, rgb) = parse_ppm(&data);

    let y_data = rgb_to_y_full(&rgb, width, height);
    let (cb_data, cr_data) = rgb_to_cbcr_subsampled(&rgb, width, height);

    let encoder = JpegEncoder::new(width, height);
    let jpeg = encoder.encode(&y_data, &cb_data, &cr_data);

    fs::write(output_path, &jpeg).expect("Failed to write JPEG file");
    eprintln!("Encoded {} -> {} ({}x{})", input_path, output_path, width, height);
}

fn parse_ppm(data: &[u8]) -> (usize, usize, Vec<u8>) {
    let mut pos = 0;
    while data[pos] == b'P' {
        break;
    }
    pos += 2;

    pos = skip_whitespace(data, pos);
    let (width, pos) = parse_usize(data, pos);
    let pos = skip_whitespace(data, pos);
    let (height, pos) = parse_usize(data, pos);
    let pos = skip_whitespace(data, pos);
    let (_maxval, mut pos) = parse_usize(data, pos);

    if data[pos] == b'\n' || data[pos] == b' ' || data[pos] == b'\t' {
        pos += 1;
    }

    let pixel_count = width * height * 3;
    let rgb = data[pos..pos + pixel_count].to_vec();

    (width, height, rgb)
}

fn skip_whitespace(data: &[u8], mut pos: usize) -> usize {
    while pos < data.len() && (data[pos] == b' ' || data[pos] == b'\t' || data[pos] == b'\n' || data[pos] == b'\r') {
        pos += 1;
    }
    if pos < data.len() && data[pos] == b'#' {
        while pos < data.len() && data[pos] != b'\n' {
            pos += 1;
        }
        pos = skip_whitespace(data, pos);
    }
    pos
}

fn parse_usize(data: &[u8], mut pos: usize) -> (usize, usize) {
    let start = pos;
    while pos < data.len() && data[pos].is_ascii_digit() {
        pos += 1;
    }
    let s = std::str::from_utf8(&data[start..pos]).unwrap();
    (s.parse().unwrap(), pos)
}

fn rgb_to_y_full(rgb: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut y = Vec::with_capacity(width * height);
    for i in 0..width * height {
        let r = rgb[i * 3];
        let g = rgb[i * 3 + 1];
        let b = rgb[i * 3 + 2];
        let (y_val, _, _) = rgb_to_ycbcr(r, g, b);
        y.push(y_val);
    }
    y
}

fn rgb_to_cbcr_subsampled(rgb: &[u8], width: usize, height: usize) -> (Vec<u8>, Vec<u8>) {
    let mcus_x = (width + 15) / 16;
    let mcus_y = (height + 15) / 16;
    let cw = mcus_x * 8;
    let ch = mcus_y * 8;
    let mut cb = vec![128u8; cw * ch];
    let mut cr = vec![128u8; cw * ch];

    for y in 0..ch {
        for x in 0..cw {
            let img_x = x * 2;
            let img_y = y * 2;
            if img_x < width && img_y < height {
                let mut sum_cb = 0i32;
                let mut sum_cr = 0i32;
                let mut count = 0;
                for dy in 0..2 {
                    for dx in 0..2 {
                        let px = img_x + dx;
                        let py = img_y + dy;
                        if px < width && py < height {
                            let idx = (py * width + px) * 3;
                            let (_, cbi, cri) = rgb_to_ycbcr(rgb[idx], rgb[idx + 1], rgb[idx + 2]);
                            sum_cb += cbi as i32;
                            sum_cr += cri as i32;
                            count += 1;
                        }
                    }
                }
                cb[y * cw + x] = (sum_cb / count) as u8;
                cr[y * cw + x] = (sum_cr / count) as u8;
            }
        }
    }

    (cb, cr)
}
