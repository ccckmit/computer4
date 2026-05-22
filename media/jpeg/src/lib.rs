use std::collections::HashMap;

pub mod encoder;
pub mod decoder;

pub use encoder::JpegEncoder;
pub use decoder::JpegDecoder;

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Image {
    pub fn new(width: u16, height: u16, data: Vec<u8>) -> Self {
        Self { width, height, data }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if x >= self.width as usize || y >= self.height as usize {
            return None;
        }
        let idx = (y * self.width as usize + x) * 3;
        Some(Color {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
        })
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width as usize || y >= self.height as usize {
            return;
        }
        let idx = (y * self.width as usize + x) * 3;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
    }

    pub fn to_rgb(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn from_rgb(width: u16, height: u16, rgb: Vec<u8>) -> Self {
        Self { width, height, data: rgb }
    }
}

pub struct RgbImage {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 3]>,
}

impl RgbImage {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![[0, 0, 0]; width * height],
        }
    }

    pub fn from_raw(width: usize, height: usize, data: &[u8]) -> Self {
        let mut pixels = Vec::with_capacity(width * height);
        for chunk in data.chunks(3) {
            if chunk.len() == 3 {
                pixels.push([chunk[0], chunk[1], chunk[2]]);
            }
        }
        Self { width, height, pixels }
    }

    pub fn to_raw(&self) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.width * self.height * 3);
        for pixel in &self.pixels {
            data.push(pixel[0]);
            data.push(pixel[1]);
            data.push(pixel[2]);
        }
        data
    }
}

pub struct YCbCrImage {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>,
    pub cb: Vec<u8>,
    pub cr: Vec<u8>,
}

impl YCbCrImage {
    pub fn from_rgb(rgb: &RgbImage) -> Self {
        let mut y = Vec::with_capacity(rgb.width * rgb.height);
        let mut cb = Vec::with_capacity(rgb.width * rgb.height);
        let mut cr = Vec::with_capacity(rgb.width * rgb.height);

        for pixel in &rgb.pixels {
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            let y_val = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
            let cb_val = (128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b).round() as u8;
            let cr_val = (128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b).round() as u8;

            y.push(y_val);
            cb.push(cb_val);
            cr.push(cr_val);
        }

        Self {
            width: rgb.width,
            height: rgb.height,
            y,
            cb,
            cr,
        }
    }

    pub fn to_rgb(&self) -> RgbImage {
        let mut pixels = Vec::with_capacity(self.width * self.height);

        for i in 0..self.y.len() {
            let y_val = self.y[i] as f32;
            let cb_val = self.cb[i] as f32 - 128.0;
            let cr_val = self.cr[i] as f32 - 128.0;

            let r = (y_val + 1.402 * cr_val).clamp(0.0, 255.0) as u8;
            let g = (y_val - 0.344136 * cb_val - 0.714136 * cr_val).clamp(0.0, 255.0) as u8;
            let b = (y_val + 1.772 * cb_val).clamp(0.0, 255.0) as u8;

            pixels.push([r, g, b]);
        }

        RgbImage {
            width: self.width,
            height: self.height,
            pixels,
        }
    }
}

pub fn rgb_to_ycbcr(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let r = r as f32;
    let g = g as f32;
    let b = b as f32;

    let y = (0.299 * r + 0.587 * g + 0.114 * b).round() as u8;
    let cb = (128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b).round() as u8;
    let cr = (128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b).round() as u8;

    (y, cb, cr)
}

pub fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y = y as f32;
    let cb = cb as f32 - 128.0;
    let cr = cr as f32 - 128.0;

    let r = (y + 1.402 * cr).clamp(0.0, 255.0) as u8;
    let g = (y - 0.344136 * cb - 0.714136 * cr).clamp(0.0, 255.0) as u8;
    let b = (y + 1.772 * cb).clamp(0.0, 255.0) as u8;

    (r, g, b)
}

pub struct DctCoefficients {
    pub dc: i16,
    pub ac: Vec<i16>,
}

#[derive(Clone)]
pub struct QuantizationTable {
    pub values: [[u8; 8]; 8],
    pub id: u8,
}

impl QuantizationTable {
    pub fn luminance() -> Self {
        Self {
            id: 0,
            values: [
                [16, 11, 10, 16, 24, 40, 51, 61],
                [12, 12, 14, 19, 26, 58, 60, 55],
                [14, 13, 16, 24, 40, 57, 69, 56],
                [14, 17, 22, 29, 51, 87, 80, 62],
                [18, 22, 37, 56, 68, 109, 103, 77],
                [24, 35, 55, 64, 81, 104, 113, 92],
                [49, 64, 78, 87, 103, 121, 120, 101],
                [72, 92, 95, 98, 112, 100, 103, 99],
            ],
        }
    }

