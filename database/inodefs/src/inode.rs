use super::disk::BLOCK_SIZE;
use super::error::Result;
use std::io::{Cursor, Read, Write};

pub const INODE_SIZE: u32 = 128;
pub const DIRECT_BLOCKS: usize = 10;
pub const MAX_FILE_SIZE: u32 = DIRECT_BLOCKS as u32 * BLOCK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Fifo = 0x1,
    Chr = 0x2,
    Dir = 0x4,
    Blk = 0x6,
    Reg = 0x8,
    Lnk = 0xA,
    Sock = 0xC,
}

impl FileType {
    pub fn from_u16(v: u16) -> Option<Self> {
        match v {
            0x1 => Some(FileType::Fifo),
            0x2 => Some(FileType::Chr),
            0x4 => Some(FileType::Dir),
            0x6 => Some(FileType::Blk),
            0x8 => Some(FileType::Reg),
            0xA => Some(FileType::Lnk),
            0xC => Some(FileType::Sock),
            _ => None,
        }
    }

    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

#[derive(Debug, Clone)]
pub struct Inode {
    pub mode: u16,
    pub uid: u16,
    pub gid: u16,
    pub size: u32,
    pub atime: u32,
    pub mtime: u32,
    pub ctime: u32,
    pub links: u16,
    pub blocks: u32,
    pub direct: [u32; DIRECT_BLOCKS],
    pub indirect: u32,
    pub double_indirect: u32,
    pub padding: u32,
}

impl Default for Inode {
    fn default() -> Self {
        Self {
            mode: 0,
            uid: 0,
            gid: 0,
            size: 0,
            atime: 0,
            mtime: 0,
            ctime: 0,
            links: 0,
            blocks: 0,
            direct: [0u32; DIRECT_BLOCKS],
            indirect: 0,
            double_indirect: 0,
            padding: 0,
        }
    }
}

impl Inode {
    pub fn new(mode: u16, uid: u16, gid: u16) -> Self {
        let now = current_time();
        Self {
            mode,
            uid,
            gid,
            size: 0,
            atime: now,
            mtime: now,
            ctime: now,
            links: 1,
            blocks: 0,
            direct: [0u32; DIRECT_BLOCKS],
            indirect: 0,
            double_indirect: 0,
            padding: 0,
        }
    }

    pub fn file_type(&self) -> Option<FileType> {
        FileType::from_u16((self.mode >> 12) & 0xF)
    }

    pub fn is_dir(&self) -> bool {
        self.file_type() == Some(FileType::Dir)
    }

    pub fn is_reg(&self) -> bool {
        self.file_type() == Some(FileType::Reg)
    }

    pub fn is_lnk(&self) -> bool {
        self.file_type() == Some(FileType::Lnk)
    }

