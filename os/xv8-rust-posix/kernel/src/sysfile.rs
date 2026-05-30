use core::mem;
use core::slice;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::abi::OpenFlag;
use crate::exec::exec;
use crate::file::{FILE_TABLE, File, FileType};
use crate::fs::{Dirent, Directory, Inode, InodeType, Path};
use crate::log::Operation;
use crate::param::{MAXARG, MAXPATH, NDEV, NOFILE};
use crate::pipe::Pipe;
use crate::proc;
use crate::proc::current_proc_and_data_mut;
use crate::riscv::PGSIZE;
use crate::syscall::{Errno, SyscallArgs};
use crate::vm::VA;

/// Allocates a file descriptor for the give file.
/// Takes over file reference from caller on success.
pub fn fd_alloc(file: File) -> Result<usize, Errno> {
    let (_proc, data) = current_proc_and_data_mut();

    for (fd, open_file) in data.open_files.iter_mut().enumerate() {
        if open_file.is_none() {
            *open_file = Some(file);
            return Ok(fd);
        }
    }

    err!(Errno::EMFILE)
}

pub fn sys_dup(args: &SyscallArgs) -> Result<usize, Errno> {
    let (_, mut file) = try_log!(args.get_file(0));
    let fd = try_log!(fd_alloc(file.clone()));
    file.dup();
    Ok(fd)
}

pub fn sys_read(args: &SyscallArgs) -> Result<usize, Errno> {
    let addr = args.get_addr(1);
    let n = args.get_int(2);
    let (_, file) = try_log!(args.get_file(0));
    log!(file.read(addr, n as usize))
}

pub fn sys_write(args: &SyscallArgs) -> Result<usize, Errno> {
    let addr = args.get_addr(1);
    let n = args.get_int(2);
    let (_, mut file) = try_log!(args.get_file(0));
    log!(file.write(addr, n as usize))
}

pub fn sys_close(args: &SyscallArgs) -> Result<usize, Errno> {
    let (fd, mut file) = try_log!(args.get_file(0));

    let (_proc, data) = current_proc_and_data_mut();

    data.open_files[fd] = None;
    file.close();

    Ok(0)
}

pub fn sys_fstat(args: &SyscallArgs) -> Result<usize, Errno> {
    let addr = args.get_addr(1);
    let (_, file) = try_log!(args.get_file(0));
    try_log!(file.stat(addr));
    Ok(0)
}

pub fn sys_link(args: &SyscallArgs) -> Result<usize, Errno> {
    let old = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let new = try_log!(args.fetch_string(args.get_addr(1), MAXPATH));

    let _op = Operation::begin();

    // get the inode of the old
    let Ok(old_inode) = log!(Path::new(&old).resolve()) else {
        err!(Errno::ENOENT)
    };

    let mut old_inner = old_inode.lock();

    // make sure it is not a directory
    if old_inner.r#type == InodeType::Directory {
        old_inode.unlock_put(old_inner);
        err!(Errno::EPERM);
    }

    // increment number of links pointing to the inode
    old_inner.nlink += 1;
    old_inode.update(&old_inner);
    old_inode.unlock(old_inner);

    // after incrementing nlink, failures must goto `bad`
    let result = (|| {
        // get the inode of the new's parent
        let (parent, name) = match log!(Path::new(&new).resolve_parent()) {
            Ok(v) => v,
            Err(_) => err!(Errno::ENOENT),
        };

        // make sure they are in the same device
        if parent.dev != old_inode.dev {
            err!(Errno::EXDEV);
        }

        let mut parent_inner = parent.lock();

        // add the inode to the new's parent
        if let Err(e) = log!(Directory::link(
            &parent,
            &mut parent_inner,
            name,
            old_inode.inum as u16
        )) {
            parent.unlock_put(parent_inner);
            err!(Errno::from(e));
        }

        parent.unlock_put(parent_inner);
        Ok(0)
    })();

    // bad
    if result.is_err() {
        let mut old_inner = old_inode.lock();
        old_inner.nlink -= 1;
        old_inode.update(&old_inner);
        old_inode.unlock(old_inner);
    }

    old_inode.put();

    result
}