    pub fn chrominance() -> Self {
        Self {
            id: 1,
            values: [
                [17, 18, 24, 47, 99, 99, 99, 99],
                [18, 21, 26, 66, 99, 99, 99, 99],
                [24, 26, 56, 99, 99, 99, 99, 99],
                [47, 66, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
                [99, 99, 99, 99, 99, 99, 99, 99],
            ],
        }
    }
}

#[derive(Clone)]
pub struct HuffmanTable {
    pub table_type: u8,
    pub table_id: u8,
    pub codes: HashMap<u8, (u16, usize)>,
    pub lookup: HashMap<(u16, usize), u8>,
}

impl HuffmanTable {
    pub fn dc_luminance() -> Self {
        let mut table = Self {
            table_type: 0,
            table_id: 0,
            codes: HashMap::new(),
            lookup: HashMap::new(),
        };
        table.build_dc_luminance();
        table
    }

    pub fn dc_chrominance() -> Self {
        let mut table = Self {
            table_type: 0,
            table_id: 1,
            codes: HashMap::new(),
            lookup: HashMap::new(),
        };
        table.build_dc_chrominance();
        table
    }

    pub fn ac_luminance() -> Self {
        let mut table = Self {
            table_type: 1,
            table_id: 0,
            codes: HashMap::new(),
            lookup: HashMap::new(),
        };
        table.build_ac_luminance();
        table
    }

    pub fn ac_chrominance() -> Self {
        let mut table = Self {
            table_type: 1,
            table_id: 1,
            codes: HashMap::new(),
            lookup: HashMap::new(),
        };
        table.build_ac_chrominance();
        table
    }

    fn build_dc_luminance(&mut self) {
        self.codes.insert(0, (0b00, 2));
        self.codes.insert(1, (0b010, 3));
        self.codes.insert(2, (0b011, 3));
        self.codes.insert(3, (0b100, 4));
        self.codes.insert(4, (0b101, 4));
        self.codes.insert(5, (0b110, 4));
        self.codes.insert(6, (0b1110, 5));
        self.codes.insert(7, (0b11110, 6));
        self.codes.insert(8, (0b111110, 7));
        self.codes.insert(9, (0b1111110, 8));
        self.codes.insert(10, (0b11111110, 9));
        self.codes.insert(11, (0b111111110, 10));
        for (cat, (code, len)) in &self.codes {
            self.lookup.insert((*code, *len), *cat);
        }
    }

    fn build_dc_chrominance(&mut self) {
        self.codes.insert(0, (0b00, 2));
        self.codes.insert(1, (0b01, 2));
        self.codes.insert(2, (0b10, 2));
        self.codes.insert(3, (0b110, 3));
        self.codes.insert(4, (0b1110, 4));
        self.codes.insert(5, (0b11110, 5));
        self.codes.insert(6, (0b111110, 6));
        self.codes.insert(7, (0b1111110, 7));
        self.codes.insert(8, (0b11111110, 8));
        self.codes.insert(9, (0b111111110, 9));
        self.codes.insert(10, (0b1111111110, 10));
        self.codes.insert(11, (0b11111111110, 11));
        for (cat, (code, len)) in &self.codes {
            self.lookup.insert((*code, *len), *cat);
        }
    }

    fn build_ac_luminance(&mut self) {
        let counts: [u8; 16] = [0, 2, 1, 3, 3, 2, 4, 3, 5, 5, 4, 4, 0, 0, 1, 125];
        let symbols: [u8; 162] = [
            0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41,
            0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91,
            0x08, 0x23, 0x42, 0xA1, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24,
            0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A,
            0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38,
            0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66,
            0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
            0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92, 0x93,
            0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
            0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7,
            0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9,
            0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1,
            0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1,
            0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
        ];
        self.build_table_from_counts(&counts, &symbols);
    }

    fn build_ac_chrominance(&mut self) {
        let counts: [u8; 16] = [0, 2, 1, 2, 4, 4, 3, 4, 7, 5, 4, 4, 0, 1, 2, 119];
        let symbols: [u8; 162] = [
            0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41,
            0x06, 0x13, 0x51, 0x61, 0x07, 0x22, 0x71, 0x14, 0x32, 0x81, 0x91,
            0x08, 0x23, 0x42, 0xA1, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0, 0x24,
            0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A,
            0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38,
            0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x53,
            0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66,
            0x67, 0x68, 0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79,
            0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x8A, 0x92,
            0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4,
            0xA5, 0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6,
            0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5, 0xC6, 0xC7, 0xC8,
            0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
            0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1,
            0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8, 0xF9, 0xFA,
        ];
        self.build_table_from_counts(&counts, &symbols);
        self.table_id = 1;
    }

