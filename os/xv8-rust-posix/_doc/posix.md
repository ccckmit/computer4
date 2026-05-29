# POSIX API 清單 — xv8-rust-posix

> 以 POSIX.1-2008 (IEEE Std 1003.1-2008) 為基準，
> 篩選適用於 xv8 核心及使用者空間的 API。
>
> ✅ = 已實現（xv7 直接或接近可用）
> 🔧 = 需修改擴充
> ⬜ = 待新增
> ❌ = 不適用（硬體/架構限制）

---

## 1. 行程管理 (Process Management)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `fork()` | ✅ | `sys_fork` | 無修改 |
| `execve()` | 🔧 | `sys_exec` | 需加 `envp` 支援；保留 `exec` 別名 |
| `execv()` | 🔧 | `sys_exec` | 包裝 `execve` |
| `_exit()` | ✅ | `sys_exit` | 無修改 |
| `wait()` | ✅ | `sys_wait` | 無修改 |
| `waitpid()` | ⬜ | — | 需加 `WNOHANG` 等選項 |
| `waitid()` | ⬜ | — | 進階 wait |
| `getpid()` | ✅ | `sys_getpid` | 無修改 |
| `getppid()` | ⬜ | — | 新增 syscall |
| `getpgrp()` | ⬜ | — | 包裝 `getpgid(0)` |
| `getpgid()` | ⬜ | — | 新增 syscall |
| `setpgid()` | ⬜ | — | 新增 syscall |
| `setsid()` | ⬜ | — | 新增 syscall |
| `getsid()` | ⬜ | — | 新增 syscall |
| `nice()` | ⬜ | — | 新增 syscall |
| `kill()` | ✅ | `sys_kill` | 延伸為 POSIX 信號傳送 |
| `signal()` | ⬜ | — | XSI 相容，實作於 libc |
| `bsd_signal()` | ⬜ | — | 同上 |

### 排程

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `sched_yield()` | ⬜ | — | 包裝 `yield` 的 POSIX 版本 |
| `sched_get_priority_max/min()` | ⬜ | — | 若導入優先權排程 |
| `sleep()` | ✅ | `sys_sleep` | 通用別名 |
| `nanosleep()` | ⬜ | — | 新增高精度 sleep |

---

## 2. 信號 (Signals)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `sigaction()` | ⬜ | — | 核心 syscall，取代 `signal()` |
| `sigprocmask()` | ⬜ | — | 新增 syscall |
| `sigpending()` | ⬜ | — | 新增 syscall |
| `sigsuspend()` | ⬜ | — | 新增 syscall |
| `sigwait()` | ⬜ | — | 新增 syscall |
| `sigemptyset()` | ⬜ | — | libc 層巨集/函式 |
| `sigfillset()` | ⬜ | — | libc 層 |
| `sigaddset()` | ⬜ | — | libc 層 |
| `sigdelset()` | ⬜ | — | libc 層 |
| `sigismember()` | ⬜ | — | libc 層 |
| `kill()` | ✅ | `sys_kill` | 延伸支援信號編號 |
| `raise()` | ⬜ | — | libc: `kill(getpid(), sig)` |
| `alarm()` | ⬜ | — | 需核心定時器支援 |

### 信號常數

```
SIGHUP(1), SIGINT(2), SIGQUIT(3), SIGILL(4), SIGTRAP(5),
SIGABRT(6), SIGBUS(7), SIGFPE(8), SIGKILL(9), SIGUSR1(10),
SIGSEGV(11), SIGUSR2(12), SIGPIPE(13), SIGALRM(14),
SIGTERM(15), SIGCHLD(17), SIGCONT(18), SIGSTOP(19),
SIGTSTP(20), SIGTTIN(21), SIGTTOU(22)
```

預設動作：`SIGKILL`/`SIGSTOP` 不可捕捉/忽略；
`SIGCHLD`/`SIGCONT` 預設忽略；其餘預設終止行程。

---