pub fn sys_unlink(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let _op = Operation::begin();

    // get the parent inode and name
    let Ok((parent, name)) = log!(Path::new(&path).resolve_parent()) else {
        err!(Errno::ENOENT);
    };

    let mut parent_inner = parent.lock();

    // cannot unlink `.` or `..`
    if name == "." || name == ".." {
        parent.unlock_put(parent_inner);
        err!(Errno::EINVAL);
    }

    // find the inode in the parent's directory entry
    let Ok(Some((offset, inode))) = log!(Directory::lookup(&parent, &mut parent_inner, name))
    else {
        parent.unlock_put(parent_inner);
        err!(Errno::ENOENT);
    };

    let mut inode_inner = inode.lock();

    assert!(inode_inner.nlink >= 1, "unlink nlink < 1");

    // if the inode is a directory and it is not empty, cannot unlink
    if inode_inner.r#type == InodeType::Directory && !Directory::is_empty(&inode, &mut inode_inner)
    {
        inode.unlock_put(inode_inner);
        parent.unlock_put(parent_inner);
        err!(Errno::ENOTEMPTY);
    }

    // replace the directory entry with an empty one
    let dir = Directory::new_empty();
    match log!(parent.write(&mut parent_inner, offset, dir.as_bytes(), false)) {
        Ok(write) => {
            assert_eq!(write, Directory::SIZE as u32, "unlink write");
        }
        Err(_) => {
            parent.unlock_put(parent_inner);
            err!(Errno::EIO)
        }
    }

    // if it is a directory, decrement parent's link count
    if inode_inner.r#type == InodeType::Directory {
        parent_inner.nlink -= 1;
        parent.update(&parent_inner);
    }
    parent.unlock_put(parent_inner);

    // decrement the inode's link count
    inode_inner.nlink -= 1;
    inode.update(&inode_inner);
    inode.unlock_put(inode_inner);

    Ok(0)
}

pub fn sys_open(args: &SyscallArgs) -> Result<usize, Errno> {
    let o_mode = args.get_int(1) as usize;
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let path = Path::new(&path);

    let _op = Operation::begin();

    let (mut inode, mut inode_inner);

    // either create a new file or find the file from the path
    if (o_mode & OpenFlag::CREATE) != 0 {
        (inode, inode_inner) = match log!(Inode::create(&path, InodeType::File, 0, 0)) {
            Ok(i) => i,
            Err(e) => {
                err!(Errno::from(e))
            }
        };
        // Set owner and permissions from current process
        let (proc, data) = current_proc_and_data_mut();
        let p_inner = proc.inner.lock();
        inode_inner.uid = p_inner.uid;
        inode_inner.gid = p_inner.gid;
        drop(p_inner);
        inode_inner.mode = crate::fs::mode::from_type(InodeType::File) & !(data.umask as u16);
    } else {
        inode = match log!(path.resolve()) {
            Ok(i) => i,
            Err(_) => {
                err!(Errno::ENOENT);
            }
        };

        inode_inner = inode.lock();

        // if it is a directory, cannot open with write mode
        if inode_inner.r#type == InodeType::Directory && o_mode != OpenFlag::READ_ONLY {
            inode.unlock_put(inode_inner);
            err!(Errno::EISDIR);
        }
    }

    // cannot open device out of range
    if inode_inner.r#type == InodeType::Device && inode_inner.major >= NDEV as u16 {
        inode.unlock_put(inode_inner);
        err!(Errno::ENOENT);
    }

    // allocate a file structure and a file descriptor
    let (fd, file) = match log!(File::alloc()) {
        Ok(mut file) => match log!(fd_alloc(file.clone())) {
            Ok(fd) => (fd, file),
            Err(e) => {
                // if err here, we must also close the file
                file.close();
                inode.unlock_put(inode_inner);
                return Err(e);
            }
        },
        Err(e) => {
            inode.unlock_put(inode_inner);
            err!(Errno::from(e));
        }
    };

    let mut file_inner = FILE_TABLE.inner[file.id].lock();
    if inode_inner.r#type == InodeType::Device {
        file_inner.r#type = FileType::Device {
            inode: inode.clone(),
            major: inode_inner.major,
        };
    } else {
        file_inner.r#type = FileType::Inode {
            inode: inode.clone(),
        };
        file_inner.offset = 0;
    }
    file_inner.readable = (o_mode & OpenFlag::WRITE_ONLY) == 0;
    file_inner.writeable =
        (o_mode & OpenFlag::WRITE_ONLY) != 0 || (o_mode & OpenFlag::READ_WRITE != 0);

    if (o_mode & OpenFlag::TRUNCATE) != 0 && inode_inner.r#type == InodeType::File {
        inode.trunc(&mut inode_inner);
    }

    inode.unlock(inode_inner);

    Ok(fd)
}

