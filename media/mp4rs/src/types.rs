#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackKind {
    Video,
    Audio,
    Hint,
    Unknown(u32),
}

impl TrackKind {
    pub fn from_handler(handler: u32) -> Self {
        match handler {
            0x76696465 => TrackKind::Video, // 'vide'
            0x736F756E => TrackKind::Audio, // 'soun'
            0x68696E74 => TrackKind::Hint,  // 'hint'
            _ => TrackKind::Unknown(handler),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            TrackKind::Video => "Video",
            TrackKind::Audio => "Audio",
            TrackKind::Hint => "Hint",
            TrackKind::Unknown(_) => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub track_id: u32,
    pub kind: TrackKind,
    pub sample_count: u32,
    pub duration: u64,
    pub timescale: u32,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub channel_count: Option<u32>,
    pub sample_rate: Option<u32>,
    pub language: [u8; 4],
}

#[derive(Debug, Clone)]
pub struct AvcNalInfo {
    pub sps_list: Vec<Vec<u8>>,
    pub pps_list: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct Mp4Info {
    pub major_brand: [u8; 4],
    pub minor_version: u32,
    pub compatible_brands: Vec<[u8; 4]>,
    pub timescale: u32,
    pub duration: u64,
    pub tracks: Vec<TrackInfo>,
}

impl std::fmt::Display for Mp4Info {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let brand = std::str::from_utf8(&self.major_brand).unwrap_or("????");
        writeln!(f, "=== MP4 File Info ===")?;
        writeln!(f, "Brand: {brand} v{}", self.minor_version)?;
        let secs = self.duration as f64 / self.timescale as f64;
        writeln!(f, "Duration: {:.3}s ({} ticks @ {} Hz)", secs, self.duration, self.timescale)?;
        writeln!(f, "Tracks: {}", self.tracks.len())?;
        for (i, tr) in self.tracks.iter().enumerate() {
            let lang = std::str::from_utf8(&tr.language[..3]).unwrap_or("???");
            let dur_secs = tr.duration as f64 / tr.timescale as f64;
            write!(f, "  Track {} (id={}): {}", i, tr.track_id, tr.kind.name())?;
            write!(f, ", {} samples", tr.sample_count)?;
            write!(f, ", {:.3}s", dur_secs)?;
            if let (Some(w), Some(h)) = (tr.width, tr.height) {
                write!(f, ", {}x{}", w, h)?;
            }
            if let (Some(sc), Some(sr)) = (tr.channel_count, tr.sample_rate) {
                write!(f, ", {}ch @ {} Hz", sc, sr)?;
            }
            write!(f, ", lang={}", lang)?;
            writeln!(f)?;
        }
        Ok(())
    }
}
