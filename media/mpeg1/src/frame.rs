use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

/// YUV420 帧结构，仅用于解码后在内存保存。
#[derive(Clone)]
pub struct YuvFrame {
    pub width: usize,
    pub height: usize,
    pub y: Vec<u8>, // luma plane (W*H)
    pub u: Vec<u8>, // chroma U plane (W/2 * H/2)
    pub v: Vec<u8>, // chroma V plane (W/2 * H/2)
}

impl YuvFrame {
    pub fn new(width: usize, height: usize) -> Self {
        let y = vec![0u8; width * height];
        let uv_w = width / 2;
        let uv_h = height / 2;
        let u = vec![0u8; uv_w * uv_h];
        let v = vec![0u8; uv_w * uv_h];
        Self { width, height, y, u, v }
    }

    /// 将 YUV420 转为 RGB24（行主序），返回 Vec<u8>（长度 = w*h*3）
    pub fn to_rgb24(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.width * self.height * 3);
        for j in 0..self.height {
            for i in 0..self.width {
                let y = self.y[j * self.width + i] as i32;
                let u = self.u[(j / 2) * (self.width / 2) + (i / 2)] as i32 - 128;
                let v = self.v[(j / 2) * (self.width / 2) + (i / 2)] as i32 - 128;

                // BT.601 转换（整数近似）
                let c = y - 16;
                let d = u;
                let e = v;
                let r = (298 * c + 409 * e + 128) >> 8;
                let g = (298 * c - 100 * d - 208 * e + 128) >> 8;
                let b = (298 * c + 516 * d + 128) >> 8;

                out.push(r.clamp(0, 255) as u8);
                out.push(g.clamp(0, 255) as u8);
                out.push(b.clamp(0, 255) as u8);
            }
        }
        out
    }

    /// 保存为 PPM（P6）文件
    pub fn save_as_ppm(&self, path: &Path) -> std::io::Result<()> {
        let rgb = self.to_rgb24();
        let mut w = BufWriter::new(File::create(path)?);
        write!(w, "P6\n{} {}\n255\n", self.width, self.height)?;
        w.write_all(&rgb)?;
        w.flush()
    }
}