pub fn sys_mkdir(args: &SyscallArgs) -> Result<usize, Errno> {
    let _op = Operation::begin();

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let (inode, mut inode_inner) =
        match log!(Inode::create(&Path::new(&path), InodeType::Directory, 0, 0)) {
            Ok(i) => i,
            Err(e) => err!(Errno::from(e)),
        };

    // Set owner and permissions from current process
    let (proc, data) = current_proc_and_data_mut();
    let p_inner = proc.inner.lock();
    inode_inner.uid = p_inner.uid;
    inode_inner.gid = p_inner.gid;
    drop(p_inner);
    inode_inner.mode = crate::fs::mode::from_type(InodeType::Directory) & !(data.umask as u16);

    inode.unlock_put(inode_inner);

    Ok(0)
}

pub fn sys_mknod(args: &SyscallArgs) -> Result<usize, Errno> {
    let _op = Operation::begin();

    let major = args.get_int(1) as u16;
    let minor = args.get_int(2) as u16;
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let (inode, mut inner) = match log!(Inode::create(
        &Path::new(&path),
        InodeType::Device,
        major,
        minor,
    )) {
        Ok(i) => i,
        Err(e) => err!(Errno::from(e)),
    };

    // Set owner and permissions from current process
    let (proc, data) = current_proc_and_data_mut();
    let p_inner = proc.inner.lock();
    inner.uid = p_inner.uid;
    inner.gid = p_inner.gid;
    drop(p_inner);
    inner.mode = crate::fs::mode::from_type(InodeType::Device) & !(data.umask as u16);

    inode.unlock_put(inner);

    Ok(0)
}

pub fn sys_chdir(args: &SyscallArgs) -> Result<usize, Errno> {
    let (_proc, data) = current_proc_and_data_mut();

    let _op = Operation::begin();

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));

    let Ok(inode) = log!(Path::new(&path).resolve()) else {
        err!(Errno::ENOENT);
    };

    let inner = inode.lock();

    if inner.r#type != InodeType::Directory {
        inode.unlock_put(inner);
        err!(Errno::ENOTDIR);
    }

    inode.unlock(inner);

    let old_cwd = mem::replace(&mut data.cwd, inode);
    old_cwd.put();

    Ok(0)
}

pub fn sys_exec(args: &SyscallArgs) -> Result<usize, Errno> {
    let uargv = args.get_addr(1);

    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let path = Path::new(&path);

    let (_proc, data) = current_proc_and_data_mut();

    let mut argv_bufs: Vec<String> = Vec::with_capacity(MAXARG);

    for i in 0..MAXARG {
        // fetch pointer argv[i] from user space
        let mut uarg: usize = 0;
        let dst = unsafe {
            slice::from_raw_parts_mut(&mut uarg as *mut usize as *mut u8, size_of::<usize>())
        };
        if log!(
            data.pagetable_mut()
                .copy_from(uargv + i * size_of::<usize>(), dst)
        )
        .is_err()
        {
            err!(Errno::EFAULT);
        }

        if uarg == 0 {
            break; // NULL terminator
        }

        // fetch string from user space
        let s = try_log!(args.fetch_string(VA::from(uarg), PGSIZE));
        argv_bufs.push(s);
    }

    let argv: Vec<&str> = argv_bufs.iter().map(|s| s.as_str()).collect::<Vec<_>>();

    log!(exec(&path, &argv)).map_err(|_| Errno::ENOEXEC)
}

