use std::fs::File;
use std::io::Write;
use std::path::Path;

use mp3lame_encoder::{Builder, Bitrate, Quality, FlushNoGap, MonoPcm, InterleavedPcm};

pub fn encode_mp3(path: &Path, samples: &[i16], sample_rate: u32, channels: u16, bitrate: u32) -> Result<(), String> {
    let bitrate_enum = match bitrate {
        8 => Bitrate::Kbps8, 16 => Bitrate::Kbps16, 24 => Bitrate::Kbps24,
        32 => Bitrate::Kbps32, 40 => Bitrate::Kbps40, 48 => Bitrate::Kbps48,
        64 => Bitrate::Kbps64, 80 => Bitrate::Kbps80, 96 => Bitrate::Kbps96,
        112 => Bitrate::Kbps112, 128 => Bitrate::Kbps128, 160 => Bitrate::Kbps160,
        192 => Bitrate::Kbps192, 224 => Bitrate::Kbps224, 256 => Bitrate::Kbps256,
        320 => Bitrate::Kbps320,
        _ => return Err(format!("unsupported bitrate: {}", bitrate)),
    };

    let mut builder = Builder::new().ok_or("failed to create LAME encoder")?;
    builder.set_num_channels(channels as u8).map_err(|e| format!("set channels: {e}"))?;
    builder.set_sample_rate(sample_rate).map_err(|e| format!("set sample rate: {e}"))?;
    builder.set_brate(bitrate_enum).map_err(|e| format!("set bitrate: {e}"))?;
    builder.set_quality(Quality::Good).map_err(|e| format!("set quality: {e}"))?;

    let mut encoder = builder.build().map_err(|e| format!("build encoder: {e}"))?;

    let out_buf_size = mp3lame_encoder::max_required_buffer_size(samples.len());
    let mut mp3_buf = Vec::with_capacity(out_buf_size);

    if channels == 1 {
        encoder.encode_to_vec(MonoPcm(samples), &mut mp3_buf)
            .map_err(|e| format!("encode: {e}"))?;
    } else {
        encoder.encode_to_vec(InterleavedPcm(samples), &mut mp3_buf)
            .map_err(|e| format!("encode: {e}"))?;
    }

    encoder.flush_to_vec::<FlushNoGap>(&mut mp3_buf)
        .map_err(|e| format!("flush: {e}"))?;

    let mut file = File::create(path).map_err(|e| format!("cannot create {}: {}", path.display(), e))?;
    file.write_all(&mp3_buf).map_err(|e| format!("write error: {e}"))?;

    Ok(())
}
