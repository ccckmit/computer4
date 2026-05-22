//! MPEG Audio Codec — command-line demo.
//!
//! Usage:
//!   mpeg_codec decode <input.mp3> <output.pcm>
//!   mpeg_codec encode <input.pcm> <output.mp3> [--bitrate 128] [--sample-rate 44100] [--channels 2]
//!   mpeg_codec info   <input.mp3>

use std::env;
use std::fs;
use std::process;

use mpeg_codec::{
    AudioFrame, Mp3Decoder, Mp3Encoder,
    encoder::EncoderConfig,
    frame::FrameHeader,
    error::CodecError,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let result = match args[1].as_str() {
        "decode" => cmd_decode(&args),
        "encode" => cmd_encode(&args),
        "info"   => cmd_info(&args),
        other    => {
            eprintln!("Unknown command: {}", other);
            print_usage(&args[0]);
            process::exit(1);
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_usage(prog: &str) {
    println!("MPEG Audio Codec v0.1.0");
    println!();
    println!("Usage:");
    println!("  {} decode <input.mp3> <output.pcm>", prog);
    println!("  {} encode <input.pcm> <output.mp3> [--bitrate 128] [--sr 44100] [--ch 2]", prog);
    println!("  {} info   <input.mp3>", prog);
}

// ─── Decode ──────────────────────────────────────────────────────────────────

fn cmd_decode(args: &[String]) -> Result<(), CodecError> {
    if args.len() < 4 {
        eprintln!("Usage: decode <input.mp3> <output.pcm>");
        return Err(CodecError::IoError("Missing arguments".into()));
    }

    let input_path  = &args[2];
    let output_path = &args[3];

    println!("Decoding '{}' → '{}'", input_path, output_path);

    let data = fs::read(input_path)
        .map_err(|e| CodecError::IoError(e.to_string()))?;

    let mut decoder = Mp3Decoder::new();
    let frames = decoder.decode_all(&data);

    if frames.is_empty() {
        println!("No frames decoded (invalid or unsupported file).");
        return Ok(());
    }

    let total_samples: usize = frames.iter().map(|f| f.samples[0].len()).sum();
    let channels = frames[0].channels;
    let sample_rate = frames[0].sample_rate;

    println!(
        "  Decoded {} frames | {} channels | {} Hz | {} total samples | {:.2} s",
        frames.len(),
        channels,
        sample_rate,
        total_samples,
        total_samples as f64 / sample_rate as f64
    );

    // Write raw 16-bit little-endian PCM (interleaved)
    let mut pcm_bytes = Vec::with_capacity(total_samples * channels as usize * 2);
    for frame in &frames {
        let len = frame.samples[0].len();
        for s in 0..len {
            for ch in 0..channels as usize {
                let sample = frame.samples.get(ch).and_then(|c| c.get(s)).copied().unwrap_or(0);
                pcm_bytes.extend_from_slice(&sample.to_le_bytes());
            }
        }
    }

    fs::write(output_path, &pcm_bytes)
        .map_err(|e| CodecError::IoError(e.to_string()))?;

    println!("  Written {} bytes of PCM to '{}'", pcm_bytes.len(), output_path);
    Ok(())
}

// ─── Encode ──────────────────────────────────────────────────────────────────

fn cmd_encode(args: &[String]) -> Result<(), CodecError> {
    if args.len() < 4 {
        eprintln!("Usage: encode <input.pcm> <output.mp3> [--bitrate N] [--sr N] [--ch N]");
        return Err(CodecError::IoError("Missing arguments".into()));
    }

    let input_path  = &args[2];
    let output_path = &args[3];

    // Parse optional flags
    let mut bitrate     = 128u32;
    let mut sample_rate = 44100u32;
    let mut channels    = 2u8;
    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "--bitrate" | "-b" => { i += 1; bitrate     = args[i].parse().unwrap_or(128); }
            "--sr"             => { i += 1; sample_rate = args[i].parse().unwrap_or(44100); }
            "--ch"             => { i += 1; channels    = args[i].parse().unwrap_or(2); }
            _                  => {}
        }
        i += 1;
    }

    println!(
        "Encoding '{}' → '{}' [{}kbps, {}Hz, {}ch]",
        input_path, output_path, bitrate, sample_rate, channels
    );

    let raw = fs::read(input_path)
        .map_err(|e| CodecError::IoError(e.to_string()))?;

    // Convert raw bytes to i16 samples (little-endian, interleaved)
    let samples_i16: Vec<i16> = raw.chunks_exact(2)
        .map(|c| i16::from_le_bytes([c[0], c[1]]))
        .collect();

    let samples_per_ch = samples_i16.len() / channels as usize;
    let mut frame = AudioFrame::new(channels, samples_per_ch, sample_rate);
    for (i, &s) in samples_i16.iter().enumerate() {
        let ch = i % channels as usize;
        let pos = i / channels as usize;
        if pos < frame.samples[ch].len() {
            frame.samples[ch][pos] = s;
        }
    }

    let config = EncoderConfig { bitrate_kbps: bitrate, sample_rate, channels, quality: 5 };
    let mut encoder = Mp3Encoder::new(config)?;

    let mut encoded = encoder.encode(&frame)?;
    encoded.extend_from_slice(&encoder.flush()?);

    // Append ID3v1 tag
    let tag = Mp3Encoder::id3v1_tag("Unknown", "Unknown", "Unknown", "2024");
    encoded.extend_from_slice(&tag);

    fs::write(output_path, &encoded)
        .map_err(|e| CodecError::IoError(e.to_string()))?;

    println!("  Written {} bytes ({:.1} kB) to '{}'", encoded.len(), encoded.len() as f64 / 1024.0, output_path);
    Ok(())
}

// ─── Info ─────────────────────────────────────────────────────────────────────

fn cmd_info(args: &[String]) -> Result<(), CodecError> {
    if args.len() < 3 {
        return Err(CodecError::IoError("Missing input file".into()));
    }
    let data = fs::read(&args[2]).map_err(|e| CodecError::IoError(e.to_string()))?;

    println!("File: {} ({} bytes)", args[2], data.len());

    // Scan for first valid frame header
    let mut frame_count = 0u32;
    let mut pos = 0usize;
    let mut first_header: Option<FrameHeader> = None;

    while pos + 4 <= data.len() {
        if data[pos] == 0xFF && (data[pos + 1] & 0xE0) == 0xE0 {
            if let Ok(hdr) = FrameHeader::parse(&data[pos..]) {
                let fs = hdr.frame_size();
                if fs == 0 { pos += 1; continue; }
                if first_header.is_none() {
                    first_header = Some(hdr.clone());
                }
                frame_count += 1;
                pos += fs;
                continue;
            }
        }
        pos += 1;
    }

    if let Some(hdr) = first_header {
        println!("  MPEG version  : {:?}", hdr.version);
        println!("  Layer         : {:?}", hdr.layer);
        println!("  Bitrate       : {} kbps", hdr.bitrate_kbps);
        println!("  Sample rate   : {} Hz", hdr.sample_rate);
        println!("  Channels      : {} ({:?})", hdr.channels(), hdr.channel_mode);
        println!("  Frame count   : {}", frame_count);
        let secs = frame_count as f64 * hdr.samples_per_frame() as f64 / hdr.sample_rate as f64;
        println!("  Duration      : {:.2} s ({:.2} min)", secs, secs / 60.0);
        println!("  Padding       : {}", hdr.padding);
        println!("  Copyright     : {}", hdr.copyright);
        println!("  Original      : {}", hdr.original);
    } else {
        println!("  No valid MPEG frames found.");
    }

    Ok(())
}
