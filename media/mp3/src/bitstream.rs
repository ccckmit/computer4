/// Bit-level reader for MPEG bitstreams
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_pos: usize,
    bit_pos: u8, // 0 = MSB of current byte
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BitReader { data, byte_pos: 0, bit_pos: 0 }
    }

    pub fn new_at(data: &'a [u8], byte_offset: usize) -> Self {
        BitReader { data, byte_pos: byte_offset, bit_pos: 0 }
    }

    /// Read `n` bits (up to 32), MSB-first
    pub fn read_bits(&mut self, n: u8) -> Option<u32> {
        if n == 0 { return Some(0); }
        let mut result: u32 = 0;
        for _ in 0..n {
            if self.byte_pos >= self.data.len() { return None; }
            let bit = (self.data[self.byte_pos] >> (7 - self.bit_pos)) & 1;
            result = (result << 1) | bit as u32;
            self.bit_pos += 1;
            if self.bit_pos == 8 {
                self.bit_pos = 0;
                self.byte_pos += 1;
            }
        }
        Some(result)
    }

    pub fn read_bit(&mut self) -> Option<bool> {
        self.read_bits(1).map(|b| b != 0)
    }

    pub fn bits_remaining(&self) -> usize {
        let bytes_left = self.data.len().saturating_sub(self.byte_pos);
        bytes_left * 8 - self.bit_pos as usize
    }

    pub fn byte_pos(&self) -> usize { self.byte_pos }
    pub fn bit_pos(&self) -> u8 { self.bit_pos }

    /// Align to next byte boundary
    pub fn align_byte(&mut self) {
        if self.bit_pos != 0 {
            self.bit_pos = 0;
            self.byte_pos += 1;
        }
    }
}

/// Bit-level writer for encoding MPEG bitstreams
pub struct BitWriter {
    data: Vec<u8>,
    bit_pos: u8, // bits written in current byte
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter { data: Vec::new(), bit_pos: 0 }
    }

    pub fn with_capacity(cap: usize) -> Self {
        BitWriter { data: Vec::with_capacity(cap), bit_pos: 0 }
    }

    /// Write `n` bits from `value` (MSB first)
    pub fn write_bits(&mut self, value: u32, n: u8) {
        for i in (0..n).rev() {
            let bit = ((value >> i) & 1) as u8;
            if self.bit_pos == 0 {
                self.data.push(0);
            }
            let last = self.data.last_mut().unwrap();
            *last |= bit << (7 - self.bit_pos);
            self.bit_pos = (self.bit_pos + 1) % 8;
        }
    }

    pub fn write_bit(&mut self, bit: bool) {
        self.write_bits(bit as u32, 1);
    }

    /// Align to byte boundary by writing zero bits
    pub fn align_byte(&mut self) {
        if self.bit_pos != 0 {
            self.bit_pos = 0;
        }
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        self.align_byte();
        self.data
    }

    pub fn len_bits(&self) -> usize {
        self.data.len() * 8 - (if self.bit_pos == 0 { 0 } else { 8 - self.bit_pos as usize })
    }
}

impl Default for BitWriter {
    fn default() -> Self { Self::new() }
}
