# xv8-rust-posix 規劃書

## 緣起

xv7-rust-octopus 是一個在 QEMU RISC-V 上運作的教學用作業系統，
繼承 MIT xv6 的設計精神。xv8 目標是在 xv7 的基礎上，
導入 POSIX.1-2008 相容的系統呼叫介面，讓使用者程式能
以接近 Linux/POSIX 的方式開發移植。

## 設計原則

1. **逐步演化** — 不一次重寫核心，而是在 xv7 架構上逐層加裝 POSIX 層
2. **雙面 ABI** — 內部保留 xv7 的簡潔實作，外部提供 POSIX 系統呼叫編號與語義
3. **#![no_std]** — 延續無標準函式庫核心風格
4. **RISC-V 64** — 鎖定 `riscv64gc-unknown-none-elf`，QEMU `-machine virt`
5. **Rust 原生** — 使用者程式以 Rust `user` crate 開發，核心與使用者空間生態皆以 Rust 為主

## 里程碑

| 階段 | 目標 | 相依 |
|------|------|------|
| M0 | 系統呼叫重新編號 + POSIX errno | xv7 |
| M1 | 信號框架 (signal) | M0 |
| M2 | 檔案系統增強 (lseek, ftruncate, getdents, symlink) | M0 |
| M3 | 記憶體映射 (mmap/munmap/mprotect) | M0 |
| M4 | POSIX 行程管理 (setsid, getpgid, nice) | M0 |
| M5 | POSIX 時間 (clock_gettime, nanosleep, timer) | M0 |
| M6 | 權限模型 (uid/gid, chmod, chown, umask) | M0 |
| M7 | termios (tcgetattr/tcsetattr, pty) | M0, M6 |
| M8 | 掛載 (mount/umount, VFS 抽象層) | M2 |
| M9 | 穩定版（回歸測試、壓力測試、API 凍結） | M0-M8 |
| M10 | BSD Socket API (socket/bind/sendto/recvfrom/connect) — v1.1 | M9 |

## 架構變更

### 系統呼叫層

```rust
// xv7 原有 (27 支)
Syscall::Fork=1, Exit=2, Wait=3, Pipe=4, Read=5,
Kill=6, Exec=7, Fstat=8, Chdir=9, Dup=10,
Getpid=11, Sbrk=12, Sleep=13, Uptime=14,
Open=15, Write=16, Mknod=17, Unlink=18,
Link=19, Mkdir=20, Close=21, Poweroff=22,
Ioctl=23, Socket=24, Send=25, Receive=26, Random=27

// xv8 擴充（POSIX 編號空間）
// 保留 1-27 相容，新增 28-80
Syscall::Lseek=28, Truncate=29, Ftruncate=30,
Getdents=31, Symlink=32, Readlink=33,
Mmap=34, Munmap=35, Mprotect=36,
Signal=37, Sigaction=38, Sigprocmask=39, Sigreturn=40,
ClockGetTime=41, NanoSleep=42,
SetSid=43, GetPgid=44, GetPpid=45, Nice=46,
Socket2=47, Bind=48, Listen=49, Accept=50,
Connect=51, Sendto=52, Recvfrom=53, Shutdown=54,
Chmod=55, Chown=56, Umask=57,
Mount=58, Umount=59,
Symlink=32, Readlink=33, // 重複？需要重新整理
Fcntl=60, Fsync=61,
TcGetAttr=62, TcSetAttr=63,
Uname=64, GetUid=65, GetGid=66, SetUid=67, SetGid=68,
```

### errno

從 xv7 自訂錯誤碼改為 POSIX errno 常數：

```rust
pub enum Errno {
    EPERM   = 1,
    ENOENT  = 2,
    ESRCH   = 3,
    EINTR   = 4,
    EIO     = 5,
    ENXIO   = 6,
    E2BIG   = 7,
    ENOEXEC = 8,
    EBADF   = 9,
    ECHILD  = 10,
    EAGAIN  = 11,
    ENOMEM  = 12,
    EACCES  = 13,
    EFAULT  = 14,
    ENOTBLK = 15,
    EBUSY   = 16,
    EEXIST  = 17,
    EXDEV   = 18,
    ENODEV  = 19,
    ENOTDIR = 20,
    EISDIR  = 21,
    EINVAL  = 22,
    ENFILE  = 23,
    EMFILE  = 24,
    ENOTTY  = 25,
    ETXTBSY = 26,
    EFBIG   = 27,
    ENOSPC  = 28,
    ESPIPE  = 29,
    EROFS   = 30,
    EMLINK  = 31,
    EPIPE   = 32,
    EDOM    = 33,
    ERANGE  = 34,
    ENAMETOOLONG = 36,
    ENOSYS  = 38,
    ENOTEMPTY = 39,
    ELOOP   = 40,
    EMSGSIZE = 90,
    // ... 依需求擴充
}
```

