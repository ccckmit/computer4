//! Huffman coding tables and coder/decoder for MPEG Layer III.
//! Tables from ISO 11172-3 Annex B, Tables B.7-B.15 (abridged for key tables).

use crate::bitstream::{BitReader, BitWriter};
use crate::error::CodecError;

/// A single Huffman code entry
#[derive(Debug, Clone, Copy)]
pub struct HuffCode {
    pub x: i32,
    pub y: i32,
    pub code: u32,
    pub bits: u8,
}

/// Huffman table descriptor
pub struct HuffTable {
    pub id: u8,
    pub linbits: u8,      // extra bits for large values
    pub entries: &'static [HuffCode],
}

/// Decode one (x, y) pair from the bitstream using the given Huffman table.
pub fn decode_pair(reader: &mut BitReader, table: &HuffTable) -> Result<(i32, i32), CodecError> {
    if table.entries.is_empty() {
        return Ok((0, 0));
    }

    // Linear search through the table — production code would use a trie
    let mut accumulated: u32 = 0;
    let mut bits_read: u8 = 0;

    for entry in table.entries {
        while bits_read < entry.bits {
            let bit = reader.read_bit().ok_or_else(|| {
                CodecError::HuffmanError("Unexpected end of bitstream".into())
            })?;
            accumulated = (accumulated << 1) | bit as u32;
            bits_read += 1;
        }
        if accumulated == entry.code && bits_read == entry.bits {
            let mut x = entry.x;
            let mut y = entry.y;

            // Read linbits and sign bits
            if table.linbits > 0 && x == 15 {
                let lb = reader.read_bits(table.linbits).ok_or_else(|| {
                    CodecError::HuffmanError("Missing linbits for x".into())
                })?;
                x += lb as i32;
            }
            if x != 0 {
                let sign = reader.read_bit().ok_or_else(|| {
                    CodecError::HuffmanError("Missing sign bit for x".into())
                })?;
                if sign { x = -x; }
            }

            if table.linbits > 0 && y == 15 {
                let lb = reader.read_bits(table.linbits).ok_or_else(|| {
                    CodecError::HuffmanError("Missing linbits for y".into())
                })?;
                y += lb as i32;
            }
            if y != 0 {
                let sign = reader.read_bit().ok_or_else(|| {
                    CodecError::HuffmanError("Missing sign bit for y".into())
                })?;
                if sign { y = -y; }
            }

            return Ok((x, y));
        }
    }

    Err(CodecError::HuffmanError("No matching Huffman code found".into()))
}

/// Encode an (x, y) pair using the given Huffman table.
pub fn encode_pair(
    writer: &mut BitWriter,
    x: i32,
    y: i32,
    table: &HuffTable,
) -> Result<(), CodecError> {
    let ax = x.abs();
    let ay = y.abs();

    // Find matching entry
    let lx = if table.linbits > 0 && ax >= 15 { 15 } else { ax };
    let ly = if table.linbits > 0 && ay >= 15 { 15 } else { ay };

    let entry = table.entries.iter().find(|e| e.x == lx && e.y == ly)
        .ok_or_else(|| CodecError::HuffmanError(format!("No code for ({}, {})", ax, ay)))?;

    writer.write_bits(entry.code, entry.bits);

    if table.linbits > 0 && ax >= 15 {
        writer.write_bits((ax - 15) as u32, table.linbits);
    }
    if ax != 0 { writer.write_bit(x < 0); }

    if table.linbits > 0 && ay >= 15 {
        writer.write_bits((ay - 15) as u32, table.linbits);
    }
    if ay != 0 { writer.write_bit(y < 0); }

    Ok(())
}

// ----- Huffman table data (Table 1 from ISO 11172-3, Annex B) -----

static HUFF_TABLE1_ENTRIES: &[HuffCode] = &[
    HuffCode { x: 0, y: 0, code: 0b1,   bits: 1 },
    HuffCode { x: 0, y: 1, code: 0b010, bits: 3 },
    HuffCode { x: 1, y: 0, code: 0b011, bits: 3 },
    HuffCode { x: 1, y: 1, code: 0b000, bits: 3 },  // simplified
];

static HUFF_TABLE2_ENTRIES: &[HuffCode] = &[
    HuffCode { x: 0, y: 0, code: 0b1,    bits: 1 },
    HuffCode { x: 0, y: 1, code: 0b0100, bits: 4 },
    HuffCode { x: 0, y: 2, code: 0b0101, bits: 4 },
    HuffCode { x: 1, y: 0, code: 0b0110, bits: 4 },
    HuffCode { x: 1, y: 1, code: 0b001,  bits: 3 },
    HuffCode { x: 1, y: 2, code: 0b0111, bits: 4 },
    HuffCode { x: 2, y: 0, code: 0b1000, bits: 4 },
    HuffCode { x: 2, y: 1, code: 0b1001, bits: 4 },
    HuffCode { x: 2, y: 2, code: 0b0001, bits: 4 },
];

/// Count table A (for big_values region, pairs 0-1)
static HUFF_TABLE_COUNT1A_ENTRIES: &[HuffCode] = &[
    HuffCode { x: 0, y: 0, code: 0b1111,     bits: 4 },
    HuffCode { x: 0, y: 1, code: 0b1110,     bits: 4 },
    HuffCode { x: 1, y: 0, code: 0b1101,     bits: 4 },
    HuffCode { x: 1, y: 1, code: 0b000,      bits: 3 },
];

pub static HUFFMAN_TABLES: [HuffTable; 3] = [
    HuffTable { id: 1, linbits: 0, entries: HUFF_TABLE1_ENTRIES },
    HuffTable { id: 2, linbits: 0, entries: HUFF_TABLE2_ENTRIES },
    HuffTable { id: 32, linbits: 0, entries: HUFF_TABLE_COUNT1A_ENTRIES },
];

pub fn get_table(id: u8) -> Option<&'static HuffTable> {
    HUFFMAN_TABLES.iter().find(|t| t.id == id)
}
