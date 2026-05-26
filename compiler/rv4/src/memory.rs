const MEM_SIZE: usize = 257 * 1024 * 1024;

pub struct Memory {
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        Memory { data: vec![0; MEM_SIZE] }
    }

    pub fn load8(&self, addr: u64) -> Result<u8, String> {
        let a = addr as usize;
        if a >= self.data.len() { return Err(format!("load8 out of bounds: {:#x}", addr)); }
        Ok(self.data[a])
    }

    pub fn load16(&self, addr: u64) -> Result<u16, String> {
        let a = addr as usize;
        if a + 1 >= self.data.len() { return Err(format!("load16 out of bounds: {:#x}", addr)); }
        Ok(self.data[a] as u16 | (self.data[a + 1] as u16) << 8)
    }

    pub fn load32(&self, addr: u64) -> Result<u32, String> {
        let a = addr as usize;
        if a + 3 >= self.data.len() { return Err(format!("load32 out of bounds: {:#x}", addr)); }
        Ok(self.data[a] as u32
            | (self.data[a + 1] as u32) << 8
            | (self.data[a + 2] as u32) << 16
            | (self.data[a + 3] as u32) << 24)
    }

    pub fn load64(&self, addr: u64) -> Result<u64, String> {
        let a = addr as usize;
        if a + 7 >= self.data.len() { return Err(format!("load64 out of bounds: {:#x}", addr)); }
        Ok(self.data[a] as u64
            | (self.data[a + 1] as u64) << 8
            | (self.data[a + 2] as u64) << 16
            | (self.data[a + 3] as u64) << 24
            | (self.data[a + 4] as u64) << 32
            | (self.data[a + 5] as u64) << 40
            | (self.data[a + 6] as u64) << 48
            | (self.data[a + 7] as u64) << 56)
    }

    pub fn store8(&mut self, addr: u64, val: u8) -> Result<(), String> {
        let a = addr as usize;
        if a >= self.data.len() { return Err(format!("store8 out of bounds: {:#x}", addr)); }
        self.data[a] = val;
        Ok(())
    }

    pub fn store16(&mut self, addr: u64, val: u16) -> Result<(), String> {
        let a = addr as usize;
        if a + 1 >= self.data.len() { return Err(format!("store16 out of bounds: {:#x}", addr)); }
        self.data[a] = val as u8;
        self.data[a + 1] = (val >> 8) as u8;
        Ok(())
    }

    pub fn store32(&mut self, addr: u64, val: u32) -> Result<(), String> {
        let a = addr as usize;
        if a + 3 >= self.data.len() { return Err(format!("store32 out of bounds: {:#x}", addr)); }
        self.data[a] = val as u8;
        self.data[a + 1] = (val >> 8) as u8;
        self.data[a + 2] = (val >> 16) as u8;
        self.data[a + 3] = (val >> 24) as u8;
        Ok(())
    }

    pub fn store64(&mut self, addr: u64, val: u64) -> Result<(), String> {
        let a = addr as usize;
        if a + 7 >= self.data.len() { return Err(format!("store64 out of bounds: {:#x}", addr)); }
        self.data[a] = val as u8;
        self.data[a + 1] = (val >> 8) as u8;
        self.data[a + 2] = (val >> 16) as u8;
        self.data[a + 3] = (val >> 24) as u8;
        self.data[a + 4] = (val >> 32) as u8;
        self.data[a + 5] = (val >> 40) as u8;
        self.data[a + 6] = (val >> 48) as u8;
        self.data[a + 7] = (val >> 56) as u8;
        Ok(())
    }

    pub fn write_bytes(&mut self, addr: u64, data: &[u8]) -> Result<(), String> {
        let a = addr as usize;
        if a + data.len() > self.data.len() {
            return Err(format!("write_bytes out of bounds: {:#x} size {}", addr, data.len()));
        }
        self.data[a..a + data.len()].copy_from_slice(data);
        Ok(())
    }
}
