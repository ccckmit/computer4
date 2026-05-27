use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write, BufReader, BufWriter};
use std::path::Path;

use crate::types::WavHeader;

pub fn read_wav_header(path: &Path) -> Result<(WavHeader, Vec<i16>), String> {
    let file = File::open(path).map_err(|e| format!("cannot open {}: {}", path.display(), e))?;
    let mut reader = BufReader::new(file);

    let mut riff = [0u8; 4];
    reader.read_exact(&mut riff).map_err(|e| format!("read error: {}", e))?;
    if &riff != b"RIFF" {
        return Err("not a RIFF file".into());
    }

    let mut chunk_size = [0u8; 4];
    reader.read_exact(&mut chunk_size).map_err(|e| format!("read error: {}", e))?;

    let mut wave = [0u8; 4];
    reader.read_exact(&mut wave).map_err(|e| format!("read error: {}", e))?;
    if &wave != b"WAVE" {
        return Err("not a WAVE file".into());
    }

    let mut fmt_id = [0u8; 4];
    reader.read_exact(&mut fmt_id).map_err(|e| format!("read error: {}", e))?;

    let mut fmt_len = [0u8; 4];
    reader.read_exact(&mut fmt_len).map_err(|e| format!("read error: {}", e))?;
    let fmt_len = u32::from_le_bytes(fmt_len);

    let mut fmt_data = vec![0u8; fmt_len as usize];
    reader.read_exact(&mut fmt_data).map_err(|e| format!("read error: {}", e))?;

    let format_tag = u16::from_le_bytes([fmt_data[0], fmt_data[1]]);
    if format_tag != 1 {
        return Err(format!("unsupported format tag: {} (only PCM supported)", format_tag));
    }

    let channels = u16::from_le_bytes([fmt_data[2], fmt_data[3]]);
    let sample_rate = u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]);
    let bits_per_sample = u16::from_le_bytes([fmt_data[14], fmt_data[15]]);

    if bits_per_sample != 16 {
        return Err(format!("only 16-bit WAV supported, got {} bits", bits_per_sample));
    }

    loop {
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() {
            return Err("no data chunk found".into());
        }

        let mut chunk_size = [0u8; 4];
        reader.read_exact(&mut chunk_size).map_err(|e| format!("read error: {}", e))?;
        let chunk_size = u32::from_le_bytes(chunk_size);

        if &chunk_id == b"data" {
            let num_samples = chunk_size as usize / (channels as usize * (bits_per_sample as usize / 8));
            let mut raw = vec![0u8; chunk_size as usize];
            reader.read_exact(&mut raw).map_err(|e| format!("read error: {}", e))?;

            let mut samples = Vec::with_capacity(num_samples * channels as usize);
            for chunk in raw.chunks(2) {
                if chunk.len() == 2 {
                    samples.push(i16::from_le_bytes([chunk[0], chunk[1]]));
                }
            }

            let header = WavHeader {
                channels,
                sample_rate,
                bits_per_sample,
                data_size: chunk_size,
            };

            return Ok((header, samples));
        } else {
            reader.seek(SeekFrom::Current(chunk_size as i64)).map_err(|e| format!("seek error: {}", e))?;
        }
    }
}

pub fn write_wav(path: &Path, header: &WavHeader, samples: &[i16]) -> Result<(), String> {
    let file = File::create(path).map_err(|e| format!("cannot create {}: {}", path.display(), e))?;
    let mut writer = BufWriter::new(file);

    let data_size = samples.len() as u32 * 2;
    let chunk_size = 36 + data_size;

    writer.write_all(b"RIFF").map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&chunk_size.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(b"WAVE").map_err(|e| format!("write error: {}", e))?;

    writer.write_all(b"fmt ").map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&16u32.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&1u16.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&header.channels.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&header.sample_rate.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&(header.sample_rate * header.channels as u32 * 2).to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&(header.channels * 2).to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&header.bits_per_sample.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;

    writer.write_all(b"data").map_err(|e| format!("write error: {}", e))?;
    writer.write_all(&data_size.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;

    for &sample in samples {
        writer.write_all(&sample.to_le_bytes()).map_err(|e| format!("write error: {}", e))?;
    }

    writer.flush().map_err(|e| format!("flush error: {}", e))?;
    Ok(())
}
