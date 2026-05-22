/// 可变长度码（VLC）表的最小实现，只使用 std。
/// 每条记录： (码长, 码值, 对应的符号)
pub struct VlcEntry {
    pub bits: u8,
    pub code: u16,
    pub value: i16,
}

/* 下面仅列出常用表的前几项，完整表请参考 ISO/IEC 11172‑2 Annex B */
pub static MACROBLOCK_ADDRESS_INCREMENT_VLC: &[VlcEntry] = &[
    VlcEntry { bits: 1,  code: 0b1,      value: 1 },
    VlcEntry { bits: 3,  code: 0b011,    value: 2 },
    VlcEntry { bits: 3,  code: 0b010,    value: 3 },
    VlcEntry { bits: 4,  code: 0b0011,   value: 4 },
    VlcEntry { bits: 5,  code: 0b00101,  value: 5 },
    VlcEntry { bits: 5,  code: 0b00100,  value: 6 },
    // …（完整表共 33 条）
];

pub static DCT_DC_LUMINANCE_VLC: &[VlcEntry] = &[
    VlcEntry { bits: 2, code: 0b00,   value: 0 },
    VlcEntry { bits: 3, code: 0b010,  value: 1 },
    VlcEntry { bits: 3, code: 0b011,  value: -1 },
    VlcEntry { bits: 4, code: 0b1000, value: 2 },
    VlcEntry { bits: 4, code: 0b1001, value: -2 },
    // …（完整表共 11 条）
];

pub static DCT_AC_VLC: &[VlcEntry] = &[
    // (run, level) 组合的表，这里仅示例前几条
    VlcEntry { bits: 2,  code: 0b10,   value: 0 },   // End_of_block
    VlcEntry { bits: 6,  code: 0b111110, value: 1 }, // run=0, level=1
    VlcEntry { bits: 6,  code: 0b111111, value: -1 },// run=0, level=-1
    // …（完整表 163 条）
];

pub static MOTION_VECTOR_VLC: &[VlcEntry] = &[
    VlcEntry { bits: 2, code: 0b01, value: 0 },
    VlcEntry { bits: 3, code: 0b001, value: 1 },
    VlcEntry { bits: 3, code: 0b000, value: -1 },
    VlcEntry { bits: 4, code: 0b1001, value: 2 },
    VlcEntry { bits: 4, code: 0b1000, value: -2 },
    // …（完整表 64 条）
];

/// 在给定的 VLC 表中查找匹配的码并返回对应的符号
pub fn decode_vlc<R: std::io::Read>(
    br: &mut crate::bitstream::BitReader<R>,
    table: &[VlcEntry],
) -> std::io::Result<i16> {
    for entry in table {
        let bits = br.read_bits(entry.bits)?;
        if bits as u16 == entry.code {
            return Ok(entry.value);
        }
        // 若不匹配，需要把已经读走的位“回退”。这里采用最直接的做法：把读的位重新放回缓存。
        // 为了保持实现简洁，假设每次 decode_vlc 调用的码长不超过表中最大的 bits。
        br.left += entry.bits;
        br.buf = (br.buf << entry.bits) | (bits as u32);
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        "VLC decode failed",
    ))
}
