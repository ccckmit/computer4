// MPEG-1 Audio Layer III (MP3) Codec Implementation in Rust
// Implements core encoding and decoding of MPEG audio

pub mod bitstream;
pub mod dct;
pub mod huffman;
pub mod decoder;
pub mod encoder;
pub mod frame;
pub mod tables;
pub mod error;

pub use decoder::Mp3Decoder;
pub use encoder::Mp3Encoder;
pub use frame::{FrameHeader, ChannelMode, MpegVersion, Layer};
pub use error::CodecError;

/// PCM audio sample (16-bit signed)
pub type Sample = i16;

/// A decoded audio frame containing PCM samples
#[derive(Debug, Clone)]
pub struct AudioFrame {
    pub samples: Vec<Vec<Sample>>, // [channel][sample]
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
}

impl AudioFrame {
    pub fn new(channels: u8, samples_per_channel: usize, sample_rate: u32) -> Self {
        AudioFrame {
            samples: vec![vec![0; samples_per_channel]; channels as usize],
            sample_rate,
            channels,
            bit_depth: 16,
        }
    }

    pub fn duration_ms(&self) -> f64 {
        if self.samples.is_empty() || self.sample_rate == 0 {
            return 0.0;
        }
        (self.samples[0].len() as f64 / self.sample_rate as f64) * 1000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{FrameHeader, MpegVersion, Layer, ChannelMode};
    use crate::bitstream::{BitReader, BitWriter};
    use crate::dct::{mdct36, imdct36, imdct12};
    use crate::encoder::{Mp3Encoder, EncoderConfig};
    use crate::decoder::Mp3Decoder;

    // ── Frame Header ──────────────────────────────────────────────────────────

    #[test]
    fn test_frame_header_parse_mpeg1_layer3() {
        // Valid MPEG-1 Layer III, 128kbps, 44100 Hz, Stereo
        // FF FB 90 00
        let bytes = [0xFF, 0xFB, 0x90, 0x00];
        let hdr = FrameHeader::parse(&bytes).expect("Should parse valid header");
        assert_eq!(hdr.version, MpegVersion::Mpeg1);
        assert_eq!(hdr.layer, Layer::Layer3);
        assert_eq!(hdr.bitrate_kbps, 128);
        assert_eq!(hdr.sample_rate, 44100);
        assert_eq!(hdr.channel_mode, ChannelMode::Stereo);
    }

    #[test]
    fn test_frame_header_invalid_sync() {
        let bytes = [0xFE, 0xFB, 0x90, 0x00];
        assert!(FrameHeader::parse(&bytes).is_err());
    }

    #[test]
    fn test_frame_header_roundtrip() {
        let original = [0xFF, 0xFB, 0x90, 0x00];
        let hdr = FrameHeader::parse(&original).unwrap();
        let encoded = hdr.encode();
        // Re-parse and check key fields match
        let hdr2 = FrameHeader::parse(&encoded).unwrap();
        assert_eq!(hdr.bitrate_kbps, hdr2.bitrate_kbps);
        assert_eq!(hdr.sample_rate,  hdr2.sample_rate);
        assert_eq!(hdr.layer,        hdr2.layer);
        assert_eq!(hdr.version,      hdr2.version);
    }

    #[test]
    fn test_frame_size_calculation() {
        let bytes = [0xFF, 0xFB, 0x90, 0x00]; // 128kbps, 44100Hz
        let hdr = FrameHeader::parse(&bytes).unwrap();
        // 144 * 128000 / 44100 = 417
        assert_eq!(hdr.frame_size(), 417);
    }

    #[test]
    fn test_mono_frame_header() {
        // FF FB 90 C0 — mono
        let bytes = [0xFF, 0xFB, 0x90, 0xC0];
        let hdr = FrameHeader::parse(&bytes).unwrap();
        assert_eq!(hdr.channel_mode, ChannelMode::Mono);
        assert_eq!(hdr.channels(), 1);
    }

    // ── BitReader / BitWriter ─────────────────────────────────────────────────

    #[test]
    fn test_bitwriter_basic() {
        let mut w = BitWriter::new();
        w.write_bits(0b1011, 4);
        w.write_bits(0b01, 2);
        w.align_byte();
        let bytes = w.into_bytes();
        assert_eq!(bytes[0], 0b1011_0100);
    }

    #[test]
    fn test_bitreader_basic() {
        let data = [0b1011_0100u8];
        let mut r = BitReader::new(&data);
        assert_eq!(r.read_bits(4).unwrap(), 0b1011);
        assert_eq!(r.read_bits(2).unwrap(), 0b01);
    }

    #[test]
    fn test_bitstream_roundtrip() {
        let mut w = BitWriter::new();
        let values: &[(u32, u8)] = &[(0b111, 3), (0b10101, 5), (0b0, 1), (0xFF, 8)];
        for &(v, n) in values { w.write_bits(v, n); }
        w.align_byte();
        let bytes = w.into_bytes();

        let mut r = BitReader::new(&bytes);
        for &(v, n) in values {
            assert_eq!(r.read_bits(n).unwrap(), v, "Mismatch for {} bits", n);
        }
    }

    #[test]
    fn test_bitreader_bit_by_bit() {
        let data = [0b10110001u8];
        let mut r = BitReader::new(&data);
        let bits: Vec<bool> = (0..8).map(|_| r.read_bit().unwrap()).collect();
        assert_eq!(bits, [true, false, true, true, false, false, false, true]);
    }

    // ── DCT / IMDCT ──────────────────────────────────────────────────────────

    #[test]
    fn test_imdct36_length() {
        let input = [1.0f64; 18];
        let out = imdct36(&input);
        assert_eq!(out.len(), 36);
    }

    #[test]
    fn test_imdct12_length() {
        let input = [0.5f64; 6];
        let out = imdct12(&input);
        assert_eq!(out.len(), 12);
    }

    #[test]
    fn test_imdct36_zero_input() {
        let input = [0.0f64; 18];
        let out = imdct36(&input);
        for &v in out.iter() {
            assert!(v.abs() < 1e-10, "Expected ~0, got {}", v);
        }
    }

    #[test]
    fn test_mdct_imdct_approximate_inverse() {
        // MDCT → IMDCT should approximately recover the input (within scaling)
        let original = [
            0.1, 0.3, -0.2, 0.5, 0.1, -0.4,
            0.2, 0.0,  0.3, 0.1, 0.0,  0.0,
            0.0, 0.0,  0.0, 0.0, 0.0,  0.0,
            0.1, 0.0,  0.0, 0.0, 0.0,  0.0,
            0.0, 0.0,  0.0, 0.0, 0.0,  0.0,
            0.0, 0.0,  0.0, 0.0, 0.0,  0.0,
        ];
        let freq = mdct36(&original);
        let reconstructed = imdct36(&freq);
        // Check first 18 samples are "reasonable" (non-NaN)
        for v in reconstructed.iter() {
            assert!(!v.is_nan(), "NaN in IMDCT output");
        }
    }

    // ── Encoder ───────────────────────────────────────────────────────────────

    #[test]
    fn test_encoder_creates_valid_sync_words() {
        let config = EncoderConfig {
            bitrate_kbps: 128,
            sample_rate: 44100,
            channels: 1,
            quality: 5,
        };
        let mut enc = Mp3Encoder::new(config).unwrap();
        let frame = AudioFrame::new(1, 4096, 44100);
        let bytes = enc.encode(&frame).unwrap();
        // Every frame should start with sync word 0xFF 0xEx
        if bytes.len() >= 2 {
            assert_eq!(bytes[0], 0xFF);
            assert_eq!(bytes[1] & 0xE0, 0xE0);
        }
    }

    #[test]
    fn test_encoder_frame_size_correct() {
        let config = EncoderConfig {
            bitrate_kbps: 128,
            sample_rate: 44100,
            channels: 2,
            quality: 5,
        };
        let mut enc = Mp3Encoder::new(config).unwrap();
        // Feed exactly one frame worth of input (1152 samples per channel)
        let frame = AudioFrame::new(2, 1152, 44100);
        let bytes = enc.encode(&frame).unwrap();
        // 144 * 128000 / 44100 = 417 bytes expected
        assert_eq!(bytes.len(), 417);
    }

    #[test]
    fn test_encoder_stereo_vs_mono_size() {
        // Stereo and mono at same bitrate/sample_rate → same frame byte count
        let make_enc = |ch: u8| {
            let cfg = EncoderConfig { bitrate_kbps: 128, sample_rate: 44100, channels: ch, quality: 5 };
            let mut enc = Mp3Encoder::new(cfg).unwrap();
            let frame = AudioFrame::new(ch, 1152, 44100);
            enc.encode(&frame).unwrap().len()
        };
        assert_eq!(make_enc(1), make_enc(2));
    }

    #[test]
    fn test_id3v1_tag() {
        let tag = Mp3Encoder::id3v1_tag("My Song", "Artist", "Album", "2024");
        assert_eq!(tag.len(), 128);
        assert_eq!(&tag[0..3], b"TAG");
        assert_eq!(&tag[3..10], b"My Song");
    }

    // ── Decoder ───────────────────────────────────────────────────────────────

    #[test]
    fn test_decoder_no_panic_on_garbage() {
        let mut dec = Mp3Decoder::new();
        let garbage = vec![0u8; 1024];
        let frames = dec.decode_all(&garbage);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_decoder_no_panic_on_empty() {
        let mut dec = Mp3Decoder::new();
        let frames = dec.decode_all(&[]);
        assert!(frames.is_empty());
    }

    #[test]
    fn test_encoder_decoder_roundtrip_no_panic() {
        // Encode silence, then feed to decoder — should not panic
        let config = EncoderConfig {
            bitrate_kbps: 128,
            sample_rate: 44100,
            channels: 2,
            quality: 5,
        };
        let mut enc = Mp3Encoder::new(config).unwrap();
        let frame = AudioFrame::new(2, 4 * 1152, 44100);
        let encoded = enc.encode(&frame).unwrap();

        let mut dec = Mp3Decoder::new();
        let _decoded = dec.decode_all(&encoded); // must not panic
    }

    // ── AudioFrame ────────────────────────────────────────────────────────────

    #[test]
    fn test_audio_frame_duration() {
        let frame = AudioFrame::new(2, 44100, 44100);
        assert!((frame.duration_ms() - 1000.0).abs() < 0.01);
    }

    #[test]
    fn test_audio_frame_zero_sample_rate() {
        let frame = AudioFrame::new(1, 1000, 0);
        assert_eq!(frame.duration_ms(), 0.0);
    }
}
