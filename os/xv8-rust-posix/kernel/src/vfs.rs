use core::slice;
use core::str;

use crate::abi::Errno;
use crate::fs::*;
use crate::param::MAXPATH;
use crate::proc;
use crate::spinlock::SpinLock;
use crate::vm::VA;

/// Maximum number of concurrent mounts
pub const MAX_MOUNTS: usize = 8;

/// A named mount point
pub struct Mount {
    pub prefix: [u8; MAXPATH],
    pub prefix_len: usize,
    pub fs: &'static dyn VfsOps,
    pub root_id: u64,
}

/// Global mount table
pub static MOUNTS: SpinLock<[Option<Mount>; MAX_MOUNTS]> =
    SpinLock::new([None, None, None, None, None, None, None, None], "mounts");

/// VFS operations for a filesystem type
pub trait VfsOps: Send + Sync {
    fn name(&self) -> &str;
    fn lookup(&self, parent: u64, name: &str) -> Result<(u64, InodeType, Option<&'static dyn VfsOps>), Errno>;
    fn read(&self, id: u64, offset: u32, dst: &mut [u8]) -> Result<u32, Errno>;
    fn write(&self, id: u64, _offset: u32, src: &[u8]) -> Result<u32, Errno>;
    fn readdir(&self, id: u64, cb: &mut dyn FnMut(u64, &str, InodeType)) -> Result<(), Errno>;
    fn stat(&self, id: u64) -> Stat;
}

/// Register a mount point.
/// Takes ownership of a SpinLockGuard on the mount table.
pub fn mount(
    prefix: &str,
    fs: &'static dyn VfsOps,
    root_id: u64,
) -> Result<usize, Errno> {
    let mut mounts = MOUNTS.lock();
    for slot in mounts.iter_mut() {
        if slot.is_none() {
            let mut buf = [0u8; MAXPATH];
            let bytes = prefix.as_bytes();
            let len = bytes.len().min(MAXPATH - 1);
            buf[..len].copy_from_slice(&bytes[..len]);
            buf[len] = 0;
            *slot = Some(Mount {
                prefix: buf,
                prefix_len: len,
                fs,
                root_id,
            });
            return Ok(0);
        }
    }
    Err(Errno::ENOMEM)
}

/// Unregister a mount point by prefix.
pub fn umount(prefix: &str) -> Result<usize, Errno> {
    let mut mounts = MOUNTS.lock();
    for slot in mounts.iter_mut() {
        if let Some(m) = slot {
            let m_prefix = core::str::from_utf8(&m.prefix[..m.prefix_len]).unwrap_or("");
            if m_prefix == prefix {
                *slot = None;
                return Ok(0);
            }
        }
    }
    Err(Errno::EINVAL)
}

/// Find a mount that matches the given path prefix.
/// Returns (fs, root_id) if found.
pub fn find_mount(path: &str) -> Option<(&'static dyn VfsOps, u64)> {
    let mounts = MOUNTS.lock();
    for slot in mounts.iter() {
        if let Some(m) = slot {
            let m_prefix = core::str::from_utf8(&m.prefix[..m.prefix_len]).unwrap_or("");
            if path == m_prefix {
                return Some((m.fs, m.root_id));
            }
        }
    }
    None
}

/// Check if the accumulated path (in the native FS) matches a mount point.
/// `resolved_components` is the path assembled so far during resolution.
/// Returns `Some((fs, root_id))` if the component triggers a mount switch.
pub fn check_mount(resolved: &str) -> Option<(&'static dyn VfsOps, u64)> {
    let mounts = MOUNTS.lock();
    for slot in mounts.iter() {
        if let Some(m) = slot {
            let m_prefix = core::str::from_utf8(&m.prefix[..m.prefix_len]).unwrap_or("");
            if resolved == m_prefix {
                return Some((m.fs, m.root_id));
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// ProcFs — minimal /proc implementation
// ---------------------------------------------------------------------------

pub struct ProcFs;

const PROC_ROOT_ID: u64 = 1;
const PROC_SELF_ID: u64 = 2;
const PROC_SELF_STATUS_ID: u64 = 3;

impl ProcFs {
    fn status_content() -> &'static [u8] {
        let pid = proc::current_pid().as_usize();
        // Static buffer — one page worth
        static mut BUF: [u8; 512] = [0; 512];
        let s = format!(
            "Name:\tprocfs\nPid:\t{}\nUid:\t0\nGid:\t0\n",
            pid,
        );
        let bytes = s.as_bytes();
        let len = bytes.len().min(512);
        unsafe {
            BUF[..len].copy_from_slice(&bytes[..len]);
            &BUF[..len]
        }
    }
}

impl VfsOps for ProcFs {
    fn name(&self) -> &str {
        "procfs"
    }

    fn lookup(&self, parent: u64, name: &str) -> Result<(u64, InodeType, Option<&'static dyn VfsOps>), Errno> {
        match (parent, name) {
            (PROC_ROOT_ID, "self") => Ok((PROC_SELF_ID, InodeType::Directory, None)),
            (PROC_SELF_ID, "status") => Ok((PROC_SELF_STATUS_ID, InodeType::File, None)),
            _ => Err(Errno::ENOENT),
        }
    }

    fn read(&self, id: u64, _offset: u32, dst: &mut [u8]) -> Result<u32, Errno> {
        if id == PROC_SELF_STATUS_ID {
            let content = Self::status_content();
            let to_copy = content.len().min(dst.len());
            dst[..to_copy].copy_from_slice(&content[..to_copy]);
            Ok(to_copy as u32)
        } else {
            Err(Errno::EINVAL)
        }
    }

    fn write(&self, _id: u64, _offset: u32, _src: &[u8]) -> Result<u32, Errno> {
        Err(Errno::EROFS)
    }

    fn readdir(&self, id: u64, cb: &mut dyn FnMut(u64, &str, InodeType)) -> Result<(), Errno> {
        if id == PROC_ROOT_ID {
            (cb)(PROC_SELF_ID, "self", InodeType::Directory);
            Ok(())
        } else if id == PROC_SELF_ID {
            (cb)(PROC_SELF_STATUS_ID, "status", InodeType::File);
            Ok(())
        } else {
            Err(Errno::ENOTDIR)
        }
    }

    fn stat(&self, id: u64) -> Stat {
        match id {
            PROC_ROOT_ID => Stat {
                r#type: InodeType::Directory,
                mode: crate::fs::mode::S_IFDIR | 0o555,
                nlink: 1,
                ..Stat::default()
            },
            PROC_SELF_ID => Stat {
                r#type: InodeType::Directory,
                mode: crate::fs::mode::S_IFDIR | 0o555,
                nlink: 1,
                ..Stat::default()
            },
            PROC_SELF_STATUS_ID => Stat {
                r#type: InodeType::File,
                size: 256,
                mode: crate::fs::mode::S_IFREG | 0o444,
                nlink: 1,
                ..Stat::default()
            },
            _ => Stat::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// DevFs — minimal /dev implementation
// ---------------------------------------------------------------------------

pub struct DevFs;

const DEV_ROOT_ID: u64 = 1;
const DEV_NULL_ID: u64 = 2;
const DEV_ZERO_ID: u64 = 3;

impl VfsOps for DevFs {
    fn name(&self) -> &str {
        "devfs"
    }

    fn lookup(&self, parent: u64, name: &str) -> Result<(u64, InodeType, Option<&'static dyn VfsOps>), Errno> {
        match (parent, name) {
            (DEV_ROOT_ID, "null") => Ok((DEV_NULL_ID, InodeType::Device, None)),
            (DEV_ROOT_ID, "zero") => Ok((DEV_ZERO_ID, InodeType::Device, None)),
            _ => Err(Errno::ENOENT),
        }
    }

    fn read(&self, id: u64, _offset: u32, _dst: &mut [u8]) -> Result<u32, Errno> {
        match id {
            DEV_NULL_ID => Ok(0),
            DEV_ZERO_ID => {
                // Reading from /dev/zero fills with zeros
                // But this is a simplified version - just return 0
                Ok(0)
            }
            _ => Err(Errno::EINVAL),
        }
    }

    fn write(&self, _id: u64, _offset: u32, src: &[u8]) -> Result<u32, Errno> {
        Ok(src.len() as u32)
    }

    fn readdir(&self, id: u64, cb: &mut dyn FnMut(u64, &str, InodeType)) -> Result<(), Errno> {
        if id == DEV_ROOT_ID {
            (cb)(DEV_NULL_ID, "null", InodeType::Device);
            (cb)(DEV_ZERO_ID, "zero", InodeType::Device);
            Ok(())
        } else {
            Err(Errno::ENOTDIR)
        }
    }

    fn stat(&self, id: u64) -> Stat {
        match id {
            DEV_ROOT_ID => Stat {
                r#type: InodeType::Directory,
                mode: crate::fs::mode::S_IFDIR | 0o555,
                nlink: 1,
                ..Stat::default()
            },
            DEV_NULL_ID => Stat {
                r#type: InodeType::Device,
                mode: crate::fs::mode::S_IFCHR | 0o666,
                major: 1,
                minor: 0,
                nlink: 1,
                ..Stat::default()
            },
            DEV_ZERO_ID => Stat {
                r#type: InodeType::Device,
                mode: crate::fs::mode::S_IFCHR | 0o666,
                major: 1,
                minor: 1,
                nlink: 1,
                ..Stat::default()
            },
            _ => Stat::default(),
        }
    }
}
