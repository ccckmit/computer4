use std::fmt;

#[derive(Debug, Clone)]
pub enum CodecError {
    InvalidSyncWord,
    UnsupportedVersion(u8),
    UnsupportedLayer(u8),
    UnsupportedBitrate,
    UnsupportedSampleRate,
    InvalidFrameHeader,
    BufferTooShort { needed: usize, available: usize },
    HuffmanError(String),
    EncoderError(String),
    IoError(String),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodecError::InvalidSyncWord => write!(f, "Invalid MPEG sync word"),
            CodecError::UnsupportedVersion(v) => write!(f, "Unsupported MPEG version: {}", v),
            CodecError::UnsupportedLayer(l) => write!(f, "Unsupported MPEG layer: {}", l),
            CodecError::UnsupportedBitrate => write!(f, "Unsupported or free bitrate"),
            CodecError::UnsupportedSampleRate => write!(f, "Unsupported sample rate"),
            CodecError::InvalidFrameHeader => write!(f, "Invalid frame header"),
            CodecError::BufferTooShort { needed, available } => {
                write!(f, "Buffer too short: needed {}, available {}", needed, available)
            }
            CodecError::HuffmanError(msg) => write!(f, "Huffman coding error: {}", msg),
            CodecError::EncoderError(msg) => write!(f, "Encoder error: {}", msg),
            CodecError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CodecError {}
