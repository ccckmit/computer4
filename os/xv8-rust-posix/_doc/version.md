# xv8-rust-posix 版本規劃

## v0.1 — 基礎移植

> 從 xv7 複製，更名為 xv8，核心可啟動並通過 xv7 既有測試。

- [x] 複製 xv7 原始碼，更名 crate（octopos → xv8）
- [x] 修正 QEMU `-cpu max` 相容問題
- [x] 確認 `./test.sh` 全部通過（7 tests）
- [x] 修改開機訊息為 `xv8 kernel is booting`
- [x] 更新版權標頭、README
- [x] 撰寫 v0.1 版本記錄 `_doc/v0.1.md`

**交付：** `cargo run --release` 可啟動 shell，`./test.sh` PASS。

---

## v0.2 — POSIX errno + 系統呼叫重編

> 導入 POSIX errno，重新定義系統呼叫編號與引數慣例。

- [x] errno 取代 xv7 自訂 `SysError`
- [x] 系統呼叫編號對齊 POSIX 標準（或自訂但公開且穩定）
- [x] 統一回傳慣例：`a0 = 0` 成功，`a0 < 0` 為 `-errno`
- [x] 新增 `syscall_table` 取代 match
- [x] 撰寫 POSIX 相容性測試（`_posix_base`）
- [x] 更新 `user/src/syscall.rs` 包裝層

**交付：** `_posix_base` 測試通過。xv7 舊使用者程式需修改才能編譯。

---

## v0.3 — 信號 (Signals)

> 核心信號框架，支援 SIGKILL/SIGTERM/SIGINT 等。

- [ ] `ProcInner` 新增信號欄位（`sigactions`、`pending`、`blocked`）
- [ ] 實作 `sys_sigaction`、`sys_sigprocmask`、`sys_sigpending`、`sys_sigsuspend`
- [ ] 實作信號遞送機制：trap 返回前檢查 `pending & ~blocked`
- [ ] `sys_kill` 改為傳送任意信號
- [ ] 預設動作處理（terminate / stop / ignore / core）
- [ ] 使用者測試程式：`_signal`（含 SIGALRM + alarm）
- [ ] `Ctrl-C` 改為透過 `SIGINT` 傳送

**交付：** `_signal` 測試通過。

---

## v0.4 — 檔案 I/O 擴充

> lseek、ftruncate、getdents、symlink、fcntl。

- [ ] `sys_lseek`：SEEK_SET / SEEK_CUR / SEEK_END
- [ ] `sys_ftruncate` / `sys_truncate`
- [ ] `sys_getdents`：目錄內容列舉
- [ ] `sys_symlink` / `sys_readlink`
- [ ] `sys_dup2`、`sys_fcntl`（F_DUPFD、F_GETFD、F_SETFD、F_GETFL）
- [ ] `sys_access`（F_OK / R_OK / W_OK / X_OK）
- [ ] Inode 新增 `mode` 欄位（`st_mode` 位元遮罩）
- [ ] stat 結構擴充為完整 POSIX `struct stat`

**交付：** `_posix_file` 測試通過，支援 `ls -l`。

---

## v0.5 — 記憶體映射 (mmap)

> 核心記憶體管理標準化。

- [ ] `sys_mmap`：MAP_ANONYMOUS + MAP_PRIVATE
- [ ] `sys_munmap`
- [ ] `sys_mprotect`
- [ ] Sv39 頁表權限直接對應 PROT_READ / PROT_WRITE / PROT_EXEC
- [ ] 檔案映射（MAP_SHARED + fd 參數）— page fault 時從 inode 回填
- [ ] 使用者堆積改用 mmap，libc `malloc` 基於 mmap
- [ ] `_posix_mmap` 測試

**交付：** `_posix_mmap` 測試通過。

---

## v0.6 — POSIX 行程 + 時間

> 行程群組、session、優先權、標準時間介面。

- [ ] `sys_setsid`、`sys_getpgid`、`sys_setpgid`、`sys_getppid`
- [ ] `sys_nice`
- [ ] `sys_waitpid`（WNOHANG / WUNTRACED）
- [ ] `sys_clock_gettime`（CLOCK_REALTIME + CLOCK_MONOTONIC）
- [ ] `sys_clock_settime`
- [ ] `sys_nanosleep`
- [ ] `sys_uname`
- [ ] `sys_getcwd`
- [ ] 核心內部 `TICKS` 改為 `Timespec`
- [ ] 使用者測試：`_posix_time`、`_posix_proc`