pub fn sys_pipe(args: &SyscallArgs) -> Result<usize, Errno> {
    // user pointer to array of two integers
    let fd_array = args.get_addr(0);

    let (_proc, data) = current_proc_and_data_mut();

    let (mut read, mut write) = match log!(Pipe::alloc()) {
        Ok(pair) => pair,
        Err(e) => err!(Errno::from(e)),
    };

    let Ok(fd0) = log!(fd_alloc(read.clone())) else {
        read.close();
        write.close();
        err!(Errno::EMFILE);
    };

    let Ok(fd1) = log!(fd_alloc(write.clone())) else {
        data.open_files[fd0] = None;
        read.close();
        write.close();
        err!(Errno::EMFILE);
    };

    let pagetable = data.pagetable_mut();

    if log!(pagetable.copy_to(&fd0.to_le_bytes(), fd_array)).is_err()
        || log!(pagetable.copy_to(&fd1.to_le_bytes(), fd_array + size_of_val(&fd1))).is_err()
    {
        data.open_files[fd0] = None;
        data.open_files[fd1] = None;
        read.close();
        write.close();
        err!(Errno::EFAULT);
    }

    Ok(0)
}

pub fn sys_ioctl(args: &SyscallArgs) -> Result<usize, Errno> {
    let ioctl_cmd = args.get_int(1) as usize;
    let ioctl_arg = args.get_int(2) as usize;
    let (_, file) = try_log!(args.get_file(0));
    log!(file.ioctl(ioctl_cmd, ioctl_arg))
}

pub fn sys_lseek(args: &SyscallArgs) -> Result<usize, Errno> {
    let offset = args.get_int(1) as i64;
    let whence = args.get_int(2) as u32;
    let (_, file) = try_log!(args.get_file(0));
    let new_off = try_log!(file.seek(offset, whence));
    Ok(new_off as usize)
}

pub fn sys_truncate(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let _op = Operation::begin();
    let mut inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(Errno::ENOENT),
    };
    let mut inner = inode.lock();
    if inner.r#type == InodeType::Directory {
        inode.unlock_put(inner);
        err!(Errno::EISDIR);
    }
    inode.trunc(&mut inner);
    inode.unlock_put(inner);
    Ok(0)
}

pub fn sys_ftruncate(args: &SyscallArgs) -> Result<usize, Errno> {
    let len = args.get_int(1) as usize;
    let (_, file) = try_log!(args.get_file(0));
    let file_inner = FILE_TABLE.inner[file.id].lock();
    match &file_inner.r#type {
        FileType::Inode { inode } | FileType::Device { inode, .. } => {
            if !file_inner.writeable {
                err!(Errno::EBADF);
            }
            let mut inode = inode.clone();
            drop(file_inner);
            let mut inode_inner = inode.lock();
            if len < inode_inner.size as usize {
                inode.trunc(&mut inode_inner);
            }
            inode.unlock(inode_inner);
            Ok(0)
        }
        _ => err!(Errno::EBADF),
    }
}

