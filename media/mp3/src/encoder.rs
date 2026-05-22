//! MPEG Audio Layer III encoder.
//!
//! Pipeline:
//!   PCM → psychoacoustic analysis → MDCT → quantize → Huffman encode → frame packing

use std::f64::consts::PI;

use crate::AudioFrame;
use crate::bitstream::BitWriter;
use crate::dct::mdct36;
use crate::error::CodecError;
use crate::frame::{ChannelMode, FrameHeader, Layer, MpegVersion};
use crate::huffman::{encode_pair, HUFFMAN_TABLES};

/// Encoder configuration
#[derive(Debug, Clone)]
pub struct EncoderConfig {
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    pub channels: u8,
    pub quality: u8, // 0 = highest, 9 = fastest
}

impl Default for EncoderConfig {
    fn default() -> Self {
        EncoderConfig {
            bitrate_kbps: 128,
            sample_rate: 44100,
            channels: 2,
            quality: 5,
        }
    }
}

/// MP3 Encoder state
pub struct Mp3Encoder {
    pub config: EncoderConfig,
    /// PCM input buffer (per channel)
    pcm_buf: Vec<Vec<f64>>,
    /// MDCT overlap buffer (per channel × 18 subbands)
    mdct_prev: Vec<[f64; 18]>,
    /// Frame counter (for metadata)
    frame_count: u64,
}

impl Mp3Encoder {
    pub fn new(config: EncoderConfig) -> Result<Self, CodecError> {
        let ch = config.channels as usize;
        if config.bitrate_kbps == 0 {
            return Err(CodecError::EncoderError("Bitrate must be > 0".into()));
        }
        Ok(Mp3Encoder {
            pcm_buf: vec![Vec::new(); ch],
            mdct_prev: vec![[0.0f64; 18]; ch],
            frame_count: 0,
            config,
        })
    }

    /// Feed PCM samples. Returns encoded bytes when full frames are available.
    pub fn encode(&mut self, frame: &AudioFrame) -> Result<Vec<u8>, CodecError> {
        // Buffer input
        for ch in 0..self.config.channels as usize {
            if ch < frame.samples.len() {
                let f64_samples: Vec<f64> = frame.samples[ch].iter()
                    .map(|&s| s as f64 / 32768.0)
                    .collect();
                self.pcm_buf[ch].extend_from_slice(&f64_samples);
            }
        }

        let mut output = Vec::new();
        // Encode full frames (1152 samples each)
        while self.pcm_buf[0].len() >= 1152 {
            let encoded = self.encode_frame()?;
            output.extend_from_slice(&encoded);
        }
        Ok(output)
    }