## 3. 檔案 I/O (File I/O)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `open()` | ✅ | `sys_open` | 無修改 |
| `openat()` | ⬜ | — | 未來擴充 |
| `creat()` | ⬜ | — | libc: `open(path, O_CREAT\|O_WRONLY\|O_TRUNC, mode)` |
| `close()` | ✅ | `sys_close` | 無修改 |
| `read()` | ✅ | `sys_read` | 無修改 |
| `write()` | ✅ | `sys_write` | 無修改 |
| `pread()` | ⬜ | — | 新增（lock-free 定位讀） |
| `pwrite()` | ⬜ | — | 新增（lock-free 定位寫） |
| `lseek()` | ⬜ | — | 新增 syscall |
| `dup()` | ✅ | `sys_dup` | 無修改 |
| `dup2()` | ⬜ | — | 新增 |
| `fcntl()` | ⬜ | — | 新增 syscall（F_DUPFD, F_GETFD, F_SETFD 等） |
| `ioctl()` | ✅ | `sys_ioctl` | POSIX 化引數型別 |
| `fsync()` | ⬜ | — | 新增（flush log） |
| `fdatasync()` | ⬜ | — | 包裝 `fsync` |
| `sync()` | ⬜ | — | 全域 sync |
| `ftruncate()` | ⬜ | — | 新增 syscall |
| `truncate()` | ⬜ | — | 新增 syscall |
| `remove()` | ⬜ | — | libc: `unlink` 或 `rmdir` |
| `rename()` | ⬜ | — | 新增 |

### 開啟旗標

```
O_RDONLY(0), O_WRONLY(1), O_RDWR(2)
O_CREAT(0x40), O_EXCL(0x80), O_NOCTTY(0x100),
O_TRUNC(0x200), O_APPEND(0x400), O_NONBLOCK(0x800),
O_DIRECTORY(0x10000), O_NOFOLLOW(0x20000), O_CLOEXEC(0x40000)
```

---

## 4. 檔案系統 (Filesystem)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `stat()` | 🔧 | `sys_fstat` | 新增 `lstat`（by path） |
| `fstat()` | ✅ | `sys_fstat` | 擴充 `Stat` 結構 |
| `lstat()` | ⬜ | — | 不追隨 symlink |
| `mkdir()` | ✅ | `sys_mkdir` | 新增 `mode` 參數 |
| `rmdir()` | ⬜ | — | 新增 |
| `unlink()` | ✅ | `sys_unlink` | 無修改 |
| `link()` | ✅ | `sys_link` | 無修改 |
| `symlink()` | ⬜ | — | 新增 |
| `readlink()` | ⬜ | — | 新增 |
| `rename()` | ⬜ | — | 新增 |
| `chdir()` | ✅ | `sys_chdir` | 無修改 |
| `fchdir()` | ⬜ | — | 新增（以 fd 切換目錄） |
| `getcwd()` | ⬜ | — | 新增 |
| `chmod()` | ⬜ | — | 新增 |
| `fchmod()` | ⬜ | — | 新增 |
| `chown()` | ⬜ | — | 新增 |
| `fchown()` | ⬜ | — | 新增 |
| `lchown()` | ⬜ | — | 新增（不追隨 symlink） |
| `umask()` | ⬜ | — | 新增 |
| `mknod()` | ✅ | `sys_mknod` | POSIX 化引數 |
| `access()` | ⬜ | — | 新增 |
| `statvfs()` | ⬜ | — | 新增（filesystem info） |

### Stat 結構

```c
struct stat {
    dev_t     st_dev;      // device ID
    ino_t     st_ino;      // inode number
    mode_t    st_mode;     // file mode (S_IFMT 等)
    nlink_t   st_nlink;    // link count
    uid_t     st_uid;      // owner UID
    gid_t     st_gid;      // owner GID
    off_t     st_size;     // file size
    blksize_t st_blksize;  // block size (1024)
    blkcnt_t  st_blocks;   // block count
    struct timespec st_atim; // access time
    struct timespec st_mtim; // modify time
    struct timespec st_ctim; // change time
};
```

### 目錄操作

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `opendir()` | ⬜ | — | libc 包裝 |
| `readdir()` | ⬜ | — | 基於 `getdents` syscall |
| `rewinddir()` | ⬜ | — | libc |
| `closedir()` | ⬜ | — | libc |
| `getdents()` | ⬜ | — | 核心 syscall（非 POSIX 但 Linux 標準） |
| `scandir()` | ⬜ | — | libc 層 |

---

