# xv8-rust-posix 版本規劃 (v1.1 ~ v1.9)

> v1.x = 非網路 POSIX 功能。網路全部歸入 v2.x。

---

## v1.1 — POSIX 工具整合 + Job Control

> 將 os/posix/tools 整合進 xv8，加入基本工作控制。

### 功能項目

- [ ] `os/posix/tools` library 化（無 std，支援交叉編譯）
- [ ] `xv8-rust-posix/tools/` — rustix shim + binary targets
- [ ] 第一批工具編譯並加入 fs.img：`cp`, `mv`, `chmod`, `chown`, `touch`, `head`, `tail`, `sort`, `uniq`, `cut`, `tr`, `wc`, `cat`, `mkdir`, `rm`, `ln`, `rmdir`
- [ ] `sys_wait3` / `sys_wait4` — 可取得子行程 resource usage
- [ ] Job Control — `SIGTTIN`/`SIGTTOU` 處理
- [ ] `sys_setpgid` / `sys_getpgrp` — 行程群組

### 工具（第一批，約 20 個）

| 工具 | 說明 |
|------|------|
| cp, mv | 檔案複製/搬移 |
| chmod, chown, touch | 屬性操作 |
| head, tail, sort, uniq, cut, tr, wc, cat | 文字處理 |
| mkdir, rm, ln, rmdir, readlink, symlink | 連結/目錄 |
| basename, dirname, realpath | 路徑處理 |

### 測試
- [ ] 所有新工具在 QEMU 中正確執行
- [ ] `sh` 背景工作（`cmd &`）正常運作

### 交付
20+ 工具可用，背景工作正常。

---

## v1.2 — 第二批 POSIX 工具 + 增強 VFS

### 工具（第二批，約 25 個）

