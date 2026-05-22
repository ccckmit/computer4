use std::io::{Read, Result};

/// 逐位读取的帮助结构体，只依赖 std::io::Read
pub struct BitReader<R: Read> {
    pub inner: R,
    pub buf: u32, // 高位对齐的位缓存
    pub left: u8, // 缓存中尚未消费的位数 (0..=32)
}

impl<R: Read> BitReader<R> {
    pub fn new(inner: R) -> Self {
        Self { inner, buf: 0, left: 0 }
    }

    /// 读取 n 位（0 < n <= 24），返回其整数值（高位在前）。
    pub fn read_bits(&mut self, n: u8) -> Result<u32> {
        assert!(n > 0 && n <= 24);
        while self.left < n {
            let mut tmp = [0u8; 1];
            let read = self.inner.read(&mut tmp)?;
            if read == 0 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    "EOF while reading bits",
                ));
            }
            self.buf = (self.buf << 8) | tmp[0] as u32;
            self.left += 8;
        }
        let shift = self.left - n;
        let val = (self.buf >> shift) & ((1u32 << n) - 1);
        self.left -= n;
        self.buf &= (1u32 << self.left) - 1;
        Ok(val)
    }

    /// 读取一个完整的 start_code（0x000001xx），返回低 8 位 xx
    pub fn read_start_code(&mut self) -> Result<u8> {
        // MPEG‑1 start code 前有 0x000001 前导
        loop {
            // 先对齐到字节边界
            while self.left % 8 != 0 {
                self.read_bits(1)?;
            }
            let mut prefix = [0u8; 3];
            self.inner.read_exact(&mut prefix)?;
            if prefix == [0x00, 0x00, 0x01] {
                let mut last = [0u8; 1];
                self.inner.read_exact(&mut last)?;
                return Ok(last[0]);
            }
        }
    }

    /// 读取一个字节（8 位），不做对齐处理
    pub fn read_u8(&mut self) -> Result<u8> {
        Ok(self.read_bits(8)? as u8)
    }

    /// 跳过 N 位（用于跳过 marker_bit 等）
    pub fn skip_bits(&mut self, mut n: u8) -> Result<()> {
        while n > 0 {
            // read_bits 最多一次读取 24 位，循环处理更大值
            let take = if n > 24 { 24 } else { n };
            self.read_bits(take)?;
            n -= take;
        }
        Ok(())
    }
}
