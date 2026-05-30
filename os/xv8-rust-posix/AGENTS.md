# xv8-rust-posix — AGENTS.md

xv8 is a self-hosted POSIX-like OS for RISC-V (QEMU `-machine virt`). Monorepo containing `kernel/` and `user/` crates plus `mkfs/` and `_doc/`.

## Build & Run

```sh
# Workspace builds kernel + user
cargo build --release

# Run in QEMU (uses .cargo/config.toml runner)
cargo run --release

# Test (builds user, creates fresh 256M fs.img with test bins, runs QEMU)
./test.sh
```

- `test.sh` builds `user` package only (`--package user`), finds all `testbin/*.rs` files, prepends `_` to bin names, creates a fresh `target/fs.img` (256M), runs mkfs.sh to populate it, and launches QEMU with the test binaries + `testmode` marker.
- Tests run against a **completely fresh filesystem** each time. Any persistent state must be in the kernel binary itself.
- After test.sh, original `fs.img` is restored from `/tmp/fs.img.backup`.

## Architecture

### Kernel (`kernel/`)
- `#![no_std]` + `buddy-alloc` crate; `kalloc.rs` declares `#[global_allocator]` for the entire binary
- Entry: `src/main.rs` → `src/lib.rs:kmain()`
- `src/abi.rs` — types shared with userspace (Stat, Timespec, Termios, Errno, Syscall enum, constants)
- `src/fs.rs` — Inode table (NINODE=50 slots), DiskInode, Path, Directory, log-based journaling
- `src/sysfile.rs` — all FS-related syscalls (open/close/read/write/mkdir/chmod/chown/...)
- `src/sysproc.rs` — process syscalls (fork/exec/wait/getpid/setuid/...)
- `src/proc.rs` — ProcInner (uid/gid/pgid/sid/nice/...), ProcData (umask/cwd/...), process pool (NPROC=64)
- `src/file.rs` — FILE_TABLE, FileType (Inode/Device/Pipe/Socket), Ioctl constants, DEVICES[]
- `src/console.rs` — Console, line editor, raw mode, termios storage

### User (`user/`)
- `lib.rs` — `pub use kernel::abi::*`; `#![no_std]` + `#![feature(alloc)]` + `extern crate alloc`; shares kernel's `#[global_allocator]` (buddy-alloc)
- `bin/` — user programs (init, sh, cat, ls, etc.)
- `testbin/` — underscore-prefixed tests (registered in `testbin/testrunner.rs:TESTS`)
- Syscall wrappers in `src/syscall.rs`: `raw::*` (raw syscall numbers) and safe wrappers

### Key Types
- `Inode { id, dev, inum }` — lightweight handle, `Copy`, `Clone`
- `InodeInner` — in-memory inode data (valid/type/size/uid/gid/mode/addrs), protected by SleepLock
- `InodeMeta` — fast metadata (dev/inum/ref), protected by SpinLock
- `InodeTable` — `meta: SpinLock<[InodeMeta; NINODE]>` + `inner: [SleepLock<InodeInner>; NINODE]`
- `File { id }` — index into `FILE_TABLE`
- `FileInner` — readable/writeable/offset/type (Inode/Device/Pipe/Socket)

### Inode Lifecycle
1. `Inode::get(dev, inum)` → allocates/free slot in table, increments ref
2. `inode.lock()` → acquires SleepLock, reads from disk if `!valid`, returns guard
3. Modify InodeInner fields
4. `inode.unlock(guard)` → drops SleepLock
5. `inode.put()` → decrements ref; if ref==1 && nlink==0: truncate, free, set `valid=false`

### Path Resolution (`Path::resolve`)
- Walks from root (`Inode::get(ROOTDEV, ROOTINO)`) or `cwd`
- Each component: `inode.lock()` → `Directory::lookup()` → advance
- `Directory::lookup()` reads on-disk dirents via `inode.read()`

### Lock Ordering
SpinLock (meta) → SleepLock (inner). Never acquire SleepLock while holding SpinLock except in `put()` when ref==1 guarantees no contention.

### Memory Model
xv8 is a **static binary** (kernel + user statically linked into one ELF). There is exactly **one `#[global_allocator]`** in the final binary, provided by `kernel/src/kalloc.rs` (using `buddy-alloc`).

User space (`#![no_std]` + `#![feature(alloc)]` + `extern crate alloc`) does **not** declare its own `#[global_allocator]`. All allocation requests (`Vec`, `String`, `Box`, etc.) flow through the kernel's `buddy-alloc`.

The path is: `alloc crate → GlobalAlloc::alloc() → buddy-alloc → sbrk(12) / mmap(34) syscall → kernel`.

