use super::{BitWriter, HuffmanTable, QuantizationTable, forward_dct, quantize, zigzag_order};

pub struct JpegEncoder {
    width: usize,
    height: usize,
    y_quant: QuantizationTable,
    c_quant: QuantizationTable,
    dc_y_table: HuffmanTable,
    dc_c_table: HuffmanTable,
    ac_y_table: HuffmanTable,
    ac_c_table: HuffmanTable,
}

const MAX_H: usize = 2;
const MAX_V: usize = 2;

impl JpegEncoder {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            y_quant: QuantizationTable::luminance(),
            c_quant: QuantizationTable::chrominance(),
            dc_y_table: HuffmanTable::dc_luminance(),
            dc_c_table: HuffmanTable::dc_chrominance(),
            ac_y_table: HuffmanTable::ac_luminance(),
            ac_c_table: HuffmanTable::ac_chrominance(),
        }
    }

    pub fn encode(&self, y_data: &[u8], cb_data: &[u8], cr_data: &[u8]) -> Vec<u8> {
        let mut output = Vec::new();

        output.push(0xFF);
        output.push(0xD8);

        self.write_quantization_table(&mut output, &self.y_quant);
        self.write_quantization_table(&mut output, &self.c_quant);

        self.write_huffman_table(&mut output, &self.dc_y_table, 0, 0);
        self.write_huffman_table(&mut output, &self.dc_c_table, 0, 1);
        self.write_huffman_table(&mut output, &self.ac_y_table, 1, 0);
        self.write_huffman_table(&mut output, &self.ac_c_table, 1, 1);

        self.write_start_of_frame(&mut output);

        self.write_start_of_scan(&mut output);

        let mcu_w = MAX_H * 8;
        let mcu_h = MAX_V * 8;
        let mcus_x = (self.width + mcu_w - 1) / mcu_w;
        let mcus_y = (self.height + mcu_h - 1) / mcu_h;

        let cw = mcus_x * 8;
        let _ch = mcus_y * 8;

        let mut prev_dc = [0i16; 3];

        for mcu_y in 0..mcus_y {
            for mcu_x in 0..mcus_x {
                for sy in 0..MAX_V {
                    for sx in 0..MAX_H {
                        let bx = mcu_x * MAX_H + sx;
                        let by = mcu_y * MAX_V + sy;
                        let mut block = [[0i16; 8]; 8];
                        for j in 0..8 {
                            for i in 0..8 {
                                let x = bx * 8 + i;
                                let y = by * 8 + j;
                                block[j][i] = if y < self.height && x < self.width {
                                    (y_data[y * self.width + x] as i16) - 128
                                } else {
                                    0
                                };
                            }
                        }
                        let dct = forward_dct(&block);
                        let q = quantize(&dct, &self.y_quant);
                        self.encode_block(&mut output, &q, &self.dc_y_table, &self.ac_y_table, &mut prev_dc[0]);
                    }
                }

                let mut cb_block = [[0i16; 8]; 8];
                let mut cr_block = [[0i16; 8]; 8];
                for j in 0..8 {
                    for i in 0..8 {
                        let x = mcu_x * 8 + i;
                        let y = mcu_y * 8 + j;
                        let idx = y * cw + x;
                        cb_block[j][i] = if idx < cb_data.len() {
                            (cb_data[idx] as i16) - 128
                        } else {
                            0
                        };
                        cr_block[j][i] = if idx < cr_data.len() {
                            (cr_data[idx] as i16) - 128
                        } else {
                            0
                        };
                    }
                }
                let cb_dct = forward_dct(&cb_block);
                let cr_dct = forward_dct(&cr_block);
                let cb_q = quantize(&cb_dct, &self.c_quant);
                let cr_q = quantize(&cr_dct, &self.c_quant);
                self.encode_block(&mut output, &cb_q, &self.dc_c_table, &self.ac_c_table, &mut prev_dc[1]);
                self.encode_block(&mut output, &cr_q, &self.dc_c_table, &self.ac_c_table, &mut prev_dc[2]);
            }
        }

        output.push(0xFF);
        output.push(0xD9);

        output
    }

    fn encode_block(&self, output: &mut Vec<u8>, block: &[[i16; 8]; 8], dc_table: &HuffmanTable, ac_table: &HuffmanTable, prev_dc: &mut i16) {
        let zigzag = zigzag_order();
        let dc_val = block[0][0];
        let mut ac_vals = Vec::new();
        for &(i, j) in zigzag.iter().skip(1) {
            ac_vals.push(block[i][j]);
        }

        let dc_diff = dc_val - *prev_dc;
        *prev_dc = dc_val;

        let dc_category = if dc_diff == 0 {
            0
        } else {
            (dc_diff.abs().ilog2() + 1) as u8
        };

        let mut writer = BitWriter::new();

        if let Some(&(code, len)) = dc_table.codes.get(&dc_category) {
            writer.write_bits(code, len as u8);
        }

        if dc_category > 0 {
            let bits = if dc_diff >= 0 { dc_diff as u16 } else { (1u16 << dc_category) - (dc_diff + 1).unsigned_abs() as u16 };
            writer.write_bits(bits, dc_category as u8);
        }

        let mut zero_count = 0;
        for &coef in &ac_vals {
            if coef == 0 {
                zero_count += 1;
            } else {
                while zero_count >= 16 {
                    if let Some(&(code, len)) = ac_table.codes.get(&0xF0) {
                        writer.write_bits(code, len as u8);
                    }
                    zero_count -= 16;
                }
                let category = if coef >= 0 {
                    (coef.ilog2() + 1) as u8
                } else {
                    ((-coef).ilog2() + 1) as u8
                };
                let rs = ((zero_count as u8) << 4) | category;
                if let Some(&(code, len)) = ac_table.codes.get(&rs) {
                    writer.write_bits(code, len as u8);
                }
                let bits = if coef >= 0 { coef as u16 } else { (1u16 << category) - ((coef + 1) as i32).unsigned_abs() as u16 };
                writer.write_bits(bits, category as u8);
                zero_count = 0;
            }
        }

        if zero_count > 0 || ac_vals.iter().all(|&x| x == 0) {
            if let Some(&(code, len)) = ac_table.codes.get(&0x00) {
                writer.write_bits(code, len as u8);
            }
        }

        let bytes = writer.finalize();
        output.extend_from_slice(&bytes);
    }

    fn write_quantization_table(&self, output: &mut Vec<u8>, table: &QuantizationTable) {
        output.push(0xFF);
        output.push(0xDB);
        output.push(0x00);
        output.push(0x43);
        output.push(table.id | 0x00);

        let zigzag = zigzag_order();
        for &(i, j) in &zigzag {
            output.push(table.values[i][j]);
        }
    }

    fn write_huffman_table(&self, output: &mut Vec<u8>, table: &HuffmanTable, table_type: u8, table_id: u8) {
        let mut counts = [0u8; 16];
        let mut symbols: Vec<u8> = Vec::new();

        for len in 1..=16 {
            let mut count_at_len = 0;
            for (symbol, (_, l)) in &table.codes {
                if *l == len {
                    count_at_len += 1;
                    symbols.push(*symbol);
                }
            }
            counts[len - 1] = count_at_len;
        }

        let total_symbols = symbols.len();
        let length = 2 + 1 + 16 + total_symbols;

        output.push(0xFF);
        output.push(0xC4);
        output.push((length >> 8) as u8);
        output.push((length & 0xFF) as u8);
        output.push((table_type << 4) | table_id);
        for &c in &counts {
            output.push(c);
        }
        output.extend_from_slice(&symbols);
    }

    fn write_start_of_frame(&self, output: &mut Vec<u8>) {
        output.push(0xFF);
        output.push(0xC0);
        output.push(0x00);
        output.push(0x11);
        output.push(0x08);
        output.push((self.height >> 8) as u8);
        output.push((self.height & 0xFF) as u8);
        output.push((self.width >> 8) as u8);
        output.push((self.width & 0xFF) as u8);
        output.push(0x03);
        output.push(0x01);
        output.push(0x22);
        output.push(0x00);
        output.push(0x02);
        output.push(0x11);
        output.push(0x01);
        output.push(0x03);
        output.push(0x11);
        output.push(0x01);
    }

    fn write_start_of_scan(&self, output: &mut Vec<u8>) {
        output.push(0xFF);
        output.push(0xDA);
        output.push(0x00);
        output.push(0x0C);
        output.push(0x03);
        output.push(0x01);
        output.push(0x00);
        output.push(0x02);
        output.push(0x11);
        output.push(0x03);
        output.push(0x11);
        output.push(0x00);
        output.push(0x3F);
        output.push(0x00);
    }
}

impl Default for JpegEncoder {
    fn default() -> Self {
        Self::new(8, 8)
    }
}