回傳方式：保持 xv7 慣例，`trapframe.a0 = -(errno as isize)`。

### 信號架構 (M1)

```rust
pub const NSIG: usize = 32;

pub enum Signal {
    SIGHUP  = 1,
    SIGINT  = 2,
    SIGQUIT = 3,
    SIGILL  = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS  = 7,
    SIGFPE  = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    SIGTERM = 15,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
}

// 每個行程保有 sigaction 表
struct ProcInner {
    // ...原有欄位...
    sigactions: [SigAction; NSIG],
    pending: AtomicU32,      // bitmask of pending signals
    blocked: AtomicU32,      // bitmask of blocked signals
}

struct SigAction {
    handler: usize,          // SIG_DFL=0, SIG_IGN=1, or fn pointer
    flags: u32,
    mask: u32,
}
```

信號遞送時機：`usertrap()` / `kerneltrap()` 返回前檢查 `pending & ~blocked`，
若無自訂 handler 則採預設動作（terminate/stop/ignore）。

### 檔案系統增強 (M2)

```rust
// 新增 lseek
pub fn sys_lseek(fd: Fd, offset: i64, whence: u32) -> Result<u64, Errno>;  // syscall 28

// 新增 ftruncate
pub fn sys_ftruncate(fd: Fd, length: usize) -> Result<(), Errno>;          // syscall 30

// 新增 getdents — 目錄讀取
pub fn sys_getdents(fd: Fd, buf: &mut [u8], len: usize) -> Result<usize, Errno>;

// 新增 symlink / readlink
pub fn sys_symlink(target: &str, linkpath: &str) -> Result<(), Errno>;
pub fn sys_readlink(path: &str, buf: &mut [u8]) -> Result<usize, Errno>;
```

### 記憶體映射 (M3)

```rust
pub fn sys_mmap(
    addr: Option<VA>,        // 提示位址（通常 NULL）
    length: usize,
    prot: MmapProt,          // PROT_READ | PROT_WRITE | PROT_EXEC | PROT_NONE
    flags: MmapFlags,        // MAP_PRIVATE | MAP_SHARED | MAP_ANONYMOUS | MAP_FIXED
    fd: Option<Fd>,
    offset: usize,
) -> Result<VA, Errno>;

pub fn sys_munmap(addr: VA, length: usize) -> Result<(), Errno>;
pub fn sys_mprotect(addr: VA, length: usize, prot: MmapProt) -> Result<(), Errno>;
```

實作方式：
- `MAP_ANONYMOUS` 對接現有 `Uvm::alloc()` / `sbrk()` 機制
- 檔案映射需新增 `vm/filebacked.rs` 模組，透過 inode read/write 進行 page fault 回填
- RISC-V Sv39 頁表權限直接對應 `PTE_R`/`PTE_W`/`PTE_X`

### POSIX 行程管理 (M4)

```rust
pub fn sys_setsid() -> Result<Pid, Errno>;           // 新建 session
pub fn sys_getpgid(pid: Pid) -> Result<Pid, Errno>;  // 取行程群組
pub fn sys_getppid() -> Pid;                         // 取父行程 PID
pub fn sys_nice(inc: isize) -> Result<isize, Errno>; // 調整優先權
```

核心新增欄位：
```rust
struct ProcData {
    // ...原有...
    pgid: Pid,       // process group
    sid: Pid,        // session id
    priority: i8,    // -20..19, POSIX nice value
    umask: u16,      // file creation mask
    uid: u32,        // user id
    gid: u32,        // group id
}
```

### POSIX 時間 (M5)

```rust
pub struct Timespec {
    tv_sec: i64,
    tv_nsec: i64,
}

pub fn sys_clock_gettime(clock_id: u32, tp: &mut Timespec) -> Result<(), Errno>;
pub fn sys_nanosleep(req: &Timespec, rem: &mut Timespec) -> Result<(), Errno>;
```

現有 `stimecmp` / `TICKS` 機制擴充，支援 `CLOCK_MONOTONIC` 和 `CLOCK_REALTIME`。

### BSD Socket API (M6)

xv7 已有 UDP socket（`socket`/`send`/`receive` 三支自訂呼叫），
改為標準 BSD 風格：

