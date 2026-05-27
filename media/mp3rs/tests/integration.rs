#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_read_mp3_info() {
        let path = Path::new("data/test.mp3");
        if !path.exists() {
            eprintln!("Skipping: data/test.mp3 not found");
            return;
        }
        let (info, pcm, sr, ch) = mp3rs::decode_mp3(path).expect("decode mp3 failed");
        assert!(info.bitrate_kbps > 0, "bitrate should be > 0");
        assert!(sr > 0, "sample rate should be > 0");
        assert!(ch > 0, "channels should be > 0");
        assert!(!pcm.is_empty(), "pcm data should not be empty");
    }

    #[test]
    fn test_roundtrip() {
        let mp3_path = Path::new("data/test.mp3");
        if !mp3_path.exists() {
            eprintln!("Skipping roundtrip: data/test.mp3 not found");
            return;
        }

        let (_info, pcm, sr, ch) = mp3rs::decode_mp3(mp3_path).expect("decode failed");

        let wav_hdr = mp3rs::WavHeader {
            channels: ch as u16,
            sample_rate: sr as u32,
            bits_per_sample: 16,
            data_size: pcm.len() as u32 * 2,
        };

        let tmp_wav = std::env::temp_dir().join("test_roundtrip.wav");
        mp3rs::write_wav(&tmp_wav, &wav_hdr, &pcm).expect("write wav failed");

        let (hdr2, pcm2) = mp3rs::read_wav_header(&tmp_wav).expect("read wav back failed");
        assert_eq!(hdr2.channels, ch as u16);
        assert_eq!(hdr2.sample_rate, sr as u32);
        assert_eq!(hdr2.bits_per_sample, 16);
        assert!(!pcm2.is_empty());

        std::fs::remove_file(&tmp_wav).ok();
    }
}