pub fn sys_getdents(args: &SyscallArgs) -> Result<usize, Errno> {
    let buf_addr = args.get_addr(1);
    let buf_len = args.get_int(2) as usize;
    let (_, file) = try_log!(args.get_file(0));

    let file_inner = FILE_TABLE.inner[file.id].lock();
    match &file_inner.r#type {
        FileType::Inode { inode } => {
            let inode = inode.clone();
            let mut offset = file_inner.offset;
            drop(file_inner);

            let mut inode_inner = inode.lock();
            if inode_inner.r#type != InodeType::Directory {
                inode.unlock(inode_inner);
                err!(Errno::EBADF);
            }

            let mut total = 0usize;
            let entsize = crate::fs::DIRSIZE + 2; // 2 for inum u16
            let mut dirbuf = [0u8; Directory::SIZE];

            while offset < inode_inner.size && total + entsize <= buf_len {
                let read = try_log!(inode.read(
                    &mut inode_inner,
                    offset,
                    &mut dirbuf,
                    false,
                ));
                if read == 0 {
                    break;
                }

                let dir = Directory::from_bytes(&dirbuf);
                offset += Directory::SIZE as u32;

                if dir.inum == 0 {
                    continue;
                }

                let dirent = Dirent::from(&dir);
                let src = unsafe {
                    slice::from_raw_parts(&dirent as *const _ as *const u8, entsize)
                };
                if log!(proc::copy_to_user(src, buf_addr + total)).is_err() {
                    if total == 0 {
                        inode.unlock(inode_inner);
                        err!(Errno::EFAULT);
                    }
                    break;
                }
                total += entsize;
            }

            // Update file offset
            let mut f_inner = FILE_TABLE.inner[file.id].lock();
            f_inner.offset = offset;
            drop(f_inner);

            inode.unlock(inode_inner);
            Ok(total)
        }
        _ => {
            drop(file_inner);
            err!(Errno::EBADF);
        }
    }
}

pub fn sys_symlink(args: &SyscallArgs) -> Result<usize, Errno> {
    let target = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let linkpath = try_log!(args.fetch_string(args.get_addr(1), MAXPATH));

    let _op = Operation::begin();

    let (parent, name) = match log!(Path::new(&linkpath).resolve_parent()) {
        Ok(v) => v,
        Err(_) => err!(Errno::ENOENT),
    };

    // check target does not already exist
    if let Ok(Some(_)) = log!(Directory::lookup(
        &parent,
        &mut parent.lock(),
        name,
    )) {
        parent.put();
        err!(Errno::EEXIST);
    }

    let inode = match log!(Inode::alloc(parent.dev, InodeType::File)) {
        Ok(i) => i,
        Err(e) => {
            parent.put();
            err!(Errno::from(e));
        }
    };

    let mut inode_inner = inode.lock();
    inode_inner.nlink = 1;
    inode.update(&inode_inner);

    // write target path into inode
    let target_bytes = target.as_bytes();
    let written = try_log!(inode.write(&mut inode_inner, 0, target_bytes, false));
    if written != target_bytes.len() as u32 {
        inode_inner.nlink = 0;
        inode.update(&inode_inner);
        inode.unlock_put(inode_inner);
        parent.put();
        err!(Errno::ENOSPC);
    }

    inode.unlock(inode_inner);

    if log!(Directory::link(
        &parent,
        &mut parent.lock(),
        name,
        inode.inum as u16,
    ))
    .is_err()
    {
        let mut inner = inode.lock();
        inner.nlink = 0;
        inode.update(&inner);
        inode.unlock_put(inner);
        parent.put();
        err!(Errno::ENOSPC);
    }

    parent.put();
    inode.put();

    Ok(0)
}

pub fn sys_readlink(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let buf_addr = args.get_addr(1);
    let buf_len = args.get_int(2) as usize;

    let _op = Operation::begin();

    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(Errno::ENOENT),
    };

    let mut inode_inner = inode.lock();
    let read_len = buf_len.min(inode_inner.size as usize);
    let mut buf = vec![0u8; read_len];
    let read = try_log!(inode.read(&mut inode_inner, 0, &mut buf, false));

    inode.unlock_put(inode_inner);

    if log!(proc::copy_to_user(
        &buf[..read as usize],
        buf_addr,
    ))
    .is_err()
    {
        err!(Errno::EFAULT);
    }

    Ok(read as usize)
}

