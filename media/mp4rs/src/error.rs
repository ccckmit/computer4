use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Mp4Error {
    Io(io::Error),
    InvalidData(&'static str),
    Unsupported(&'static str),
    NoMoovBox,
    NoMdatBox,
    NoVideoTrack,
    TrackNotFound(u32),
    SampleOutOfRange(u32),
}

impl fmt::Display for Mp4Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mp4Error::Io(e) => write!(f, "I/O error: {e}"),
            Mp4Error::InvalidData(msg) => write!(f, "Invalid MP4 data: {msg}"),
            Mp4Error::Unsupported(msg) => write!(f, "Unsupported: {msg}"),
            Mp4Error::NoMoovBox => write!(f, "No moov box found"),
            Mp4Error::NoMdatBox => write!(f, "No mdat box found"),
            Mp4Error::NoVideoTrack => write!(f, "No video track found"),
            Mp4Error::TrackNotFound(id) => write!(f, "Track {id} not found"),
            Mp4Error::SampleOutOfRange(n) => write!(f, "Sample {n} out of range"),
        }
    }
}

impl std::error::Error for Mp4Error {}

impl From<io::Error> for Mp4Error {
    fn from(e: io::Error) -> Self { Mp4Error::Io(e) }
}

pub type Result<T> = std::result::Result<T, Mp4Error>;