## 5. 記憶體管理 (Memory Management)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `sbrk()` | ✅ | `sys_sbrk` | 保留相容 |
| `brk()` | ⬜ | — | libc: 包裝 `sbrk` |
| `mmap()` | ⬜ | — | 新增 syscall |
| `munmap()` | ⬜ | — | 新增 syscall |
| `mprotect()` | ⬜ | — | 新增 syscall |
| `msync()` | ⬜ | — | 檔案映射需 flush |
| `mlock()` | ❌ | — | 非即時系統，實作優先權低 |
| `munlock()` | ❌ | — | 同上 |

### mmap 旗標

```
PROT_READ(1), PROT_WRITE(2), PROT_EXEC(4), PROT_NONE(0)
MAP_SHARED(1), MAP_PRIVATE(2), MAP_FIXED(0x10),
MAP_ANONYMOUS(0x20), MAP_GROWSDOWN(0x100)
```

---

## 6. 時間 (Time)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `time()` | 🔧 | `uptime` | 改回報 realtime 秒數 |
| `clock_gettime()` | ⬜ | — | 新增 |
| `clock_settime()` | ⬜ | — | 新增 |
| `clock_getres()` | ⬜ | — | 新增 |
| `gettimeofday()` | ⬜ | — | libc 包裝 `clock_gettime` |
| `nanosleep()` | ⬜ | — | 新增 syscall |
| `sleep()` | ✅ | `sys_sleep` | 可基於 `nanosleep` 實作 |

### 時鐘 ID

```
CLOCK_REALTIME(0), CLOCK_MONOTONIC(1),
CLOCK_PROCESS_CPUTIME_ID(2), CLOCK_THREAD_CPUTIME_ID(3)
```

---

## 7. 使用者/群組 (User & Group)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `getuid()` | ⬜ | — | 新增（初始回傳 0） |
| `geteuid()` | ⬜ | — | 同上 |
| `getgid()` | ⬜ | — | 同上 |
| `getegid()` | ⬜ | — | 同上 |
| `setuid()` | ⬜ | — | 新增（若無權限回傳 EPERM） |
| `setgid()` | ⬜ | — | 同上 |
| `getgroups()` | ⬜ | — | 新增 |
| `getlogin()` | ⬜ | — | libc 層 |

---

## 8. Socket / 網路 (POSIX.1-2008 規格)

> ⚠ v1.1 範圍。v1.0 不包含此章功能。

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `socket()` | ⬜ | `sys_socket` | POSIX 化（domain/type/protocol） |
| `bind()` | ⬜ | — | 新增 syscall |
| `connect()` | ⬜ | — | 新增 syscall |
| `listen()` | ⬜ | — | 新增（TCP 專用） |
| `accept()` | ⬜ | — | 新增（TCP 專用） |
| `sendto()` | ⬜ | `sys_send` | 改為標準 POSIX 簽名 |
| `recvfrom()` | ⬜ | `sys_receive` | 改為標準 POSIX 簽名 |
| `send()` | ⬜ | — | libc: `sendto(fd, buf, len, flags, NULL, 0)` |
| `recv()` | ⬜ | — | libc: `recvfrom(...)` |
| `sendmsg()` | ⬜ | — | 進階 |
| `recvmsg()` | ⬜ | — | 進階 |
| `shutdown()` | ⬜ | — | 新增 |
| `getsockname()` | ⬜ | — | 新增 |
| `getpeername()` | ⬜ | — | 新增 |
| `setsockopt()` | ⬜ | — | 新增 |
| `getsockopt()` | ⬜ | — | 新增 |
| `poll()` | ⬜ | — | I/O 多工 |
| `select()` | ⬜ | — | libc 基於 `poll` 實作 |

### Socket 常數

```
AF_UNIX(1), AF_INET(2)
SOCK_STREAM(1), SOCK_DGRAM(2), SOCK_RAW(3)
IPPROTO_TCP(6), IPPROTO_UDP(17), IPPROTO_IP(0)

struct sockaddr_in {
    sa_family_t sin_family;  // AF_INET
    in_port_t   sin_port;    // network byte order
    struct in_addr sin_addr; // network byte order
    char        sin_zero[8];
};
```

---

