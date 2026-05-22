use crate::bitstream::BitReader;
use crate::parser::{SequenceHeader, PictureHeader, SliceHeader, MacroBlock, PictureCodingType};
use crate::frame::YuvFrame;
use crate::idct::idct_8x8;
use crate::motion::motion_compensate;
use std::io::Read;
use std::path::PathBuf;

pub struct Decoder<R: Read> {
    pub br: BitReader<R>,
    pub width: usize,
    pub height: usize,
    forward_ref: Option<YuvFrame>,
    backward_ref: Option<YuvFrame>,
    cur_frame: YuvFrame,
    picture_idx: usize,
    target_idx: Option<usize>,
    target_path: Option<PathBuf>,
}

impl<R: Read> Decoder<R> {
    pub fn new(mut src: R) -> std::io::Result<Self> {
        let mut br = BitReader::new(src);
        // 读取 Sequence Header 前的 start_code（0xB3）
        let _seq_start = br.read_start_code()?; // Expect 0xB3
        let seq = SequenceHeader::parse(&mut br)?;
        let width = seq.horizontal_size as usize;
        let height = seq.vertical_size as usize;
        Ok(Self {
            br,
            width,
            height,
            forward_ref: None,
            backward_ref: None,
            cur_frame: YuvFrame::new(width, height),
            picture_idx: 0,
            target_idx: None,
            target_path: None,
        })
    }

    /// 设置想要提取的帧号以及保存路径
    pub fn set_target(&mut self, idx: usize, path: PathBuf) {
        self.target_idx = Some(idx);
        self.target_path = Some(path);
    }

    /// 主循环：读取码流并解码每个 picture
    pub fn run(mut self) -> std::io::Result<()> {
        loop {
            let start_code = match self.br.read_start_code() {
                Ok(sc) => sc,
                Err(_) => break, // EOF
            };
            match start_code {
                0x00 => {
                    // picture_start_code
                    let pic_hdr = PictureHeader::parse(&mut self.br)?;
                    self.decode_picture(&pic_hdr)?;
                    // 检查是否需要保存当前帧
                    if let Some(target) = self.target_idx {
                        if self.picture_idx == target {
                            if let Some(p) = &self.target_path {
                                self.cur_frame.save_as_ppm(p)?;
                                println!("Saved picture {} to {}", target, p.display());
                                return Ok(());
                            }
                        }
                    }
                    self.picture_idx += 1;
                }
                0xB3 => {
                    // user_data_start_code，直接忽略
                }
                _ => {
                    // 其它 start_code（slice、group_of_pictures 等）在 picture 解码内部处理
                }
            }
        }
        Ok(())
    }

    fn decode_picture(&mut self, pic_hdr: &PictureHeader) -> std::io::Result<()> {
        match pic_hdr.coding_type {
            PictureCodingType::I => self.decode_i_picture(),
            PictureCodingType::P => self.decode_p_picture(),
            PictureCodingType::B => self.decode_b_picture(),
        }
    }

    fn decode_i_picture(&mut self) -> std::io::Result<()> {
        self.decode_macroblocks(true, None)
    }

    fn decode_p_picture(&mut self) -> std::io::Result<()> {
        let ref_frame = self.forward_ref.clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing forward reference for P picture",
            )
        })?;
        self.decode_macroblocks(false, Some(&ref_frame))
    }

    fn decode_b_picture(&mut self) -> std::io::Result<()> {
        let fwd = self.forward_ref.clone().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Missing forward reference for B picture",
            )
        })?;
        // Backward reference not used in this simplified implementation
        self.decode_macroblocks(false, Some(&fwd))
    }

    /// `intra` 为 true 表示所有宏块都是 intra（I‑frame），否则使用参考帧进行运动补偿。
    fn decode_macroblocks(
        &mut self,
        intra: bool,
        ref_frame: Option<&YuvFrame>,
    ) -> std::io::Result<()> {
        loop {
            // 读取下一个 start_code；如果是 picture_start_code（0x00），说明本 picture 已结束。
            let next = match self.br.read_start_code() {
                Ok(sc) => sc,
                Err(_) => break,
            };
            if next == 0x00 {
                // 把 start_code 回退，让外层再次处理该 picture_start_code
                self.br.left += 24;
                self.br.buf = (self.br.buf << 24) | 0x000001u32;
                break;
            }
            // Slice Header（这里只读取 vertical_position）
            let _slice = SliceHeader::parse(&mut self.br, next)?;
            // 循环读取宏块，直到遇到下一个 start_code（在外层循环里捕获）
            while let Ok(mb) = MacroBlock::parse(&mut self.br, intra) {
                // 1. 逆量化（直接乘以 quantiser）
                let mut dequant = [0i16; 64];
                for i in 0..64 {
                    dequant[i] = mb.coeffs[i] * mb.quantiser as i16;
                }
                // 2. 逆 DCT
                let mut block = [0u8; 64];
                idct_8x8(&dequant, &mut block);
                // 3. 写块到当前帧（这里仅处理 Y 分量）
                self.put_block(&mb, &block);
                // 4. 如为 P/B 并有参考帧，进行运动补偿（仅 Y 分量示例）
                if !intra {
                    if let Some(rf) = ref_frame {
                        motion_compensate(
                            rf,
                            &mut self.cur_frame.y,
                            mb.mb_x,
                            mb.mb_y,
                            mb.mv_x,
                            mb.mv_y,
                            mb.full_pel,
                        );
                    }
                }
            }
        }
        // 本 picture 完成后，更新参考帧缓存
        match self.cur_frame.clone() {
            frame => {
                // 对 I/P 帧，当前帧成为下一帧的前向参考；对 B‑frame 本示例保持不变
                self.forward_ref = Some(frame.clone());
                // 为后续帧准备空帧缓冲区
                self.cur_frame = YuvFrame::new(self.width, self.height);
            }
        }
        Ok(())
    }

    /// 把解码得到的 8×8 块写入 YUV 帧（这里只写到 Y 分量；U/V 按 4×4 子块类似处理）
    fn put_block(&mut self, mb: &MacroBlock, block: &[u8; 64]) {
        // 示例：宏块左上角坐标（以 16 像素步长）
        let base_x = mb.mb_x * 16;
        let base_y = mb.mb_y * 16;
        for dy in 0..8 {
            for dx in 0..8 {
                let src = block[dy * 8 + dx];
                let dst_x = base_x + dx;
                let dst_y = base_y + dy;
                if dst_x < self.width && dst_y < self.height {
                    self.cur_frame.y[dst_y * self.width + dst_x] = src;
                }
            }
        }
        // UV 组件写入略（可按 8×8 中的 4×4 子块填充）
    }
}
