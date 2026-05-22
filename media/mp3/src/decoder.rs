//! MPEG Audio Layer III decoder.
//!
//! Pipeline:
//!   raw bytes → frame sync → header parse → side info → scale factors
//!               → Huffman decode → dequantize → IMDCT → polyphase filterbank → PCM

use crate::AudioFrame;
use crate::bitstream::BitReader;
use crate::dct::{imdct36, imdct12, BlockType};
use crate::error::CodecError;
use crate::frame::{FrameHeader, Layer};
use crate::huffman::{decode_pair, get_table};
use crate::tables::{pow43, dequant_scale, scalefac_scale, SFB_LONG_44100};

/// Granule side information (per channel, per granule)
#[derive(Debug, Default, Clone, Copy)]
struct GranuleInfo {
    part2_3_length: u16,
    big_values: u16,
    global_gain: u8,
    scalefac_compress: u8,
    window_switching: bool,
    block_type: u8,
    mixed_block_flag: bool,
    table_select: [u8; 3],
    subblock_gain: [u8; 3],
    region0_count: u8,
    region1_count: u8,
    preflag: bool,
    scalefac_scale: bool,
    count1table_select: u8,
}

/// Main MP3 decoder
pub struct Mp3Decoder {
    /// Overlap-add buffer for IMDCT (per channel, 18 subbands × 32 samples)
    overlap: Vec<[[f64; 18]; 32]>,
    /// Current frame header (if synced)
    header: Option<FrameHeader>,
    /// Internal buffer for byte accumulation
    buf: Vec<u8>,
}

impl Mp3Decoder {
    pub fn new() -> Self {
        Mp3Decoder {
            overlap: vec![[[0.0f64; 18]; 32]; 2],
            header: None,
            buf: Vec::new(),
        }
    }

    /// Feed raw bytes and return decoded PCM frames as they become available.
    pub fn feed(&mut self, data: &[u8]) -> Vec<AudioFrame> {
        self.buf.extend_from_slice(data);
        let mut frames = Vec::new();

        loop {
            // Find sync word
            let Some(sync_pos) = self.find_sync() else { break };
            if self.buf.len() < sync_pos + 4 { break; }

            let hdr_bytes = &self.buf[sync_pos..sync_pos + 4];
            let header = match FrameHeader::parse(hdr_bytes) {
                Ok(h) => h,
                Err(_) => {
                    self.buf.drain(..sync_pos + 1);
                    continue;
                }
            };

            let frame_size = header.frame_size();
            if self.buf.len() < sync_pos + frame_size { break; }

            let frame_bytes: Vec<u8> = self.buf[sync_pos..sync_pos + frame_size].to_vec();
            self.buf.drain(..sync_pos + frame_size);

            match self.decode_frame(&frame_bytes, &header) {
                Ok(audio) => frames.push(audio),
                Err(_e) => { /* skip corrupt frame */ }
            }
        }
        frames
    }

    /// Find the next MPEG sync word (0xFFE0) in the buffer.
    fn find_sync(&self) -> Option<usize> {
        for i in 0..self.buf.len().saturating_sub(1) {
            if self.buf[i] == 0xFF && (self.buf[i + 1] & 0xE0) == 0xE0 {
                return Some(i);
            }
        }
        None
    }

    /// Decode one complete MPEG frame.
    pub fn decode_frame(&mut self, data: &[u8], header: &FrameHeader) -> Result<AudioFrame, CodecError> {
        match header.layer {
            Layer::Layer3 => self.decode_layer3(data, header),
            _ => Err(CodecError::UnsupportedLayer(0)),
        }
    }