## 9. 終端機 I/O (Terminal I/O)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `tcgetattr()` | ⬜ | `ioctl(CONSOLE_SET_RAW)` | 新增完整 termios |
| `tcsetattr()` | ⬜ | — | 新增完整 termios |
| `tcsendbreak()` | ⬜ | — | 新增 |
| `tcdrain()` | ⬜ | — | 新增 |
| `tcflush()` | ⬜ | — | 新增 |
| `tcflow()` | ⬜ | — | 新增 |
| `cfmakeraw()` | ⬜ | — | libc 層 |
| `cfsetispeed()` | ⬜ | — | libc 層 |
| `cfsetospeed()` | ⬜ | — | libc 層 |
| `ttyname()` | ⬜ | — | 未來實作 |
| `isatty()` | ⬜ | — | 包裝 `tcgetattr` |

---

## 10. 裝置控制 (Device Control)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `ioctl()` | ✅ | `sys_ioctl` | POSIX 化常數命名 |
| `sysfs()` | ❌ | — | RISC-V 無 sysfs 概念 |

---

## 11. 系統資訊 (System Information)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `uname()` | ⬜ | — | 新增 syscall |
| `sysconf()` | ⬜ | — | libc 層 |
| `pathconf()` | ⬜ | — | libc 層 |
| `confstr()` | ⬜ | — | libc 層 |

### utsname 結構

```c
struct utsname {
    char sysname[65];    // "xv8"
    char nodename[65];   // hostname
    char release[65];    // kernel release
    char version[65];    // kernel version
    char machine[65];    // "riscv64"
};
```

---

## 12. 語言環境 (Language / Locale)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `setlocale()` | ⬜ | — | libc 層（最小實作） |
| `localeconv()` | ⬜ | — | libc 層 |

---

## 13. 正規表達式 (Regular Expressions)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `regcomp()` | ⬜ | — | libc 層（可連結 `regex4`） |
| `regexec()` | ⬜ | — | libc 層 |
| `regerror()` | ⬜ | — | libc 層 |
| `regfree()` | ⬜ | — | libc 層 |

---

## 14. 執行緒 (Threads — POSIX Threads)

> Pthread 為 POSIX.1-2008 選項（系統支援），
> 先以 `ENOSYS` 回應，列為未來工作。

| 函式 | xv8 狀態 | 說明 |
|------|----------|------|
| `pthread_create()` | ❌ | 未來：核心 thread 支援 |
| `pthread_join()` | ❌ | |
| `pthread_detach()` | ❌ | |
| `pthread_mutex_*()` | ❌ | |
| `pthread_cond_*()` | ❌ | |
| `pthread_key_*()` | ❌ | TLS |

---

## 15. IPC (Interprocess Communication)

| 函式 | xv8 狀態 | xv7 對應 | 說明 |
|------|----------|----------|------|
| `pipe()` | ✅ | `sys_pipe` | 無修改 |
| `pipe2()` | ⬜ | — | 新增（含 O_CLOEXEC/O_NONBLOCK） |
| `read()`/`write()` on pipe | ✅ | 既有 | 無修改 |
| `mkfifo()` | ⬜ | — | 新增（FIFO 特殊檔） |
| `ftok()` | ⬜ | — | libc 層 |
| `msgget()` | ❌ | — | System V IPC，非必要 |
| `msgsnd()` | ❌ | — | |
| `msgrcv()` | ❌ | — | |
| `semget()` | ❌ | — | |
| `semop()` | ❌ | — | |
| `shmget()` | ❌ | — | |
| `shmat()` | ❌ | — | |

---

---

## 優先實作順序（建議）

| 順位 | 類別 | 原因 |
|------|------|------|
| 1 | errno 標準化 | 所有 syscall 的基礎 |
| 2 | 信號框架 | 行程控制不可或缺 |
| 3 | lseek / ftruncate | 檔案操作完整性 |
| 4 | mmap / munmap | 記憶體管理標準介面 |
| 5 | getdents | 目錄列舉（ls 需要） |
| 6 | clock_gettime / nanosleep | 時間標準介面 |
| 7 | uid/gid + 權限 | 安全模型基礎 |
| 8 | termios | 終端機控制標準 |
| 9 | mount/umount + VFS | 檔案系統擴充性 |
