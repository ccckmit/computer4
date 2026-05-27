use std::fmt;

#[derive(Debug, Clone)]
pub struct WavHeader {
    pub channels: u16,
    pub sample_rate: u32,
    pub bits_per_sample: u16,
    pub data_size: u32,
}

impl fmt::Display for WavHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "channels: {}, sample_rate: {} Hz, bits: {}, data_size: {} bytes",
            self.channels, self.sample_rate, self.bits_per_sample, self.data_size
        )
    }
}

#[derive(Debug, Clone)]
pub struct Mp3Info {
    pub bitrate_kbps: i32,
    pub sample_rate: i32,
    pub channels: i32,
    pub frame_bytes: usize,
}

impl fmt::Display for Mp3Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "bitrate: {} kbps, sample_rate: {} Hz, channels: {}, frame_bytes: {}",
            self.bitrate_kbps, self.sample_rate, self.channels, self.frame_bytes
        )
    }
}