    pub fn to_bytes(&self) -> [u8; INODE_SIZE as usize] {
        let mut buf = [0u8; INODE_SIZE as usize];
        let mut cursor = Cursor::new(&mut buf);

        cursor.write_all(&self.mode.to_le_bytes()).unwrap();
        cursor.write_all(&self.uid.to_le_bytes()).unwrap();
        cursor.write_all(&self.gid.to_le_bytes()).unwrap();
        cursor.write_all(&self.size.to_le_bytes()).unwrap();
        cursor.write_all(&self.atime.to_le_bytes()).unwrap();
        cursor.write_all(&self.mtime.to_le_bytes()).unwrap();
        cursor.write_all(&self.ctime.to_le_bytes()).unwrap();
        cursor.write_all(&self.links.to_le_bytes()).unwrap();
        cursor.write_all(&self.blocks.to_le_bytes()).unwrap();
        for &d in &self.direct {
            cursor.write_all(&d.to_le_bytes()).unwrap();
        }
        cursor.write_all(&self.indirect.to_le_bytes()).unwrap();
        cursor.write_all(&self.double_indirect.to_le_bytes()).unwrap();
        cursor.write_all(&self.padding.to_le_bytes()).unwrap();

        buf
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let mut cursor = Cursor::new(data);

        let mut mode_buf = [0u8; 2];
        let mut uid_buf = [0u8; 2];
        let mut gid_buf = [0u8; 2];
        let mut size_buf = [0u8; 4];
        let mut atime_buf = [0u8; 4];
        let mut mtime_buf = [0u8; 4];
        let mut ctime_buf = [0u8; 4];
        let mut links_buf = [0u8; 2];
        let mut blocks_buf = [0u8; 4];
        let mut direct_buf = [0u8; 4 * DIRECT_BLOCKS];
        let mut indirect_buf = [0u8; 4];
        let mut double_indirect_buf = [0u8; 4];
        let mut padding_buf = [0u8; 4];

        cursor.read_exact(&mut mode_buf).unwrap();
        cursor.read_exact(&mut uid_buf).unwrap();
        cursor.read_exact(&mut gid_buf).unwrap();
        cursor.read_exact(&mut size_buf).unwrap();
        cursor.read_exact(&mut atime_buf).unwrap();
        cursor.read_exact(&mut mtime_buf).unwrap();
        cursor.read_exact(&mut ctime_buf).unwrap();
        cursor.read_exact(&mut links_buf).unwrap();
        cursor.read_exact(&mut blocks_buf).unwrap();
        cursor.read_exact(&mut direct_buf).unwrap();
        cursor.read_exact(&mut indirect_buf).unwrap();
        cursor.read_exact(&mut double_indirect_buf).unwrap();
        cursor.read_exact(&mut padding_buf).unwrap();

        let mut direct = [0u32; DIRECT_BLOCKS];
        for i in 0..DIRECT_BLOCKS {
            let offset = i * 4;
            direct[i] = u32::from_le_bytes([
                direct_buf[offset],
                direct_buf[offset + 1],
                direct_buf[offset + 2],
                direct_buf[offset + 3],
            ]);
        }

        Self {
            mode: u16::from_le_bytes(mode_buf),
            uid: u16::from_le_bytes(uid_buf),
            gid: u16::from_le_bytes(gid_buf),
            size: u32::from_le_bytes(size_buf),
            atime: u32::from_le_bytes(atime_buf),
            mtime: u32::from_le_bytes(mtime_buf),
            ctime: u32::from_le_bytes(ctime_buf),
            links: u16::from_le_bytes(links_buf),
            blocks: u32::from_le_bytes(blocks_buf),
            direct,
            indirect: u32::from_le_bytes(indirect_buf),
            double_indirect: u32::from_le_bytes(double_indirect_buf),
            padding: u32::from_le_bytes(padding_buf),
        }
    }

    pub fn allocated_blocks(&self) -> u32 {
        self.blocks
    }

    pub fn touch(&mut self) {
        let now = current_time();
        self.atime = now;
        self.mtime = now;
    }
}

fn current_time() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32
}

pub fn make_mode(file_type: FileType, permissions: u16) -> u16 {
    (file_type.to_u16() << 12) | (permissions & 0x1FF)
}

pub fn file_permissions(mode: u16) -> u16 {
    mode & 0x1FF
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inode_creation() {
        let inode = Inode::new(0o100644, 1000, 1000);

        assert_eq!(inode.uid, 1000);
        assert_eq!(inode.gid, 1000);
        assert_eq!(inode.links, 1);
        assert_eq!(inode.size, 0);
        assert!(inode.is_reg());
    }

    #[test]
    fn test_inode_serialization() {
        let mut inode = Inode::new(0o40755, 1000, 1000);
        inode.direct[0] = 100;
        inode.indirect = 200;
        inode.size = 4096;

        let bytes = inode.to_bytes();
        let inode2 = Inode::from_bytes(&bytes);

        assert_eq!(inode.mode, inode2.mode);
        assert_eq!(inode.uid, inode2.uid);
        assert_eq!(inode.gid, inode2.gid);
        assert_eq!(inode.size, inode2.size);
        assert_eq!(inode.direct, inode2.direct);
        assert_eq!(inode.indirect, inode2.indirect);
    }

    #[test]
    fn test_file_type() {
        let reg_inode = Inode::new(make_mode(FileType::Reg, 0o644), 0, 0);
        let dir_inode = Inode::new(make_mode(FileType::Dir, 0o755), 0, 0);

        assert!(reg_inode.is_reg());
        assert!(!reg_inode.is_dir());
        assert!(dir_inode.is_dir());
        assert!(!dir_inode.is_reg());
    }
}