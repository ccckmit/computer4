use std::io::Cursor;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use crate::types::Mp3Info;

pub fn decode_mp3(path: &Path) -> Result<(Mp3Info, Vec<i16>, i32, i32), String> {
    let mut file = File::open(path).map_err(|e| format!("cannot open {}: {}", path.display(), e))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("read error: {}", e))?;

    let mut decoder = minimp3::Decoder::new(Cursor::new(&data));
    let mut all_pcm = Vec::new();
    let mut sample_rate = 0i32;
    let mut channels = 0i32;
    let mut bitrate = 0i32;
    let mut frame_bytes = 0usize;

    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                if sample_rate == 0 {
                    sample_rate = frame.sample_rate;
                    channels = frame.channels as i32;
                    bitrate = frame.bitrate;
                    frame_bytes = frame.data.len();
                }
                for &sample in &frame.data {
                    all_pcm.push(sample);
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(err) => return Err(format!("decode error: {:?}", err)),
        }
    }

    if all_pcm.is_empty() {
        return Err("no audio frames decoded".into());
    }

    let info = Mp3Info { bitrate_kbps: bitrate, sample_rate, channels, frame_bytes };
    Ok((info, all_pcm, sample_rate, channels))
}