| 工具 | 說明 |
|------|------|
| grep, sed, awk | 文字分析（核心工具） |
| find, xargs | 工具組合 |
| test, [ | 條件判斷 |
| expr, printf | 數值/格式化輸出 |
| env, nohup, nice, sleep | 程序/時間 |
| id, whoami, pwd, uname | 系統資訊 |
| stat, file | 檔案狀態 |
| dd | 區塊複製 |
| install | 安裝工具 |
| link, unlink | 連結操作 |
| tee | 資料分流 |

### 功能項目

- [ ] `tmpfs` — 基於記憶體的暫存檔案系統
- [ ] `devpts` — 虛擬終端機 pseudo-device
- [ ] `procfs` 增強 — `/proc/<pid>/` 更多欄位（cmdline, environ, maps...）

### 交付
50+ 工具可用，VFS 支援 tmpfs/devpts。

---

## v1.3 — 第三批 POSIX 工具

### 工具（第三批，約 25 個）

| 工具 | 說明 |
|------|------|
| tar, compress, zcat, gzip, gunzip | 封存/壓縮 |
| diff, cmp, comm | 檔案比較 |
| patch | 套用 patch |
| split, csplit | 檔案分割 |
| expand, unexpand | 空白處理 |
| fold, fmt | 文字格式化 |
| nl, column | 行列格式化 |
| join | 資料關聯 |
| paste | 資料合併 |
| sort -r, sort -k, sort -u | 進階排序 |
| od, hexdump | 二進位輸出 |

### 交付
75+ 工具可用。

---

## v1.4 — 第四批 POSIX 工具 + 執行緒支援

### 工具（第四批，約 25 個）

| 工具 | 說明 |
|------|------|
| kill, wait | 行程控制 |
| pwd -P, cd -L, cd -P | 路徑處理 |
| id -G, id -u, id -n | 擴充 id |
| du, df | 磁碟使用 |
| ls -l, ls -a, ls -R | 增強 ls |
| chmod -R, chown -R | 遞迴操作 |
| lsblk, lsof | 區塊/鎖定檔案 |
| md5sum, sha1sum, sha256sum, cksum | 雜湊運算 |
| base64, base32 | 編碼 |
| factor, primes（已有）, seq | 數列 |
| jot, rand | 亂數（可選） |

### 功能項目

- [ ] `sys_clone` — 建立執行緒（不同於 fork，共享記憶體空間）
- [ ] `pthread_create` / `pthread_exit` / `pthread_join`
- [ ] `pthread_mutex_init/lock/unlock/destroy`
- [ ] `pthread_cond_*`（可選）

### 交付
100+ 工具可用，pthread 基礎支援。

---

## v1.5 — 動態連結 + 第五批工具

### 工具（第五批，約 20 個）

| 工具 | 說明 |
|------|------|
| ar, ranlib | 靜態庫操作 |
| nm, objdump, strings, strip | 目標檔工具 |
| size, readelf | ELF 分析 |
| ld, ld.bfd, ld.gold | 連結器（可選） |
| make, yacc, lex, flex | 建構工具 |
| m4 | 巨集處理器 |
| diff3, sdiff | 差異比較 |
| pr | 格式化列印 |

### 功能項目

- [ ] ELF dynamic loader — 解析 .dynamic、PLT/GOT
- [ ] `ld.so` — 動態連結器/載入器
- [ ] `dlopen` / `dlsym` / `dlclose`
- [ ] 基礎 .so 函式庫

### 交付
120+ 工具可用，動態連結支援。

---

## v1.6 — 記憶體管理改進

### 功能項目

- [ ] `mmap(MAP_SHARED)` — 共享記憶體對應
- [ ] `msync()` — 記憶體同步到磁碟
- [ ] `madvise()` — 記憶體使用建議
- [ ] POSIX Shared Memory — `shm_open()` / `shm_unlink()`
- [ ] `mlock` / `munlock` — 鎖定記憶體
- [ ] `mremap` — 重新映射記憶體
- [ ] `brk` / `sbrk` 改進

### 工具
- [ ] `ipcs` — 顯示共享記憶體段
- [ ] `ipcrm` — 移除共享記憶體

### 交付
共享記憶體正常運作，mmap 支援 MAP_SHARED。

---

## v1.7 — 使用者管理 + 第六批工具

### 工具（第六批，約 10 個）

| 工具 | 說明 |
|------|------|
| su | 切換使用者 |
| sudo | 替代 su（可選） |
| passwd, chpasswd | 密碼管理 |
| chfn, chsh | 使用者資訊 |
| useradd, userdel, usermod | 帳號管理 |
| groupadd, groupdel, groupmod | 群組管理 |
| newgrp | 切換群組 |
| who, w | 目前使用者 |

### 功能項目

- [ ] `/etc/passwd` 格式與解析
- [ ] `/etc/group` 格式與解析
- [ ] `sys_getpwnam` / `sys_getpwuid`
- [ ] `sys_getgrnam` / `sys_getgrgid`
- [ ] Login — `login` 程式

### 交付
多使用者帳號系統，login 正常運作。

---

## v1.8 — System Startup 與 Init

### 功能項目

- [ ] `/etc/rc` 開機腳本
- [ ] `/etc/fstab` — 自動掛載設定
- [ ] System V init 風格 — `rcS`, `rc2`, `rc3` 等 runlevel
- [ ] `init` 改進 — 正確處理 `getty` 和 login
- [ ] `getty` / `mingetty` — 虛擬主控台
- [ ] 系統日誌 — `syslogd` / `logger`
- [ ] `crond` — 背景行程排程

### 工具
- [ ] `halt`, `reboot`, `shutdown` — 系統關機
- [ ] `dmesg` — 核心訊息
- [ ] `logsave` — 儲存日誌

### 交付
開機後自動進入 login prompt，支援 runlevel。

---

## v1.9 — SMP / Multi-core + 工具收尾

### 功能項目

- [ ] QEMU `-smp 2` 或更高核心數啟動
- [ ] Per-CPU 資料結構
- [ ] SMP-safe spinlock
- [ ] 核心間中斷（IPI）
- [ ] TLS (Thread-Local Storage)
- [ ] SMP 安全的 timer 中斷分發

### 工具（最後一批，視情況補齊）

| 工具 | 說明 |
|------|------|
| top | 行程監控（多核心顯示） |
| uptime | 顯示負載 |
| free | 記憶體使用 |
| watch | 循環執行 |

### 交付
`-smp 2` 啟動後，兩個核心同時運作穩定。

---

## 版本時間軸

```
v1.0 ─→ v1.1 ─→ v1.2 ─→ v1.3 ─→ v1.4 ─→ v1.5 ─→ v1.6 ─→ v1.7 ─→ v1.8 ─→ v1.9
核心     工具整合  工具+     工具+     工具+     動態連結  記憶體     使用者     開機      SMP
        +JobCtrl   VFS增強  第三批    第四批    +工具      管理      管理      程序     multi-core
```

---

## v2.x 網路功能（另行規劃）

所有網路相關功能移到 v2.x：

- v2.1 — BSD Socket API（TCP/UDP）
- v2.2 — 網路工具（telnet, nc, curl, wget...）
- v2.3 — DHCP + 網路設定
- v2.4 — DNS resolver
- v2.5 — HTTP/HTTPS client
- v2.6 — 網路服務（httpd, ftpd, nfs...）

---

## 備註

- 每個版本都以 `./test.sh` 全部通過為交付標準
- 工具數量為預估值，視實作情況調整
- v1.1 是最關鍵的起點：完成 rustix shim + 第一批工具整合