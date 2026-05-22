use crate::bitstream::BitReader;
use crate::vlc::{decode_vlc, MACROBLOCK_ADDRESS_INCREMENT_VLC, DCT_DC_LUMINANCE_VLC, DCT_AC_VLC, MOTION_VECTOR_VLC};
use std::io::Result;

/// Sequence Header（0xB3）
pub struct SequenceHeader {
    pub horizontal_size: u16,
    pub vertical_size: u16,
    // 其它字段（aspect_ratio、frame_rate、bit_rate 等）在这里略去，可自行补全
}
impl SequenceHeader {
    pub fn parse<R: std::io::Read>(br: &mut BitReader<R>) -> Result<Self> {
        let horizontal_size = br.read_bits(12)? as u16;
        let vertical_size = br.read_bits(12)? as u16;
        // 跳过 aspect_ratio_information、frame_rate_code、bit_rate、marker_bit、vbv_buffer_size、constrained_parameters_flag
        br.skip_bits(4 + 4 + 18 + 1 + 10 + 1)?; // 38 bits total
        // 量化矩阵可选，这里直接跳过（如果有则需要读取长度并丢弃）
        Ok(Self { horizontal_size, vertical_size })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PictureCodingType { I, P, B }
/// Picture Header（0x00）
pub struct PictureHeader {
    pub coding_type: PictureCodingType,
}
impl PictureHeader {
    pub fn parse<R: std::io::Read>(br: &mut BitReader<R>) -> Result<Self> {
        // temporal_reference（10 位）
        br.skip_bits(10)?;
        let pic_type = br.read_bits(3)?;
        let coding_type = match pic_type {
            1 => PictureCodingType::I,
            2 => PictureCodingType::P,
            3 => PictureCodingType::B,
            _ => return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Unsupported picture coding type",
            )),
        };
        // vbv_delay（16 位）
        br.skip_bits(16)?;
        // forward/backward motion_vector 以及 extra 变量这里省略
        Ok(Self { coding_type })
    }
}

/// Slice Header（0x01..0xAF）
pub struct SliceHeader {
    pub vertical_position: u8,
}
impl SliceHeader {
    pub fn parse<R: std::io::Read>(br: &mut BitReader<R>, start_code: u8) -> Result<Self> {
        // start_code 的低 8 位即为 slice_vertical_position
        Ok(Self { vertical_position: start_code })
    }
}

/// MacroBlock（简化版，仅演示必要字段）
pub struct MacroBlock {
    pub mb_x: usize,
    pub mb_y: usize,
    pub block_idx: usize,
    pub intra: bool,
    pub quantiser: u8,
    pub full_pel: bool,
    pub mv_x: i16,
    pub mv_y: i16,
    pub coeffs: [i16; 64],
}
impl MacroBlock {
    /// 解析一个宏块。`intra` 参数指示当前 picture 是否为 intra（I‑frame）
    pub fn parse<R: std::io::Read>(br: &mut BitReader<R>, intra: bool) -> Result<Self> {
        // macroblock_address_increment（这里使用表）
        let _inc = decode_vlc(br, MACROBLOCK_ADDRESS_INCREMENT_VLC)?;
        // 这里省略 macroblock_type、coded_block_pattern 等字段的完整解析
        let intra_flag = intra; // 对 I‑frame 均为 intra；P/B 需进一步判断（略）
        let quantiser = br.read_bits(5)? as u8;
        let full_pel = br.read_bits(1)? != 0;
        let (mv_x, mv_y) = if intra_flag {
            (0i16, 0i16)
        } else {
            let mx = decode_vlc(br, MOTION_VECTOR_VLC)? as i16;
            let my = decode_vlc(br, MOTION_VECTOR_VLC)? as i16;
            (mx, my)
        };
        // 读取 DCT 系数（DC + AC）
        let mut coeffs = [0i16; 64];
        if intra_flag {
            let dc = decode_vlc(br, DCT_DC_LUMINANCE_VLC)? as i16;
            coeffs[0] = dc;
        }
        // 读取 AC 系数（run, level）
        let mut i = if intra_flag { 1 } else { 0 };
        while i < 64 {
            let run = decode_vlc(br, DCT_AC_VLC)? as usize;
            if run == 0 { break; } // End_of_block
            let level = decode_vlc(br, DCT_AC_VLC)? as i16;
            i += run;
            if i < 64 {
                coeffs[i] = level;
                i += 1;
            } else { break; }
        }
        // 为简化，这里把宏块坐标置为 0，实际解码时需要根据 macroblock_address_increment 计算
        Ok(Self {
            mb_x: 0,
            mb_y: 0,
            block_idx: 0,
            intra: intra_flag,
            quantiser,
            full_pel,
            mv_x,
            mv_y,
            coeffs,
        })
    }
}