    fn build_table_from_counts(&mut self, counts: &[u8; 16], symbols: &[u8]) {
        let mut code = 0u16;
        let mut sym_idx = 0;
        for (len_idx, &count) in counts.iter().enumerate() {
            let len = len_idx + 1;
            for _ in 0..count {
                let symbol = symbols[sym_idx];
                self.codes.insert(symbol, (code, len));
                self.lookup.insert((code, len), symbol);
                code += 1;
                sym_idx += 1;
            }
            code <<= 1;
        }
    }
}

pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit_pos: u8,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_pos: 0,
        }
    }

    pub fn read_bits(&mut self, n: u8) -> u16 {
        let mut result = 0u16;
        for _ in 0..n {
            if self.pos >= self.data.len() {
                break;
            }
            let byte = self.data[self.pos];
            let bit = (byte >> (7 - self.bit_pos)) & 1;
            result = (result << 1) | bit as u16;
            self.bit_pos += 1;
            if self.bit_pos >= 8 {
                self.bit_pos = 0;
                self.pos += 1;
            }
        }
        result
    }

    pub fn align(&mut self) {
        if self.bit_pos > 0 {
            self.bit_pos = 0;
            self.pos += 1;
        }
    }

    pub fn peek_byte(&self) -> Option<u8> {
        if self.pos < self.data.len() {
            Some(self.data[self.pos])
        } else {
            None
        }
    }
}

pub struct BitWriter {
    data: Vec<u8>,
    current_byte: u8,
    bit_pos: u8,
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            current_byte: 0,
            bit_pos: 0,
        }
    }

    pub fn write_bits(&mut self, value: u16, num_bits: u8) {
        for i in (0..num_bits).rev() {
            let bit = ((value >> i) & 1) as u8;
            self.current_byte = (self.current_byte << 1) | bit;
            self.bit_pos += 1;
            if self.bit_pos == 8 {
                self.data.push(self.current_byte);
                self.current_byte = 0;
                self.bit_pos = 0;
            }
        }
    }

    pub fn finalize(mut self) -> Vec<u8> {
        if self.bit_pos > 0 {
            self.current_byte <<= 8 - self.bit_pos;
            self.data.push(self.current_byte);
        }
        self.data
    }

    pub fn len(&self) -> usize {
        self.data.len() + if self.bit_pos > 0 { 1 } else { 0 }
    }
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

pub fn forward_dct(block: &[[i16; 8]; 8]) -> [[i16; 8]; 8] {
    let mut result = [[0i16; 8]; 8];
    let mut temp = [[0.0_f32; 8]; 8];

    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0.0f32;
            for x in 0..8 {
                for y in 0..8 {
                    let pixel = block[x][y] as f32;
                    let cos_x = (std::f32::consts::PI * (2.0 * x as f32 + 1.0) * u as f32 / 16.0).cos();
                    let cos_y = (std::f32::consts::PI * (2.0 * y as f32 + 1.0) * v as f32 / 16.0).cos();
                    sum += pixel * cos_x * cos_y;
                }
            }
            let cu = if u == 0 { 1.0 / std::f32::consts::SQRT_2 } else { 1.0 };
            let cv = if v == 0 { 1.0 / std::f32::consts::SQRT_2 } else { 1.0 };
            temp[u][v] = 0.25 * cu * cv * sum;
        }
    }

    for i in 0..8 {
        for j in 0..8 {
            result[i][j] = (temp[i][j] + 0.5) as i16;
        }
    }

    result
}

pub fn inverse_dct(block: &[[i16; 8]; 8]) -> [[i16; 8]; 8] {
    let mut result = [[0i16; 8]; 8];

    for x in 0..8 {
        for y in 0..8 {
            let mut sum = 0.0f32;
            for u in 0..8 {
                for v in 0..8 {
                    let pixel = block[u][v] as f32;
                    let cu = if u == 0 { 1.0 / std::f32::consts::SQRT_2 } else { 1.0 };
                    let cv = if v == 0 { 1.0 / std::f32::consts::SQRT_2 } else { 1.0 };
                    let cos_u = (std::f32::consts::PI * (2.0 * x as f32 + 1.0) * u as f32 / 16.0).cos();
                    let cos_v = (std::f32::consts::PI * (2.0 * y as f32 + 1.0) * v as f32 / 16.0).cos();
                    sum += cu * cv * pixel * cos_u * cos_v;
                }
            }
            result[x][y] = (sum * 0.25 + 128.0).clamp(0.0, 255.0) as i16;
        }
    }

    result
}