**交付：** `_posix_time` + `_posix_proc` 測試通過。

---

## v0.7 — 權限 + termios

> 使用者/群組模型、標準終端機控制。

- [ ] `ProcData` 新增 uid / gid / umask
- [ ] `sys_getuid`、`sys_geteuid`、`sys_getgid`、`sys_getegid`
- [ ] `sys_setuid`、`sys_setgid`
- [ ] `sys_chmod`、`sys_fchmod`、`sys_chown`、`sys_fchown`
- [ ] `sys_umask`
- [ ] `sys_tcgetattr`、`sys_tcsetattr`
- [ ] Termios 完整支援：ICANON、ECHO、ISIG 等
- [ ] 現有 Console raw/cooked 模式改接 termios 架構
- [ ] `isatty`、`ttyname`（libc 層）

**交付：** `_posix_termios` 測試通過。

---

## v0.8 — VFS + 掛載 (Mount)

> 虛擬檔案系統抽象層，支援多種 FS 類型。

- [ ] `Filesystem` trait：root()、name()、open()、readdir()…
- [ ] `MountTable`：路徑前綴 → filesystem 對應
- [ ] `sys_mount` / `sys_umount`
- [ ] 原生 FS 封裝為 `sfs` 模組
- [ ] 新增 `devfs`（/dev/console、/dev/null、/dev/random）
- [ ] 新增 `procfs`（/proc/self/…）
- [ ] 新增 `tmpfs`（/tmp）
- [ ] 根目錄掛載流程重新設計
- [ ] 裝置編號（dev_t）統一管理

**交付：** 可 mount procfs、devfs，`_posix_vfs` 測試通過。

---

## v1.0 — 穩定版

> POSIX 核心 API 完整，Rust 使用者程式可運用所有 POSIX 功能。

- [ ] 所有 syscall 回歸測試全數通過
- [ ] 系統呼叫編號最終凍結
- [ ] 核心 panic 路徑完整測試（OOM、無效 fd、權限錯誤…）
- [ ] 多核穩定性測試（長時間 fork/exec/pipe 壓力）
- [ ] 檔案系統一致性測試（斷電復原、日誌重播）
- [ ] 撰寫 xv8 POSIX API 文件（`man` 風格）
- [ ] 效能基準測試（syscall latency、context switch、disk throughput）
- [ ] 移植 `coreutils` Rust 實作（cat、ls、echo、rm、mkdir 等）以驗證 API

**交付：** `./test.sh` 全部通過 + 穩定性壓力測試 24 小時無故障。

---

## v1.1 — BSD Socket API

> 網路功能移至 v1.1，作為 v1.0 後的延伸。

- [ ] `sys_socket`（domain / type / protocol 標準引數）
- [ ] `sys_bind`
- [ ] `sys_connect`
- [ ] `sys_listen` / `sys_accept`（TCP 框架，可先回 ENOSYS）
- [ ] `sys_sendto` / `sys_recvfrom`（取代 xv7 send / receive）
- [ ] `sys_setsockopt` / `sys_getsockopt`
- [ ] `sys_shutdown`
- [ ] `struct sockaddr_in` 核心實作
- [ ] UDP 收發測試（`_posix_udp`）
- [ ] DHCP + shell `ping` 工具

**交付：** `_posix_udp` 測試通過，可在 xv8 內收發 UDP 封包。

---

## 版本時間軸

```
v0.1 ─→ v0.2 ─→ v0.3 ─→ v0.4 ─→ v0.5 ─→ v0.6 ─→ v0.7 ─→ v0.8 ─→ v1.0 ─→ v1.1
基礎     errno    信號     檔案I/O   mmap     行程+時間  權限+     VFS+     穩定版    Socket
         +編號              擴充                POSIX     termios  掛載               API
```

## 附註

- **向後相容：** v0.2 後舊 xv7 使用者程式須重新編譯，v0.2–v1.0 間系統呼叫編號一旦確定即凍結。
- **測試優先：** 每版本須包含對應的 `_testrunner` 測試，確保 `./test.sh` 全部通過。
- **命名：** 核心 crate 名稱從 `octopos` 改為 `xv8`；使用者 crate 從 `user` 改為 `libxv8` 或保留 `user`。
