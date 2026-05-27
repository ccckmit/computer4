#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_avcc_to_annex_b_conversion() {
        let len: u32 = 8;
        let mut data = Vec::new();
        data.extend_from_slice(&len.to_be_bytes());
        data.extend_from_slice(&[0x65, 0x88, 0x84, 0x01, 0x02, 0x03, 0x04, 0x05]);
        mp4rs::avcc_to_annex_b(&mut data);
        assert_eq!(&data[..4], &[0, 0, 0, 1]);
        assert_eq!(&data[4..], &[0x65, 0x88, 0x84, 0x01, 0x02, 0x03, 0x04, 0x05]);
    }

    #[test]
    fn test_avcc_to_annex_b_multi_nalu() {
        // Two NALUs: sizes 6 and 4
        let n1: u32 = 6;
        let n2: u32 = 4;
        let mut data = Vec::new();
        data.extend_from_slice(&n1.to_be_bytes());
        data.extend_from_slice(&[0x41, 0x9a, 0x22, 0x10, 0x01, 0x02]);
        data.extend_from_slice(&n2.to_be_bytes());
        data.extend_from_slice(&[0x42, 0x01, 0x02, 0x03]);
        mp4rs::avcc_to_annex_b(&mut data);
        assert_eq!(&data[0..4], &[0, 0, 0, 1]);
        assert_eq!(data[4], 0x41);
        // second start code
        let second_start = 4 + 6;
        assert_eq!(&data[second_start..second_start + 4], &[0, 0, 0, 1]);
        assert_eq!(data[second_start + 4], 0x42);
    }

    #[test]
    fn test_build_annex_b_stream() {
        let avc = mp4rs::types::AvcNalInfo {
            sps_list: vec![vec![0x67, 0x64, 0x00, 0x1e]],
            pps_list: vec![vec![0x68, 0xeb, 0xe3, 0xcb, 0x22, 0xc0]],
        };
        let frame = vec![0, 0, 0, 1, 0x65, 0x88];
        let stream = mp4rs::build_annex_b_stream(&avc, &frame);
        // Should have: start_code + SPS + start_code + PPS + frame
        assert!(stream.len() > 0);
        assert_eq!(&stream[0..4], &[0, 0, 0, 1]);
        assert_eq!(&stream[4..8], &[0x67, 0x64, 0x00, 0x1e]);
    }

    #[test]
    fn test_open_nonexistent_file() {
        let result = mp4rs::open(Path::new("/nonexistent/test.mp4"));
        assert!(result.is_err());
    }

    #[test]
    fn test_track_sample_count_nonexistent() {
        let path = Path::new("/nonexistent/test.mp4");
        let result = mp4rs::track_sample_count(path, 0);
        assert!(result.is_err());
    }
}
