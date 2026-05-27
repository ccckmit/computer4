mod demuxer;
mod error;
pub mod types;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use crate::demuxer::Demuxer;

pub use error::{Mp4Error, Result};

/// Open an MP4 file and read its structure.
pub fn open(path: impl AsRef<Path>) -> Result<types::Mp4Info> {
    let d = Demuxer::open(path)?;
    Ok(d.info)
}

/// Read a specific frame (sample) from a video track as raw H.264 AVCC data.
/// Returns (raw_data, is_keyframe).
pub fn read_frame(
    path: impl AsRef<Path>,
    track_idx: usize,
    sample_idx: u32,
) -> Result<(Vec<u8>, bool)> {
    let d = Demuxer::open(&path)?;
    if track_idx >= d.tracks.len() {
        return Err(Mp4Error::TrackNotFound(track_idx as u32));
    }
    let track = &d.tracks[track_idx];
    let entries = &track.entries;
    if (sample_idx as usize) >= entries.len() {
        return Err(Mp4Error::SampleOutOfRange(sample_idx));
    }
    let entry = &entries[sample_idx as usize];

    let mut file = File::open(path.as_ref())?;
    let mut buf = vec![0u8; entry.size as usize];
    file.seek(SeekFrom::Start(entry.offset))?;
    file.read_exact(&mut buf)?;

    Ok((buf, false))
}

/// Read a frame and convert AVCC length-prefixed NALUs to Annex-B start codes.
pub fn read_frame_annex_b(
    path: impl AsRef<Path>,
    track_idx: usize,
    sample_idx: u32,
) -> Result<(Vec<u8>, bool)> {
    let (mut raw, keyframe) = read_frame(path, track_idx, sample_idx)?;
    avcc_to_annex_b(&mut raw);
    Ok((raw, keyframe))
}

/// Convert AVCC (4-byte length prefix) to Annex-B (0x00000001 start code) in place.
/// The converted data may shrink (if a length field happened to be 0x00000001 or similar)
/// but we assume the buffer is large enough to hold the start code.
pub fn avcc_to_annex_b(data: &mut [u8]) {
    if data.len() < 4 {
        return;
    }
    let mut pos = 0usize;
    while pos + 4 <= data.len() {
        let nalu_len = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        if nalu_len == 0 {
            break; // sanity
        }
        data[pos] = 0;
        data[pos + 1] = 0;
        data[pos + 2] = 0;
        data[pos + 3] = 1;
        pos += 4 + nalu_len;
    }
}

/// Concatenate SPS, PPS, and a frame into a single Annex-B byte stream.
pub fn build_annex_b_stream(
    avc: &types::AvcNalInfo,
    frame_data: &[u8],
) -> Vec<u8> {
    let start_code = [0u8, 0, 0, 1];
    let mut out = Vec::new();

    for sps in &avc.sps_list {
        out.extend_from_slice(&start_code);
        out.extend_from_slice(sps);
    }
    for pps in &avc.pps_list {
        out.extend_from_slice(&start_code);
        out.extend_from_slice(pps);
    }
    // Frame data (already Annex-B)
    out.extend_from_slice(frame_data);

    out
}

/// Get AVC config (SPS/PPS) for a track.
pub fn avc_config(path: impl AsRef<Path>, track_idx: usize) -> Result<Option<types::AvcNalInfo>> {
    let d = Demuxer::open(path)?;
    if track_idx >= d.tracks.len() {
        return Err(Mp4Error::TrackNotFound(track_idx as u32));
    }
    Ok(d.tracks[track_idx].avc.clone())
}

/// Get sample count for a track.
pub fn track_sample_count(path: impl AsRef<Path>, track_idx: usize) -> Result<u32> {
    let d = Demuxer::open(path)?;
    if track_idx >= d.tracks.len() {
        return Err(Mp4Error::TrackNotFound(track_idx as u32));
    }
    Ok(d.tracks[track_idx].info.sample_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_avcc_to_annex_b() {
        // Simulate a simple AVCC frame: one NALU of 10 bytes
        let len: u32 = 10;
        let mut data = Vec::new();
        data.extend_from_slice(&len.to_be_bytes());
        data.extend_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        avcc_to_annex_b(&mut data);
        assert_eq!(data[0], 0);
        assert_eq!(data[1], 0);
        assert_eq!(data[2], 0);
        assert_eq!(data[3], 1);
        assert_eq!(data[4], 1);
    }
}