    /// Flush any remaining buffered samples (zero-padded).
    pub fn flush(&mut self) -> Result<Vec<u8>, CodecError> {
        let ch = self.config.channels as usize;
        let needed = 1152usize;
        for c in 0..ch {
            while self.pcm_buf[c].len() < needed {
                self.pcm_buf[c].push(0.0);
            }
        }
        self.encode_frame()
    }

fn encode_frame(&mut self) -> Result<Vec<u8>, CodecError> {
let ch = self.config.channels as usize;
let samples: Vec<Vec<f64>> = (0..ch)
.map(|c| self.pcm_buf[c].drain(..1152).collect())
.collect();

// ---- Determine MPEG version based on sample rate ----
let version = match self.config.sample_rate {
44100 | 48000 | 32000 => MpegVersion::Mpeg1,
22050 | 24000 | 16000 => MpegVersion::Mpeg2,
11025 | 12000 | 8000 => MpegVersion::Mpeg25,
_ => MpegVersion::Mpeg1, // Default to MPEG1
};

// ---- Build header ----
let header = FrameHeader {
version,
layer: Layer::Layer3,
bitrate_kbps: self.config.bitrate_kbps,
sample_rate: self.config.sample_rate,
padding: false,
channel_mode: if ch == 1 { ChannelMode::Mono } else { ChannelMode::JointStereo },
mode_extension: 0,
copyright: false,
original: true,
};

        let frame_size = header.frame_size();
        let mut writer = BitWriter::with_capacity(frame_size);

        // Write header bytes
        let hdr_bytes = header.encode();
        for b in hdr_bytes { writer.write_bits(b as u32, 8); }

        // ---- Side information (simplified, stereo) ----
        writer.write_bits(0, 9);  // main_data_begin = 0
        writer.write_bits(0, if ch == 2 { 3 } else { 5 }); // private_bits
        for _ in 0..ch {
            for _ in 0..4 { writer.write_bit(false); } // scfsi
        }

        // Write granule side info (2 granules × channels)
        // We use fixed parameters for simplicity
        let gran_bits: u16 = 100; // placeholder
        for _gran in 0..2 {
            for _c in 0..ch {
                writer.write_bits(gran_bits as u32, 12); // part2_3_length
                writer.write_bits(0, 9);   // big_values = 0
                writer.write_bits(200, 8); // global_gain
                writer.write_bits(0, 4);   // scalefac_compress
                writer.write_bit(false);   // window_switching = false
                writer.write_bits(0, 5);   // table_select[0]
                writer.write_bits(0, 5);   // table_select[1]
                writer.write_bits(0, 5);   // table_select[2]
                writer.write_bits(3, 4);   // region0_count
                writer.write_bits(3, 3);   // region1_count
                writer.write_bit(false);   // preflag
                writer.write_bit(false);   // scalefac_scale
                writer.write_bit(false);   // count1table_select
            }
        }
        writer.align_byte();

        // ---- Main data: MDCT + quantize + Huffman ----
        for gran in 0..2usize {
            for c in 0..ch {
                let gran_start = gran * 576;
                let gran_end = (gran_start + 576).min(samples[c].len());
                let gran_slice = &samples[c][gran_start..gran_end];

                // Apply MDCT over 32 subbands of 18 samples each
                let mdct_out = self.mdct_frame(gran_slice, c);

                // Simple quantization: scale to integers
                let global_gain = 200i32;
                let scale = 2.0_f64.powf(0.25 * (global_gain as f64 - 210.0));

                let quantized: Vec<i32> = mdct_out.iter()
                    .map(|&x| {
                        let q = (x.abs() / scale).powf(0.75).round() as i32;
                        if x < 0.0 { -q } else { q }
                    })
                    .collect();

                // Write scale factors (all zero for simplicity)
                for _ in 0..22 { writer.write_bits(0, 4); }

                // Huffman encode pairs
                let tbl = &HUFFMAN_TABLES[1]; // table 2
                for i in (0..quantized.len().min(576)).step_by(2) {
                    let x = quantized[i];
                    let y = if i + 1 < quantized.len() { quantized[i + 1] } else { 0 };
                    let _ = encode_pair(&mut writer, x, y, tbl); // ignore errors for corrupt frames
                }
            }
        }

        writer.align_byte();
        let mut encoded = writer.into_bytes();

        // Pad or trim to exact frame size
        encoded.resize(frame_size, 0);
        self.frame_count += 1;
        Ok(encoded)
    }

    /// Apply MDCT to one granule of PCM data (576 samples → 576 freq coefficients).
    fn mdct_frame(&mut self, pcm: &[f64], ch: usize) -> Vec<f64> {
        let mut result = vec![0.0f64; 576];
        // Apply 36-point MDCT in blocks of 36 (overlap by 18)
        for sb in 0..32 {
            let start = sb * 18;
            // Build 36-point input: 18 from previous + 18 new
            let mut block = [0.0f64; 36];
            for i in 0..18 {
                block[i] = self.mdct_prev[ch][i]; // previous overlap
            }
            for i in 0..18 {
                let idx = start + i;
                block[18 + i] = if idx < pcm.len() { pcm[idx] } else { 0.0 };
            }

            // Apply analysis window (normal block)
            for i in 0..36 {
                block[i] *= (PI / 36.0 * (i as f64 + 0.5)).sin();
            }

            let mdct_out = mdct36(&block);

            // Store second half as overlap
            if ch < self.mdct_prev.len() {
                for i in 0..18 {
                    self.mdct_prev[ch][i] = block[18 + i];
                }
            }

            for k in 0..18 {
                let out_idx = start + k;
                if out_idx < 576 { result[out_idx] = mdct_out[k]; }
            }
        }
        result
    }

    /// Generate an ID3v1 tag (128 bytes) for metadata.
    pub fn id3v1_tag(title: &str, artist: &str, album: &str, year: &str) -> [u8; 128] {
        let mut tag = [0u8; 128];
        tag[0] = b'T'; tag[1] = b'A'; tag[2] = b'G';
        let copy_field = |dst: &mut [u8], src: &str| {
            for (i, c) in src.bytes().enumerate().take(dst.len()) { dst[i] = c; }
        };
        copy_field(&mut tag[3..33], title);
        copy_field(&mut tag[33..63], artist);
        copy_field(&mut tag[63..93], album);
        copy_field(&mut tag[93..97], year);
        tag[127] = 0xFF; // genre = unknown
        tag
    }
}