pub fn sys_access(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let _mode = args.get_int(1) as u32;

    match log!(Path::new(&path).resolve()) {
        Ok(inode) => {
            inode.put();
            Ok(0)
        }
        Err(_) => err!(Errno::ENOENT),
    }
}

pub fn sys_fcntl(args: &SyscallArgs) -> Result<usize, Errno> {
    let fd = args.get_int(0) as usize;
    let cmd = args.get_int(1) as usize;
    let arg = args.get_int(2) as usize;

    let (_proc, data) = current_proc_and_data_mut();

    match cmd {
        F_DUPFD => {
            if fd >= NOFILE || data.open_files[fd].is_none() {
                err!(Errno::EBADF);
            }
            let mut target = arg.max(fd + 1).max(0);
            while target < NOFILE {
                if data.open_files[target].is_none() {
                    let file = data.open_files[fd].as_mut().unwrap();
                    let mut dup_file = file.clone();
                    dup_file.dup();
                    data.open_files[target] = Some(dup_file);
                    return Ok(target);
                }
                target += 1;
            }
            err!(Errno::EMFILE)
        }
        F_GETFD => Ok(0), // no close-on-exec yet
        F_SETFD => {
            let _flags = arg;
            Ok(0)
        }
        F_GETFL => {
            if fd >= NOFILE || data.open_files[fd].is_none() {
                err!(Errno::EBADF);
            }
            let file_inner = FILE_TABLE.inner[data.open_files[fd].as_ref().unwrap().id].lock();
            let flags = if file_inner.readable && file_inner.writeable {
                2
            } else if file_inner.writeable {
                1
            } else {
                0
            };
            Ok(flags)
        }
        _ => err!(Errno::EINVAL),
    }
}

pub fn sys_dup2(args: &SyscallArgs) -> Result<usize, Errno> {
    let oldfd = args.get_int(0) as usize;
    let newfd = args.get_int(1) as usize;

    let (_proc, data) = current_proc_and_data_mut();

    if oldfd >= NOFILE || data.open_files[oldfd].is_none() {
        err!(Errno::EBADF);
    }
    if newfd >= NOFILE {
        err!(Errno::EBADF);
    }

    if oldfd == newfd {
        return Ok(oldfd);
    }

    // close newfd if open
    if let Some(mut file) = data.open_files[newfd].take() {
        file.close();
    }

    let mut file = data.open_files[oldfd].as_ref().unwrap().clone();
    file.dup();
    data.open_files[newfd] = Some(file);

    Ok(newfd)
}

pub const F_DUPFD: usize = 0;
pub const F_GETFD: usize = 1;
pub const F_SETFD: usize = 2;
pub const F_GETFL: usize = 3;

pub fn sys_chmod(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let mode = args.get_raw(1) as u16;

    let _op = Operation::begin();

    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(Errno::ENOENT),
    };

    let mut inner = inode.lock();
    inner.mode = (inner.mode & 0o170000) | (mode & 0o7777) as u16;
    inode.update(&inner);
    inode.unlock_put(inner);
    Ok(0)
}

pub fn sys_chown(args: &SyscallArgs) -> Result<usize, Errno> {
    let path = try_log!(args.fetch_string(args.get_addr(0), MAXPATH));
    let uid = args.get_raw(1) as u32;
    let gid = args.get_raw(2) as u32;

    let inode = match log!(Path::new(&path).resolve()) {
        Ok(i) => i,
        Err(_) => err!(Errno::ENOENT),
    };

    let mut inner = inode.lock();
    if uid != u32::MAX {
        inner.uid = uid;
    }
    if gid != u32::MAX {
        inner.gid = gid;
    }
    inode.unlock_put(inner);
    Ok(0)
}

pub fn sys_umask(args: &SyscallArgs) -> Result<usize, Errno> {
    let new_mask = args.get_raw(0) as u16;
    let (_, data) = current_proc_and_data_mut();
    let old = data.umask;
    data.umask = new_mask & 0o777;
    Ok(old as usize)
}