    /// Full Layer III frame decode pipeline.
    fn decode_layer3(&mut self, data: &[u8], header: &FrameHeader) -> Result<AudioFrame, CodecError> {
        let channels = header.channels() as usize;
        let stereo = channels == 2;

        // Header is at offset 0, side info starts at byte 4
        // (CRC is skipped — protection_bit handling omitted for brevity)
        let side_info_offset = 4usize;
        let side_info_len = if stereo { 32 } else { 17 };

        if data.len() < side_info_offset + side_info_len {
            return Err(CodecError::BufferTooShort {
                needed: side_info_offset + side_info_len,
                available: data.len(),
            });
        }

        let mut reader = BitReader::new_at(data, side_info_offset);

        // --- Side information ---
        let _main_data_begin = reader.read_bits(9).unwrap_or(0);
        let _private_bits = reader.read_bits(if stereo { 3 } else { 5 }).unwrap_or(0);

        // Scalefactor selection info (per channel, 4 bands)
        let mut scfsi = [[false; 4]; 2];
        for ch in 0..channels {
            for band in 0..4 {
                scfsi[ch][band] = reader.read_bit().unwrap_or(false);
            }
        }

        // Granule info (2 granules × channels)
        let mut gran_info = [[GranuleInfo::default(); 2]; 2]; // [ch][gran]
        for gran in 0..2 {
            for ch in 0..channels {
                let g = &mut gran_info[ch][gran];
                g.part2_3_length   = reader.read_bits(12).unwrap_or(0) as u16;
                g.big_values        = reader.read_bits(9).unwrap_or(0) as u16;
                g.global_gain       = reader.read_bits(8).unwrap_or(0) as u8;
                g.scalefac_compress = reader.read_bits(4).unwrap_or(0) as u8;
                g.window_switching  = reader.read_bit().unwrap_or(false);
                if g.window_switching {
                    g.block_type       = reader.read_bits(2).unwrap_or(0) as u8;
                    g.mixed_block_flag = reader.read_bit().unwrap_or(false);
                    g.table_select[0]  = reader.read_bits(5).unwrap_or(0) as u8;
                    g.table_select[1]  = reader.read_bits(5).unwrap_or(0) as u8;
                    for i in 0..3 {
                        g.subblock_gain[i] = reader.read_bits(3).unwrap_or(0) as u8;
                    }
                    g.region0_count = if g.block_type == 2 { 8 } else { 7 };
                    g.region1_count = 36;
                } else {
                    g.table_select[0]  = reader.read_bits(5).unwrap_or(0) as u8;
                    g.table_select[1]  = reader.read_bits(5).unwrap_or(0) as u8;
                    g.table_select[2]  = reader.read_bits(5).unwrap_or(0) as u8;
                    g.region0_count    = reader.read_bits(4).unwrap_or(0) as u8;
                    g.region1_count    = reader.read_bits(3).unwrap_or(0) as u8;
                }
                g.preflag            = reader.read_bit().unwrap_or(false);
                g.scalefac_scale     = reader.read_bit().unwrap_or(false);
                g.count1table_select = reader.read_bit().unwrap_or(false) as u8;
            }
        }

        // --- Main data starts after side info ---
        let main_data_offset = side_info_offset + side_info_len;
        let mut main_reader = BitReader::new_at(data, main_data_offset);

        let samples_per_frame = header.samples_per_frame();
        let mut audio = AudioFrame::new(channels as u8, samples_per_frame, header.sample_rate);

        // Process 2 granules
        for gran in 0..2 {
            for ch in 0..channels {
                let g = &gran_info[ch][gran];

                // --- Scale factors ---
                let sfc = g.scalefac_compress as usize;
                let slen1 = [0,0,0,0,3,1,1,1,2,2,2,3,3,3,4,4][sfc];
                let slen2 = [0,1,2,3,0,1,2,3,1,2,3,1,2,3,2,3][sfc];

                let mut scalefacs = [0i32; 22];
                for sfb in 0..11 { scalefacs[sfb] = main_reader.read_bits(slen1).unwrap_or(0) as i32; }
                for sfb in 11..22 { scalefacs[sfb] = main_reader.read_bits(slen2).unwrap_or(0) as i32; }

                // --- Huffman decode: big_values region ---
                let mut is = [0i32; 576];
                let big_values = g.big_values as usize;

                // Region boundaries (simplified: use region0/1 counts)
                let r0_sfb = (g.region0_count as usize + 1).min(22);
                let r1_sfb = (g.region0_count as usize + g.region1_count as usize + 2).min(22);
                let r0_end = SFB_LONG_44100.get(r0_sfb).copied().unwrap_or(576).min(big_values * 2);
                let r1_end = SFB_LONG_44100.get(r1_sfb).copied().unwrap_or(576).min(big_values * 2);

for i in (0..big_values * 2).step_by(2) {
if i + 1 >= 576 { break; }
let tbl_id = if i < r0_end { g.table_select[0] }
else if i < r1_end { g.table_select[1] }
else { g.table_select[2] };

if let Some(tbl) = get_table(tbl_id) {
if let Ok((x, y)) = decode_pair(&mut main_reader, tbl) {
is[i] = x;
is[i + 1] = y;
}
}
}

// Count1 region (quads: v,w,x,y each 1 bit)
let mut i = big_values * 2;
                while i < 572 && main_reader.bits_remaining() > 0 {
                    let count1_tbl = if g.count1table_select == 1 { 32 } else { 33 };
                    if let Some(tbl) = get_table(count1_tbl) {
                        if let Ok((x, y)) = decode_pair(&mut main_reader, tbl) {
                            if i + 1 < 576 { is[i] = x; is[i + 1] = y; }
                        }
                    }
                    i += 2;
                }

                // --- Dequantize ---
                let mut xr = [0.0f64; 576];
                let scale = dequant_scale(g.global_gain);

                for sfb in 0..21 {
                    let start = SFB_LONG_44100[sfb];
                    let end   = SFB_LONG_44100[sfb + 1];
                    let sf = scalefac_scale(scalefacs[sfb], g.scalefac_scale);
                    for idx in start..end {
                        if idx < 576 {
                            xr[idx] = pow43(is[idx]) * scale * sf;
                        }
                    }
                }

                // --- IMDCT + overlap-add ---
                let block_type = match g.block_type {
                    0 => BlockType::Normal,
                    1 => BlockType::StartBlock,
                    2 => BlockType::ShortBlocks,
                    3 => BlockType::StopBlock,
                    _ => BlockType::Normal,
                };

                let mut pcm_out = vec![0.0f64; 576];
                if block_type == BlockType::ShortBlocks {
                    // Three short blocks per subband
                    for sb in 0..32 {
                        let base = sb * 18;
                        for win in 0..3 {
                            let mut freq_in = [0.0f64; 6];
                            for k in 0..6 {
                                let idx = base + win * 6 + k;
                                freq_in[k] = if idx < 576 { xr[idx] } else { 0.0 };
                            }
                            let time_out = imdct12(&freq_in);
                            for n in 0..12 {
                                let out_idx = sb * 18 + win * 6 + n;
                                if out_idx < 576 { pcm_out[out_idx] += time_out[n]; }
                            }
                        }
                    }
                } else {
                    for sb in 0..32 {
                        let mut freq_in = [0.0f64; 18];
                        for k in 0..18 {
                            let idx = sb * 18 + k;
                            freq_in[k] = if idx < 576 { xr[idx] } else { 0.0 };
                        }
                        let time_out = imdct36(&freq_in);

                        // Overlap-add
                        for n in 0..18 {
                            let out_idx = sb * 18 + n;
                            if out_idx < 576 {
                                pcm_out[out_idx] = time_out[n] + self.overlap[ch][sb][n];
                            }
                        }
                        // Store second half in overlap buffer
                        for n in 0..18 {
                            self.overlap[ch][sb][n] = time_out[n + 18];
                        }
                    }
                }

                // Convert f64 PCM → i16 samples
                let gran_offset = gran * (samples_per_frame / 2);
                for s in 0..576.min(samples_per_frame / 2) {
                    let sample = (pcm_out[s] * 32767.0).clamp(-32768.0, 32767.0) as i16;
                    let idx = gran_offset + s;
                    if idx < audio.samples[ch].len() {
                        audio.samples[ch][idx] = sample;
                    }
                }
            }
        }

        Ok(audio)
    }

    /// Decode from a complete in-memory byte slice (e.g. a .mp3 file).
    pub fn decode_all(&mut self, data: &[u8]) -> Vec<AudioFrame> {
        self.feed(data)
    }

    pub fn reset(&mut self) {
        self.buf.clear();
        self.overlap = vec![[[0.0f64; 18]; 32]; 2];
    }
}

impl Default for Mp3Decoder {
    fn default() -> Self { Self::new() }
}
