use super::{BitReader, HuffmanTable, QuantizationTable, inverse_dct, dequantize, zigzag_order, decode_coefficients, ycbcr_to_rgb};

pub struct JpegDecoder {
    width: usize,
    height: usize,
    comp_h: [usize; 3],
    comp_v: [usize; 3],
    comp_qt: [usize; 3],
    max_h: usize,
    max_v: usize,
    num_comps: usize,
    qt: [QuantizationTable; 4],
    dc_ht: [Option<HuffmanTable>; 4],
    ac_ht: [Option<HuffmanTable>; 4],
    sos_dc: [usize; 3],
    sos_ac: [usize; 3],
}

impl JpegDecoder {
    pub fn new() -> Self {
        let dc_y = HuffmanTable::dc_luminance();
        let dc_c = HuffmanTable::dc_chrominance();
        let ac_y = HuffmanTable::ac_luminance();
        let ac_c = HuffmanTable::ac_chrominance();
        Self {
            width: 0,
            height: 0,
            comp_h: [1; 3],
            comp_v: [1; 3],
            comp_qt: [0; 3],
            max_h: 1,
            max_v: 1,
            num_comps: 0,
            qt: [QuantizationTable::luminance(), QuantizationTable::chrominance(), QuantizationTable::luminance(), QuantizationTable::luminance()],
            dc_ht: [Some(dc_y), Some(dc_c), None, None],
            ac_ht: [Some(ac_y), Some(ac_c), None, None],
            sos_dc: [0; 3],
            sos_ac: [0; 3],
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
                        self.num_comps = data[pos + 7] as usize;
                        if self.num_comps > 3 {
                            self.num_comps = 3;
                        }
                        for i in 0..self.num_comps {
                            let off = pos + 8 + i * 3;
                            if off + 2 < data.len() {
                                let _cid = data[off];
                                let samp = data[off + 1];
                                self.comp_h[i] = (samp >> 4) as usize;
                                self.comp_v[i] = (samp & 0x0F) as usize;
                                self.comp_qt[i] = data[off + 2] as usize;
                                if self.comp_h[i] > self.max_h { self.max_h = self.comp_h[i]; }
                                if self.comp_v[i] > self.max_v { self.max_v = self.comp_v[i]; }
                            }
                        }
                    }
                    pos += length;
                }
                0xDB => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    pos += 2;
                    let end = pos + length - 2;
                    while pos + 64 < end && pos + 64 < data.len() {
                        let info = data[pos];
                        let table_id = (info & 0x0F) as usize;
                        pos += 1;
                        let qt = self.read_quantization_table(&data[pos..pos + 64], table_id);
                        if table_id < 4 {
                            self.qt[table_id] = qt;
                        }
                        pos += 64;
                    }
                    pos = end;
                }
                0xC4 => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    pos += 2;
                    let end = pos + length - 2;
                    while pos < end && pos < data.len() {
                        let info = data[pos];
                        let tc = (info >> 4) as usize;
                        let id = (info & 0x0F) as usize;
                        pos += 1;
                        let mut bits = [0u8; 16];
                        let mut nvals = 0;
                        for b in &mut bits {
                            if pos < data.len() {
                                *b = data[pos];
                                nvals += *b as usize;
                                pos += 1;
                            }
                        }
                        let mut vals = Vec::with_capacity(nvals);
                        for _ in 0..nvals {
                            if pos < data.len() {
                                vals.push(data[pos]);
                                pos += 1;
                            }
                        }
                        let mut ht = HuffmanTable {
                            table_type: tc as u8,
                            table_id: id as u8,
                            codes: std::collections::HashMap::new(),
                            lookup: std::collections::HashMap::new(),
                        };
                        let mut code = 0u16;
                        let mut sym_idx = 0;
                        for (len_idx, &count) in bits.iter().enumerate() {
                            let len = len_idx + 1;
                            for _ in 0..count {
                                if sym_idx < vals.len() {
                                    let symbol = vals[sym_idx];
                                    ht.codes.insert(symbol, (code, len));
                                    ht.lookup.insert((code, len), symbol);
                                    code += 1;
                                    sym_idx += 1;
                                }
                            }
                            code <<= 1;
                        }
                        if tc == 0 && id < 4 {
                            self.dc_ht[id] = Some(ht);
                        } else if tc == 1 && id < 4 {
                            self.ac_ht[id] = Some(ht);
                        }
                    }
                    pos = end;
                }
                0xDA => {
                    let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                    let n = data[pos + 2] as usize;
                    let mut off = pos + 3;
                    for i in 0..n.min(3) {
                        if off + 1 < data.len() {
                            let _cid = data[off];
                            let tbl = data[off + 1];
                            self.sos_dc[i] = (tbl >> 4) as usize;
                            self.sos_ac[i] = (tbl & 0x0F) as usize;
                            off += 2;
                        }
                    }
                    pos += length;
                    break;
                }
                0xD9 => { break; }
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
        let cw = (self.width + self.max_h - 1) / self.max_h;
        let ch = (self.height + self.max_v - 1) / self.max_v;
        let c_stride = cw * ch;
        let mut y_data = vec![0u8; y_size];
        let mut cb_data = vec![128u8; c_stride];
        let mut cr_data = vec![128u8; c_stride];

        let blocks_x = (self.width + 7) / 8;
        let blocks_y = (self.height + 7) / 8;
        let num_blocks = blocks_x * blocks_y;

        let mut bit_reader = BitReader::new(&data[pos..]);

        let mut prev_dc = [0i16; 3];

        let dc_tables: Vec<&HuffmanTable> = (0..3)
            .map(|i| {
                let idx = self.sos_dc[i];
                self.dc_ht[idx].as_ref().unwrap_or(&HuffmanTable::dc_luminance())
            })
            .collect();
        let ac_tables: Vec<&HuffmanTable> = (0..3)
            .map(|i| {
                let idx = self.sos_ac[i];
                self.ac_ht[idx].as_ref().unwrap_or(&HuffmanTable::ac_luminance())
            })
            .collect();

        for block_idx in 0..num_blocks {
            let by = block_idx / blocks_x;
            let bx = block_idx % blocks_x;

            for comp in 0..self.num_comps.min(3) {
                let (dc, ac) = decode_coefficients(&mut bit_reader, dc_tables[comp], ac_tables[comp], prev_dc[comp]);
                prev_dc[comp] = dc;

                let mut block = [[0i16; 8]; 8];
                block[0][0] = dc;

                let zigzag = zigzag_order();
                for (i, &(zx, zy)) in zigzag.iter().enumerate().skip(1) {
                    if i - 1 < ac.len() {
                        block[zx][zy] = ac[i - 1];
                    }
                }

                let qt_id = if comp < 3 { self.comp_qt[comp] } else { 0 };
                let qt = if qt_id < 4 { &self.qt[qt_id] } else { &self.qt[0] };
                let dq_block = dequantize(&block, qt);
                let idct_block = inverse_dct(&dq_block);

                let comp_w = self.comp_h[comp];
                let comp_v = self.comp_v[comp];
                let mcu_w = self.max_h * 8;
                let mcu_h = self.max_v * 8;
                let mcux = bx;
                let mcuy = by;

                for j in 0..8 {
                    for i in 0..8 {
                        let comp_x = (mcux * mcu_w + i * self.max_h / comp_w) * comp_w / self.max_h;
                        let comp_y = (mcuy * mcu_h + j * self.max_v / comp_v) * comp_v / self.max_v;
                        let px = mcux * mcu_w + i * self.max_h / comp_w;
                        let py = mcuy * mcu_h + j * self.max_v / comp_v;

                        if px < self.width && py < self.height {
                            let val = idct_block[j][i].clamp(0, 255) as u8;
                            if comp == 0 {
                                y_data[py * self.width + px] = val;
                            } else if comp == 1 {
                                let cidx = (py / self.max_v) * cw + (px / self.max_h);
                                if cidx < cb_data.len() {
                                    cb_data[cidx] = val;
                                }
                            } else if comp == 2 {
                                let cidx = (py / self.max_v) * cw + (px / self.max_h);
                                if cidx < cr_data.len() {
                                    cr_data[cidx] = val;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((y_data, cb_data, cr_data))
    }

    pub fn decode_to_rgb(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        let (y_data, cb_data, cr_data) = self.decode(data)?;

        let cw = (self.width + self.max_h - 1) / self.max_h;
        let mut rgb = Vec::with_capacity(self.width * self.height * 3);

        for y in 0..self.height {
            for x in 0..self.width {
                let y_idx = y * self.width + x;
                let cb_idx = (y / self.max_v) * cw + (x / self.max_h);

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
            comp_h: self.comp_h,
            comp_v: self.comp_v,
            comp_qt: self.comp_qt,
            max_h: self.max_h,
            max_v: self.max_v,
            num_comps: self.num_comps,
            qt: Default::default(),
            dc_ht: [None, None, None, None],
            ac_ht: [None, None, None, None],
            sos_dc: self.sos_dc,
            sos_ac: self.sos_ac,
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