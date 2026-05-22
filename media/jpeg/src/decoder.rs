use super::{BitReader, HuffmanTable, QuantizationTable, inverse_dct, dequantize, zigzag_order, decode_huffman, decode_coefficients, ycbcr_to_rgb};

pub struct JpegDecoder {
    width: usize,
    height: usize,
    y_quant: QuantizationTable,
    c_quant: QuantizationTable,
    dc_y_table: HuffmanTable,
    dc_c_table: HuffmanTable,
    ac_y_table: HuffmanTable,
    ac_c_table: HuffmanTable,
}

impl JpegDecoder {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            y_quant: QuantizationTable::luminance(),
            c_quant: QuantizationTable::chrominance(),
            dc_y_table: HuffmanTable::dc_luminance(),
            dc_c_table: HuffmanTable::dc_chrominance(),
            ac_y_table: HuffmanTable::ac_luminance(),
            ac_c_table: HuffmanTable::ac_chrominance(),
        }
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String> {
        if data.len() < 2 || data[0] != 0xFF || data[1] != 0xD8 {
            return Err("Not a valid JPEG file".to_string());
        }

        let mut pos = 2;
        while pos < data.len() {
            if data[pos] != 0xFF {
                pos += 1;
                continue;
            }

            let marker = data[pos + 1];
            pos += 2;

            match marker {
                0xC0 | 0xC2 => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    if pos + 8 < data.len() {
                        let _precision = data[pos + 2];
                        self.height = ((data[pos + 3] as usize) << 8) | (data[pos + 4] as usize);
                        self.width = ((data[pos + 5] as usize) << 8) | (data[pos + 6] as usize);
                    }
                    pos += length;
                }
                0xDB => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    pos += 2;
                    if pos + length - 2 < data.len() {
                        let table_id = data[pos] & 0x0F;
                        let quant_table = self.read_quantization_table(&data[pos + 1..pos + 65], table_id);
                        if table_id == 0 {
                            self.y_quant = quant_table;
                        } else {
                            self.c_quant = quant_table;
                        }
                    }
                    pos += length - 2;
                }
                0xC4 => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    pos += 2;
                    let table_data = &data[pos..pos + length - 2];
                    self.parse_huffman_tables(table_data);
                    pos += length - 2;
                }
                0xDA => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    pos += length;
                    break;
                }
                0xD9 => {
                    break;
                }
                0xE0 | 0xE1 | 0xE2 | 0xE3 | 0xE4 | 0xE5 | 0xE6 | 0xE7 |
                0xE8 | 0xE9 | 0xEA | 0xEB | 0xEC | 0xED | 0xEE | 0xEF => {
                    if pos < data.len() {
                        let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                        pos += length;
                    }
                }
                _ => {
                    if pos < data.len() {
                        pos += 1;
                    }
                }
            }
        }

        if self.width == 0 || self.height == 0 {
            return Err("Could not determine image dimensions".to_string());
        }

        let y_size = self.width * self.height;
        let c_size = y_size / 4;
        let mut y_data = vec![0u8; y_size];
        let mut cb_data = vec![128u8; c_size];
        let mut cr_data = vec![128u8; c_size];

        let num_blocks_y = ((self.width + 7) / 8) * ((self.height + 7) / 8);
        let num_blocks_c = ((self.width / 2 + 7) / 8) * ((self.height / 2 + 7) / 8);

        let mut bit_reader = BitReader::new(&data[pos..]);

        let mut prev_dc_y = 0i16;
        let mut prev_dc_cb = 0i16;
        let mut prev_dc_cr = 0i16;

        for block_idx in 0..num_blocks_y {
            let by = block_idx / ((self.width + 7) / 8);
            let bx = block_idx % ((self.width + 7) / 8);

            let (dc, ac) = decode_coefficients(&mut bit_reader, &self.dc_y_table, &self.ac_y_table, prev_dc_y);
            prev_dc_y = dc;

            let mut block = [[0i16; 8]; 8];
            block[0][0] = dc;

            let zigzag = zigzag_order();
            for (i, &(x, y)) in zigzag.iter().enumerate().skip(1) {
                if i - 1 < ac.len() {
                    block[x][y] = ac[i - 1];
                }
            }

            let dq_block = dequantize(&block, &self.y_quant);
            let idct_block = inverse_dct(&dq_block);

            for j in 0..8 {
                for i in 0..8 {
                    let px = bx * 8 + i;
                    let py = by * 8 + j;
                    if px < self.width && py < self.height {
                        y_data[py * self.width + px] = idct_block[j][i].clamp(0, 255) as u8;
                    }
                }
            }
        }

        for block_idx in 0..num_blocks_c {
            let by = block_idx / ((self.width / 2 + 7) / 8);
            let bx = block_idx % ((self.width / 2 + 7) / 8);

            let (dc_cb, ac_cb) = decode_coefficients(&mut bit_reader, &self.dc_c_table, &self.ac_c_table, prev_dc_cb);
            prev_dc_cb = dc_cb;

            let (dc_cr, ac_cr) = decode_coefficients(&mut bit_reader, &self.dc_c_table, &self.ac_c_table, prev_dc_cr);
            prev_dc_cr = dc_cr;

            let mut cb_block = [[0i16; 8]; 8];
            let mut cr_block = [[0i16; 8]; 8];
            cb_block[0][0] = dc_cb;
            cr_block[0][0] = dc_cr;

            let zigzag = zigzag_order();
            for (i, &(x, y)) in zigzag.iter().enumerate().skip(1) {
                if i - 1 < ac_cb.len() {
                    cb_block[x][y] = ac_cb[i - 1];
                    cr_block[x][y] = ac_cr[i - 1];
                }
            }

            let cb_dq = dequantize(&cb_block, &self.c_quant);
            let cr_dq = dequantize(&cr_block, &self.c_quant);
            let cb_idct = inverse_dct(&cb_dq);
            let cr_idct = inverse_dct(&cr_dq);

            for j in 0..8 {
                for i in 0..8 {
                    let px = bx * 8 + i;
                    let py = by * 8 + j;
                    if px < self.width / 2 && py < self.height / 2 {
                        let idx = py * (self.width / 2) + px;
                        if idx < cb_data.len() {
                            cb_data[idx] = cb_idct[j][i].clamp(0, 255) as u8;
                            cr_data[idx] = cr_idct[j][i].clamp(0, 255) as u8;
                        }
                    }
                }
            }
        }

        Ok((y_data, cb_data, cr_data))
    }

    pub fn decode_to_rgb(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        let (y_data, cb_data, cr_data) = self.decode(data)?;

        let mut rgb = Vec::with_capacity(self.width * self.height * 3);

        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = y * self.width + x;
                let cb_idx = (y / 2) * (self.width / 2) + (x / 2);

                let y_val = if y_idx < y_data.len() { y_data[y_idx] } else { 128 };
                let cb_val = if cb_idx < cb_data.len() { cb_data[cb_idx] } else { 128 };
                let cr_val = if cb_idx < cr_data.len() { cr_data[cb_idx] } else { 128 };

                let (r, g, b) = ycbcr_to_rgb(y_val, cb_val, cr_val);
                rgb.push(r);
                rgb.push(g);
                rgb.push(b);
            }
        }

        Ok(rgb)
    }

    fn read_quantization_table(&self, data: &[u8], _table_id: u8) -> QuantizationTable {
        let mut values = [[0u8; 8]; 8];
        let zigzag = zigzag_order();
        for (i, &(x, y)) in zigzag.iter().enumerate() {
            if i < data.len() {
                values[x][y] = data[i];
            }
        }
        QuantizationTable { values, id: _table_id }
    }

    fn parse_huffman_tables(&mut self, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let _table_class_and_id = data[0];
        let table_class = (_table_class_and_id >> 4) & 0x0F;
        let table_id = _table_class_and_id & 0x0F;

        let counts = &data[1..17];
        let symbols_start = 17;

        let mut symbols: Vec<u8> = Vec::new();
        for &count in counts {
            for _ in 0..count {
                if symbols_start + symbols.len() < data.len() {
                    symbols.push(data[symbols_start + symbols.len()]);
                }
            }
        }

        let mut code = 0u16;
        let mut symbol_idx = 0;

        for (len, &count) in counts.iter().enumerate() {
            for _ in 0..count {
                if symbol_idx < symbols.len() {
                    let symbol = symbols[symbol_idx];

                    if table_class == 0 && table_id == 0 {
                        self.dc_y_table.codes.insert(symbol, (code, len + 1));
                        self.dc_y_table.lookup.insert((code, len + 1), symbol);
                    } else if table_class == 0 && table_id == 1 {
                        self.dc_c_table.codes.insert(symbol, (code, len + 1));
                        self.dc_c_table.lookup.insert((code, len + 1), symbol);
                    } else if table_class == 1 && table_id == 0 {
                        self.ac_y_table.codes.insert(symbol, (code, len + 1));
                        self.ac_y_table.lookup.insert((code, len + 1), symbol);
                    } else if table_class == 1 && table_id == 1 {
                        self.ac_c_table.codes.insert(symbol, (code, len + 1));
                        self.ac_c_table.lookup.insert((code, len + 1), symbol);
                    }

                    symbol_idx += 1;
                }
                code += 1;
            }
            code <<= 1;
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}

impl Default for JpegDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for JpegDecoder {
    fn clone(&self) -> Self {
        Self {
            width: self.width,
            height: self.height,
            y_quant: self.y_quant.clone(),
            c_quant: self.c_quant.clone(),
            dc_y_table: self.dc_y_table.clone(),
            dc_c_table: self.dc_c_table.clone(),
            ac_y_table: self.ac_y_table.clone(),
            ac_c_table: self.ac_c_table.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decoder_creation() {
        let decoder = JpegDecoder::new();
        assert_eq!(decoder.width, 0);
        assert_eq!(decoder.height, 0);
    }
}