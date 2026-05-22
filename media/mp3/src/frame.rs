use crate::error::CodecError;

/// MPEG version identifier
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MpegVersion {
    Mpeg1,
    Mpeg2,
    Mpeg25,
}

/// MPEG audio layer
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Layer {
    Layer1,
    Layer2,
    Layer3,
}

/// Channel mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMode {
    Stereo,
    JointStereo,
    DualChannel,
    Mono,
}

/// Parsed MPEG audio frame header (32 bits)
#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub version: MpegVersion,
    pub layer: Layer,
    pub bitrate_kbps: u32,
    pub sample_rate: u32,
    pub padding: bool,
    pub channel_mode: ChannelMode,
    pub mode_extension: u8,
    pub copyright: bool,
    pub original: bool,
}

// Bitrate table [version_idx][layer_idx][bitrate_idx]
// version: 0=MPEG1, 1=MPEG2/2.5
// layer:   0=L1,    1=L2,    2=L3
const BITRATE_TABLE: [[[u32; 16]; 3]; 2] = [
    // MPEG 1
    [
        [0,32,64,96,128,160,192,224,256,288,320,352,384,416,448,0], // L1
        [0,32,48,56, 64, 80, 96,112,128,160,192,224,256,320,384,0], // L2
        [0,32,40,48, 56, 64, 80, 96,112,128,160,192,224,256,320,0], // L3
    ],
    // MPEG 2 / 2.5
    [
        [0,32,48,56,64,80,96,112,128,144,160,176,192,224,256,0],    // L1
        [0, 8,16,24,32,40,48, 56, 64, 80, 96,112,128,144,160,0],    // L2
        [0, 8,16,24,32,40,48, 56, 64, 80, 96,112,128,144,160,0],    // L3
    ],
];

const SAMPLE_RATE_TABLE: [[u32; 4]; 3] = [
    [44100, 48000, 32000, 0], // MPEG 1
    [22050, 24000, 16000, 0], // MPEG 2
    [11025, 12000,  8000, 0], // MPEG 2.5
];

impl FrameHeader {
    /// Parse a 4-byte MPEG frame header
    pub fn parse(bytes: &[u8]) -> Result<Self, CodecError> {
        if bytes.len() < 4 {
            return Err(CodecError::BufferTooShort { needed: 4, available: bytes.len() });
        }

        // Sync word: first 11 bits must all be 1
        if bytes[0] != 0xFF || (bytes[1] & 0xE0) != 0xE0 {
            return Err(CodecError::InvalidSyncWord);
        }

        // Version (bits 20-19)
        let version_bits = (bytes[1] >> 3) & 0x03;
        let version = match version_bits {
            0b00 => MpegVersion::Mpeg25,
            0b10 => MpegVersion::Mpeg2,
            0b11 => MpegVersion::Mpeg1,
            _ => return Err(CodecError::UnsupportedVersion(version_bits)),
        };

        // Layer (bits 18-17)
        let layer_bits = (bytes[1] >> 1) & 0x03;
        let layer = match layer_bits {
            0b01 => Layer::Layer3,
            0b10 => Layer::Layer2,
            0b11 => Layer::Layer1,
            _ => return Err(CodecError::UnsupportedLayer(layer_bits)),
        };

        // Bitrate index (bits 15-12)
        let bitrate_idx = ((bytes[2] >> 4) & 0x0F) as usize;
        let ver_idx = if version == MpegVersion::Mpeg1 { 0 } else { 1 };
        let lay_idx = match layer { Layer::Layer1 => 0, Layer::Layer2 => 1, Layer::Layer3 => 2 };
        let bitrate_kbps = BITRATE_TABLE[ver_idx][lay_idx][bitrate_idx];
        if bitrate_kbps == 0 {
            return Err(CodecError::UnsupportedBitrate);
        }

        // Sample rate index (bits 11-10)
        let sr_idx = ((bytes[2] >> 2) & 0x03) as usize;
        let ver_sr_idx = match version {
            MpegVersion::Mpeg1  => 0,
            MpegVersion::Mpeg2  => 1,
            MpegVersion::Mpeg25 => 2,
        };
        let sample_rate = SAMPLE_RATE_TABLE[ver_sr_idx][sr_idx];
        if sample_rate == 0 {
            return Err(CodecError::UnsupportedSampleRate);
        }

        let padding       = (bytes[2] >> 1) & 0x01 == 1;
        let channel_bits  = (bytes[3] >> 6) & 0x03;
        let channel_mode  = match channel_bits {
            0b00 => ChannelMode::Stereo,
            0b01 => ChannelMode::JointStereo,
            0b10 => ChannelMode::DualChannel,
            _    => ChannelMode::Mono,
        };
        let mode_extension = (bytes[3] >> 4) & 0x03;
        let copyright      = (bytes[3] >> 3) & 0x01 == 1;
        let original       = (bytes[3] >> 2) & 0x01 == 1;

        Ok(FrameHeader {
            version, layer, bitrate_kbps, sample_rate,
            padding, channel_mode, mode_extension, copyright, original,
        })
    }

