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
                        self.max_h = 1;
                        self.max_v = 1;
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
                        let mut values = [[0u8; 8]; 8];
                        let zigzag = zigzag_order();
                        for (i, &(zx, zy)) in zigzag.iter().enumerate() {
                            if i < 64 {
                                values[zx][zy] = data[pos + i];
                            }
                        }
                        if table_id < 4 {
                            self.qt[table_id] = QuantizationTable { values, id: table_id as u8 };
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
                0xE0..=0xEF => {
                    if pos < data.len() {
                        let length = ((data[pos] as usize) << 8) | (data[pos + 1] as usize);
                        pos += length;
                    }
                }
                _ => { pos += 1; }
            }
        }

        if self.width == 0 || self.height == 0 {
            return Err("Could not determine image dimensions".to_string());
        }

        let mcu_w = self.max_h * 8;
        let mcu_h = self.max_v * 8;
        let mcus_x = (self.width + mcu_w - 1) / mcu_w;
        let mcus_y = (self.height + mcu_h - 1) / mcu_h;

        let mut comp_data: Vec<Vec<u8>> = Vec::new();
        let mut comp_widths: Vec<usize> = Vec::new();
        for c in 0..self.num_comps {
            let cw = mcus_x * self.comp_h[c] * 8;
            let ch = mcus_y * self.comp_v[c] * 8;
            comp_data.push(vec![128u8; cw * ch]);
            comp_widths.push(cw);
        }

        let scan_data = &data[pos..];
        let mut br = BitReader::new(scan_data);
        let mut prev_dc = [0i16; 4];

        let default_dc = HuffmanTable::dc_luminance();
        let default_ac = HuffmanTable::ac_luminance();

        for mcu_y in 0..mcus_y {
            for mcu_x in 0..mcus_x {
                for comp in 0..self.num_comps {
                    let dc_tbl = self.dc_ht[self.sos_dc[comp]].as_ref().unwrap_or(&default_dc);
                    let ac_tbl = self.ac_ht[self.sos_ac[comp]].as_ref().unwrap_or(&default_ac);

                    for sy in 0..self.comp_v[comp] {
                        for sx in 0..self.comp_h[comp] {
                            let (dc, ac) = decode_coefficients(&mut br, dc_tbl, ac_tbl, prev_dc[comp]);
                            prev_dc[comp] = dc;

                            let mut block = [[0i16; 8]; 8];
                            block[0][0] = dc;
                            let zigzag = zigzag_order();
                            for (zi, &(zx, zy)) in zigzag.iter().enumerate().skip(1) {
                                if zi - 1 < ac.len() {
                                    block[zx][zy] = ac[zi - 1];
                                }
                            }

                            let qt = if self.comp_qt[comp] < 4 { &self.qt[self.comp_qt[comp]] } else { &self.qt[0] };
                            let dq = dequantize(&block, qt);
                            let idct = inverse_dct(&dq);

                            let bx = mcu_x * self.comp_h[comp] + sx;
                            let by = mcu_y * self.comp_v[comp] + sy;
                            let stride = comp_widths[comp];

                            for j in 0..8 {
                                for i in 0..8 {
                                    let px = bx * 8 + i;
                                    let py = by * 8 + j;
                                    if px < comp_widths[comp] && py < comp_data[comp].len() / comp_widths[comp] {
                                        comp_data[comp][py * stride + px] = idct[j][i].clamp(0, 255) as u8;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut y_out = vec![128u8; self.width * self.height];
        let mut cb_out = vec![128u8; self.width * self.height];
        let mut cr_out = vec![128u8; self.width * self.height];
        let comp_h = self.comp_h;
        let comp_v = self.comp_v;
        let max_h = self.max_h;
        let max_v = self.max_v;

        for comp in 0..self.num_comps.min(3) {
            let stride = comp_widths[comp];
            let src = &comp_data[comp];
            let w = stride;
            let h = src.len() / stride;
            let scale_h = max_h / comp_h[comp];
            let scale_v = max_v / comp_v[comp];

            for sy in 0..h {
                for sx in 0..w {
                    let val = src[sy * w + sx];
                    for dy in 0..scale_v {
                        for dx in 0..scale_h {
                            let px = sx * scale_h + dx;
                            let py = sy * scale_v + dy;
                            if px < self.width && py < self.height {
                                let dst_idx = py * self.width + px;
                                match comp {
                                    0 => y_out[dst_idx] = val,
                                    1 => cb_out[dst_idx] = val,
                                    _ => cr_out[dst_idx] = val,
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((y_out, cb_out, cr_out))
    }

    pub fn decode_to_rgb(&mut self, data: &[u8]) -> Result<Vec<u8>, String> {
        let (y_data, cb_data, cr_data) = self.decode(data)?;
        let mut rgb = Vec::with_capacity(self.width * self.height * 3);

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let yv = y_data[idx];
                let cb = cb_data[idx];
                let cr = cr_data[idx];
                let (r, g, b) = ycbcr_to_rgb(yv, cb, cr);
                rgb.push(r);
                rgb.push(g);
                rgb.push(b);
            }
        }

        Ok(rgb)
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