pub fn zigzag_order() -> [(usize, usize); 64] {
    [
        (0, 0), (0, 1), (1, 0), (2, 0), (1, 1), (0, 2), (0, 3), (1, 2),
        (2, 1), (3, 0), (4, 0), (3, 1), (2, 2), (1, 3), (0, 4), (0, 5),
        (1, 4), (2, 3), (3, 2), (4, 1), (5, 0), (6, 0), (5, 1), (4, 2),
        (3, 3), (2, 4), (1, 5), (0, 6), (0, 7), (1, 6), (2, 5), (3, 4),
        (4, 3), (5, 2), (6, 1), (7, 0), (7, 1), (6, 2), (5, 3), (4, 4),
        (3, 5), (2, 6), (1, 7), (2, 7), (3, 6), (4, 5), (5, 4), (6, 3),
        (7, 2), (7, 3), (6, 4), (5, 5), (4, 6), (3, 7), (4, 7), (5, 6),
        (6, 5), (7, 4), (7, 5), (6, 6), (5, 7), (6, 7), (7, 6), (7, 7),
    ]
}

pub fn encode_coefficients(dc: i16, ac: &[i16]) -> Vec<(u8, i16)> {
    let mut result = Vec::new();

    let dc_category = if dc == 0 {
        0
    } else {
        (dc.abs().ilog2() + 1) as u8
    };
    let dc_magnitude = if dc >= 0 { dc as u16 } else { (dc as i32 + 1).unsigned_abs() as u16 };
    result.push((dc_category, dc_magnitude as i16));

    let mut zero_count = 0;
    for &coef in ac {
        if coef == 0 {
            zero_count += 1;
        } else {
            while zero_count >= 16 {
                result.push((0x00, 0));
                zero_count -= 16;
            }
            let category = if coef >= 0 {
                (coef.ilog2() + 1) as u8
            } else {
                ((-coef).ilog2() + 1) as u8
            };
            let magnitude = if coef >= 0 {
                coef as u16
            } else {
                ((coef + 1) as i32).unsigned_abs() as u16
            };
            let rs = (zero_count << 4) | category;
            result.push((rs, magnitude as i16));
            zero_count = 0;
        }
    }

    if zero_count > 0 || result.len() == 1 {
        result.push((0x00, 0));
    }

    result
}

pub fn decode_coefficients(decoder: &mut BitReader, dc_table: &HuffmanTable, ac_table: &HuffmanTable, prev_dc: i16) -> (i16, Vec<i16>) {
    let mut ac = vec![0i16; 63];
    let dc_category = decode_huffman(decoder, dc_table);
    let dc = if dc_category == 0 {
        prev_dc
    } else {
        let bits = decoder.read_bits(dc_category as u8) as i16;
        let delta = if bits < (1i16 << (dc_category - 1)) {
            (bits as i32) - ((1i32 << dc_category) - 1)
        } else {
            bits as i32
        };
        ((prev_dc as i32) + delta).clamp(i16::MIN as i32, i16::MAX as i32) as i16
    };

    let mut i = 0;
    while i < 63 {
        let rs = decode_huffman(decoder, ac_table);
        let zero_count = (rs >> 4) as usize;
        let category = (rs & 0x0F) as u8;

        i += zero_count;

        if category == 0 {
            break;
        }

        if i < 63 {
            let bits = decoder.read_bits(category as u8) as i16;
            let magnitude = if bits < (1i16 << (category - 1)) {
                (bits as i32) - ((1i32 << category) - 1)
            } else {
                bits as i32
            };
            ac[i] = magnitude.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            i += 1;
        }
    }

    (dc, ac)
}

pub fn decode_huffman(decoder: &mut BitReader, table: &HuffmanTable) -> u8 {
    let mut code = 0u16;
    for len in 1..=16 {
        code = (code << 1) | decoder.read_bits(1) as u16;
        if let Some(&value) = table.lookup.get(&(code, len)) {
            return value;
        }
    }
    0
}

pub fn quantize(block: &[[i16; 8]; 8], table: &QuantizationTable) -> [[i16; 8]; 8] {
    let mut result = [[0i16; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            let q = table.values[i][j] as i16;
            if q > 0 {
                let val = (block[i][j] as i32) + if block[i][j] > 0 { q as i32 / 2 } else { -(q as i32 / 2) };
                result[i][j] = (val / q as i32).clamp(i16::MIN as i32, i16::MAX as i32) as i16;
            }
        }
    }
    result
}

pub fn dequantize(block: &[[i16; 8]; 8], table: &QuantizationTable) -> [[i16; 8]; 8] {
    let mut result = [[0i16; 8]; 8];
    for i in 0..8 {
        for j in 0..8 {
            let v = (block[i][j] as i32) * (table.values[i][j] as i32);
            result[i][j] = v.clamp(i16::MIN as i32, i16::MAX as i32) as i16;
        }
    }
    result
}