```rust
pub fn sys_socket(domain: u32, type_: u32, protocol: u32) -> Result<Fd, Errno>;
pub fn sys_bind(fd: Fd, addr: &SockAddr, addrlen: u32) -> Result<(), Errno>;
pub fn sys_sendto(fd: Fd, buf: &[u8], flags: u32, dest: &SockAddr, addrlen: u32) -> Result<usize, Errno>;
pub fn sys_recvfrom(fd: Fd, buf: &mut [u8], flags: u32, src: &mut SockAddr, addrlen: &mut u32) -> Result<usize, Errno>;
pub fn sys_connect(fd: Fd, addr: &SockAddr, addrlen: u32) -> Result<(), Errno>;
pub fn sys_shutdown(fd: Fd, how: u32) -> Result<(), Errno>;
```

UDP 實作直接包裝現有 e1000 + net stack，TCP 可先以 `ENOSYS` 回應或逐步實作。

### 權限模型 (M7)

```rust
pub fn sys_chmod(path: &str, mode: u16) -> Result<(), Errno>;
pub fn sys_chown(path: &str, uid: u32, gid: u32) -> Result<(), Errno>;
pub fn sys_umask(mask: u16) -> u16;
```

Inode 新增 `mode` 欄位（POSIX `st_mode` 位元遮罩），現有 `InodeType` 改為
「以 `S_IFMT` 辨識」。

### termios (M8)

```rust
pub struct Termios {
    c_iflag: u32,    // input flags
    c_oflag: u32,    // output flags
    c_cflag: u32,    // control flags
    c_lflag: u32,    // local flags
    c_cc: [u8; 32],  // control characters
}

pub fn sys_tcgetattr(fd: Fd, termios: &mut Termios) -> Result<(), Errno>;
pub fn sys_tcsetattr(fd: Fd, action: u32, termios: &Termios) -> Result<(), Errno>;
```

現有 `Console::raw` 模式擴充為完整 Termios，cooked 模式對應 `ICANON | ECHO`。

### VFS / 掛載 (M9)

```rust
pub fn sys_mount(source: &str, target: &str, fstype: &str, flags: usize) -> Result<(), Errno>;
pub fn sys_umount(target: &str) -> Result<(), Errno>;
```

新增 VFS 抽象層：

```rust
trait Filesystem {
    fn name(&self) -> &str;
    fn root(&self) -> Inode;
    fn mount(source: &str, flags: usize) -> Result<Self, Errno> where Self: Sized;
}

struct MountTable {
    mounts: Vec<Mount>,         // each mount binds a path prefix → Filesystem
}

struct Mount {
    path: String,
    fs: Box<dyn Filesystem>,
    root: Inode,
}
```

初始實作僅支援 xv7 原生 FS（`sfs`），未來可加入 `devfs`、`tmpfs`、`procfs`。

### 穩定版 (M10)

- 所有 syscall 回歸測試全數通過
- 系統呼叫編號最終凍結，不再變動
- 長時間多核壓力測試（fork/exec/pipe 循環、記憶體壓力）
- 檔案系統日誌復原測試
- 效能基準測試（syscall latency、context switch 成本）
- 移植 Rust 實作的 `coreutils`（cat、ls、echo 等）驗證 API 實用性

## 測試策略

| 測試類型 | 工具 | 說明 |
|----------|------|------|
| 核心測試 | 擴充 `_testrunner` | 每階段新增對應測試 |
| POSIX 相容 | 移植 OpenBSD `regress` 子集 | 或自行撰寫小測試 |
| libc 測試 | 移植 `scc` (Simple C Compiler) | C 編譯器自我編譯驗證 |

## 程式碼目錄結構變更

```
xv8-rust-posix/
├── kernel/src/
│   ├── posix/           # POSIX 相容層（新）
│   │   ├── mod.rs
│   │   ├── signal.rs
│   │   ├── time.rs
│   │   ├── socket.rs
│   │   ├── termios.rs
│   │   └── vfs.rs
│   ├── syscall.rs       # 擴充 dispatch
│   ├── proc.rs          # 擴充 signal、uid/gid
│   ├── file.rs          # 擴充 lseek、fcntl
│   ├── fs.rs            # 擴充 symlink、mode、getdents
│   ├── vm.rs            # 擴充 mmap/munmap
│   └── ... (其餘沿用 xv7)
└── _doc/
    ├── plan.md           # 本文件
    └── posix.md          # POSIX API 清單
```

## 參考資料

- POSIX.1-2008 (IEEE Std 1003.1-2008)
- Linux man-pages (man7.org)
- xv6 原始實作 (MIT)
- xv7-rust-octopus (Boran Seckin)