    /// Encode this header to 4 bytes
    pub fn encode(&self) -> [u8; 4] {
        let version_bits: u8 = match self.version {
            MpegVersion::Mpeg25 => 0b00,
            MpegVersion::Mpeg2  => 0b10,
            MpegVersion::Mpeg1  => 0b11,
        };
        let layer_bits: u8 = match self.layer {
            Layer::Layer1 => 0b11,
            Layer::Layer2 => 0b10,
            Layer::Layer3 => 0b01,
        };

        let ver_idx = if self.version == MpegVersion::Mpeg1 { 0 } else { 1 };
        let lay_idx = match self.layer { Layer::Layer1 => 0, Layer::Layer2 => 1, Layer::Layer3 => 2 };
        let bitrate_idx = BITRATE_TABLE[ver_idx][lay_idx]
            .iter()
            .position(|&b| b == self.bitrate_kbps)
            .unwrap_or(9) as u8;

        let ver_sr_idx = match self.version {
            MpegVersion::Mpeg1  => 0,
            MpegVersion::Mpeg2  => 1,
            MpegVersion::Mpeg25 => 2,
        };
        let sr_idx = SAMPLE_RATE_TABLE[ver_sr_idx]
            .iter()
            .position(|&r| r == self.sample_rate)
            .unwrap_or(0) as u8;

        let channel_bits: u8 = match self.channel_mode {
            ChannelMode::Stereo      => 0b00,
            ChannelMode::JointStereo => 0b01,
            ChannelMode::DualChannel => 0b10,
            ChannelMode::Mono        => 0b11,
        };

        [
            0xFF,
            0xE0 | (version_bits << 3) | (layer_bits << 1) | 1, // protection_bit=1
            (bitrate_idx << 4) | (sr_idx << 2) | (self.padding as u8) << 1,
            (channel_bits << 6) | (self.mode_extension << 4)
                | ((self.copyright as u8) << 3)
                | ((self.original as u8) << 2),
        ]
    }

    /// Frame size in bytes (including header)
    pub fn frame_size(&self) -> usize {
        match self.layer {
            Layer::Layer1 => {
                let slots = (12 * self.bitrate_kbps * 1000 / self.sample_rate + self.padding as u32) * 4;
                slots as usize
            }
            Layer::Layer2 | Layer::Layer3 => {
                let slots = 144 * self.bitrate_kbps * 1000 / self.sample_rate + self.padding as u32;
                slots as usize
            }
        }
    }

    pub fn channels(&self) -> u8 {
        if self.channel_mode == ChannelMode::Mono { 1 } else { 2 }
    }

    pub fn samples_per_frame(&self) -> usize {
        match self.layer {
            Layer::Layer1 => 384,
            Layer::Layer2 => 1152,
            Layer::Layer3 => 1152,
        }
    }
}