`user/src/args.rs` provides `Args::from_stack()` which reads argc/argv from the stack. This can be converted to `Vec<String>` using `alloc::vec![String]` once `alloc` is available.

### libposix Dual Target
`os/posix/libposix/` is designed to compile for both Mac (`#[cfg(unix)]`) and xv8 (`#[cfg(target_os = "none")`):

| Layer | Mac | xv8 |
|-------|-----|-----|
| Base | `std` | `no_std` + `alloc` |
| I/O | `std::io` + `libc` | `ecall` inline asm |
| Args | `std::env::args()` | `Args::from_stack()` → `Vec<String>` |
| Exit | `std::process::exit()` | `sys_exit` (ecall) |

`libposix/io.rs` uses `#[cfg(unix)]` / `#[cfg(target_os = "none")]` cfg gates to provide the same public API (`File`, `Read`, `Write`, `print`, `println`, `exit`, `args`, etc.) on both platforms.

### Log/Journal
- `log::write(&buf)` instead of `BCACHE::write()`; `log::commit()` on `Operation::drop` when outstanding==0
- Every FS mutation must be wrapped in `Operation::begin()` / `log!(...)` / `drop(op)`.

### v0.10 VFS (complete)
- `kernel/src/vfs.rs` — `VfsOps` trait, mount table (MAX_MOUNTS=8), ProcFs/DevFs implementations, `check_mount()`, `init()`
- `kernel/src/fs.rs` — `InodeInner.vfs: Option<&'static dyn VfsOps>`, `Inode::alloc_vfs()`, path resolution integration
- `sys_mount`/`sys_umount` at syscall indices 80/81
- `user/src/syscall.rs` — `raw::mount()`, `raw::umount()`, `syscall4()`
- `_posix_vfs` test — mount/umount/read /proc/self/status/open /dev (6 subtests, all pass)

## Critical Bug History

### v0.8: Inode::get() reset valid=false on slot reuse (fixed)
When `ref==0` and a slot's `(dev,inum)` matched the lookup, the code fell through to the `empty` path and set `valid=false`, causing `lock()` to re-read from disk and overwrite uid/gid with 0.
**Fix:** Changed `if inode.r#ref > 0 && inode.dev == dev && inode.inum == inum` to `if inode.dev == dev && inode.inum == inum` (match regardless of ref). The refcount is incremented (`r#ref += 1`) for both ref>0 and ref==0 cases.

### v0.8: sys_chmod missing Operation::begin()
`log::write()` called outside a transaction panics. `sys_chmod` now calls `Operation::begin()` before modifying the inode.

## Version History (see `_doc/v*.md`)

| Version | Content |
|---------|---------|
| v0.1 | Initial xv8 port from xv7 |
| v0.2 | POSIX errno + syscall table |
| v0.3 | Signals (SIGKILL/TERM/INT) |
| v0.4 | File I/O extensions (lseek/truncate/getdents/fcntl/symlink) |
| v0.5 | mmap (MAP_ANONYMOUS/MAP_PRIVATE) |
| v0.6 | Process management (setsid/getpgid/getppid/nice) |
| v0.7 | clock_gettime + nanosleep |
| v0.8 | uid/gid/umask + chmod/chown + inode slot reuse fix |
| v0.9 | Termios (TCGETS/TCSETS, Console::termios) |
| v0.10 | VFS + Mount (complete) — ProcFs, DevFs, mount/umount syscalls, path resolution integration |

## Important Notes

- **Rust edition 2024** — `let` chains (`let Some(x) = ... && condition`) and `else {}` blocks have different type inference behavior vs 2021. Stick to explicit conditionals for clarity.
- **uid/gid not persisted** — InodeInner.uid/gid are in-memory only. On every boot, all inodes start with uid=0.
- **DiskInode has no uid/gid** — Only type/major/minor/nlink/size/addrs. The uid/gid are loaded from memory and never written to disk.
- **FILESYSTEM_NAME length** — DIRSIZE=14 bytes per filename. Keep test binary names ≤13 chars (e.g., `_posix_tty` works, `_posix_termios` does not).
- **println! on bare metal** — `println!` writes to the UART/console. Heavy debug output during boot can cause QEMU to hang or timeout. Use sparingly.
- **Inode slot reuse** — After `put()` sets ref=0, the slot retains valid=true and all InodeInner fields. A subsequent `get()` for the same `(dev,inum)` will find the slot with ref==0, increment ref, and NOT reset valid. This preserves in-memory data.
- **No `Drop` impl on `Inode`** — You must call `put()` explicitly. When `Inode` goes out of scope without `put()`, the refcount leaks.
- **Pre-existing warnings** — kernel: unused `fd` in sys_mmap, unused `flags` in MmapRegion; user: unused CLOCK_MONOTONIC/CLOCK_REALTIME imports.