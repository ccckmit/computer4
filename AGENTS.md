# computer4

自製電腦系統 monorepo。每個子目錄都是**獨立的 Rust crate**（根目錄無 Cargo workspace）。僅能個別建置/測試。

## 建置與測試

```sh
cargo build              # 當前 crate（永不 --workspace）
cargo test               # 當前 crate
cargo run                # 有 main.rs 者可用
./test.sh                # 多數 crate 用此腳本（build + test）
./run.sh                 # GUI/媒體 crate 經常用此腳本
```

無 Cargo 的獨立 `rustc` crate：
```sh
cd compiler/py4 && rustc py4.rs -o py4 && ./py4
cd tool/regex4 && rustc regex4.rs -o regex4 && ./regex4
```

提交/推送：
```sh
./git.sh <msg> <branch>  # git add . && commit -m "$msg-$branch" && push
```

## Monorepo 地圖

| 目錄 | Crate | Edition | 說明 |
|---|---|---|---|
| **compiler/** | | | |
| `compiler/lli4/` | lli4 | 2021 | LLVM IR 直譯器 — `lli4::interpret()` |
| `compiler/rustc4/` | rustc4 | 2021 | Rust → LLVM IR 編譯器 — `rustc4::compile()` |
| `compiler/rv4/` | rv4 | 2021 | RISC-V 模擬器 (RV32I/RV64I/RV64GC) — `rv4::run_elf()` |
| `compiler/objdump/` | objdump_lib | 2021 | ELF 分析器 (clap CLI) |
| `compiler/py4/` | (standalone) | — | Python 直譯器 — `py4.rs` + `lib4.rs` |
| **database/** | | | |
| `database/db6/` | db6 | 2021 | 旗艦 KV+SQL+FTS+Msgq。REPL / server / gRPC。另有 [AGENTS.md](database/db6/AGENTS.md) |
| `database/sql4/` | sql4 | 2024 | SQLite-like，支援 CJK FTS |
| `database/btree/` | btree | 2024 | BTree 引擎（有 `test.sh`） |
| `database/lsm/` | lsm | 2021 | LSM-Tree 引擎（有 `test.sh`） |
| `database/fts/` | fts | 2021 | 全文檢索 |
| `database/swisstable/` | swisstable | 2021 | Swiss Table（有 examples/） |
| `database/patricia-trie/` | patricia-trie | 2024 | Patricia trie |
| `database/redblacktree/` | redblacktree | 2024 | LLRB 樹。另有 [AGENTS.md](database/redblacktree/AGENTS.md) |
| `database/inodefs/` | inodefs | 2021 | Inode 虛擬檔案系統 |
| **math4/** | math4rs | 2021 | 統計、繪圖、ndarray、代數、微積分、線性代數、幾何。另有 [AGENTS.md](math4/AGENTS.md) |
| **crypto/** | | | |
| `crypto/ssl4/` | ssl4 | 2021 | SSL/TLS (rustls + tokio-rustls) |
| `crypto/keygen/` | keygen | 2021 | RSA/ECDSA 金鑰與憑證 CLI 產生器 |
| **gui/** | | | |
| `gui/win4/` | win4 | 2021 | 視窗管理器 (eframe/egui) |
| `gui/game4/` | game4 | 2021 | 遊戲框架 — WebSocket server + JS 前端 |
| **web/** | | | |
| `web/browser4/` | browser4 | 2021 | 瀏覽器 (eframe + boa_engine JS) |
| `web/browser5/` | browser5 | 2021 | 瀏覽器，使用自製 xdom4/js4 — 區域路徑依賴 |
| `web/md4browser/` | md4browser | 2021 | Markdown 瀏覽器 (eframe) |
| `web/xdom4/` | xdom4 | 2021 | XML/DOM 函式庫（CSS 選擇器） |
| `web/js4/` | js4 | 2021 | JavaScript 引擎（tokenizer → AST → interpreter） |
| **media/** | | | |
| `media/jpeg/` | jpeg | 2021 | JPEG 編解碼器 (PPM↔JPEG) |
| `media/mp3/` | mpeg_codec | 2021 | MP3 解碼/編碼器 |
| `media/mpeg1/` | mpeg1_decoder | 2021 | MPEG-1 視訊解碼器（僅 stdlib） |
| `media/aplayer4/` | aplayer4 | 2024 | 音訊播放器 (rodio + crossterm TUI) |
| **eda/** | | | |
| `eda/verilog2rust/` | verilog2rust | 2021 | Verilog → Rust (rhdl) 轉換器 + rhdl 硬體描述函式庫 |
| `eda/ic4/` | ic4 | 2021 | IC 設計 — 合成、實體設計、視覺化 |
| `eda/synthesis/` | synthesis | 2021 | 邏輯合成（HDL→netlist→optimizer→techmap） |
| `eda/ruspice/` | ruspice | 2021 | SPICE-like 類比電路模擬器 |
| **os/** | | | |
| `os/mini-riscv-os/` | mini-riscv-os | 2021 | 最小 RISC-V OS 核心（`#![no_std]` staticlib，QEMU） |
| `os/rvboard4/` | rvboard4 | 2021 | RISC-V BSP + `os/rvboard4/simulator/` (SDL2 GUI 類比) |
| `os/xv6-rust-octopus/` | *workspace* | 2024 | xv6 移植：核心 + 使用者 + mkfs（nightly，QEMU） |
| `os/xv7-rust-octopus/` | *workspace* | 2024 | xv7 + 網路支援（TAP 設備） |
| `os/xv8-rust-posix/` | xv8 (kernel) + user | 2021 | POSIX 相容 xv7 進化版（nightly，QEMU）。另有 [AGENTS.md](os/xv8-rust-posix/AGENTS.md) |
| `os/posix/tools/` | tools | 2021 | **124+ POSIX 工具**（`sh`、`ls`、`diff`、`grep`、`awk` 等）。134 binary targets，214 tests。另有 [_doc/](os/posix/_doc/) 下多版本文件 |
| **tool/** | | | |
| `tool/lz4/` | lz4 | 2024 | LZ4 壓縮 |
| `tool/regex4/` | (standalone) | — | 正規表達式引擎 — `regex4.rs` |
| `tool/vi4/` | vi4 | 2021 | 終端機文字編輯器 (crossterm) |

## 慣例

- **無根 workspace** — 每個頂層 crate 有自己的 `Cargo.lock` 和 `target/`
- **例外：** `os/xv6-rust-octopus/` 和 `os/xv7-rust-octopus/` 各為 Cargo workspace（核心 + 使用者 + mkfs）
- Edition：多數 = 2021；`sql4`、`btree`、`patricia-trie`、`redblacktree`、`lz4`、`aplayer4` + octopos 核心/使用者/mkfs = 2024
- 原始碼註解使用繁體中文
- `#![allow(dead_code)]` 位於 `math4/src/lib.rs` 和 `db6/src/lib.rs`
- 無 CI/CD，無根目錄 `rust-toolchain.toml`（octopos 內部各自鎖定 nightly）
- `rustc` 獨立 crate：`compiler/py4/`（`py4.rs` + `lib4.rs`）、`tool/regex4/`（`regex4.rs`）

## 編譯器管線

`rustc4` 寫出 `.ir` → `lli4` 直譯 `.ir`

## browser5 注意事項

- 區域路徑依賴：`xdom4 = { path = "../xdom4" }`、`js4 = { path = "../js4" }`
- `run.sh`：`cargo test && RUST_BACKTRACE=1 cargo run`
- 使用 `scraper` 解析 HTML；`JsRuntime` 包裹 `js4` 執行 JS
- DOM API：`document.getElementById()`、`element.innerText`（可讀寫）

## Wiki 參考

`_wiki/` 目錄包含本專案領域知識的詳細說明，涵蓋 RISC-V、LLVM IR、ruHDL、LSM-Tree、全文檢索、Swiss Table、Patricia Trie、LLRB 樹、ELF 格式等主題。

## 各套件專屬指令檔

- [`math4/AGENTS.md`](math4/AGENTS.md) — NaN 處理、多項式升冪順序、R/JS 命名
- [`database/db6/AGENTS.md`](database/db6/AGENTS.md) — 架構、REPL 指令、引擎 trait
- [`database/redblacktree/AGENTS.md`](database/redblacktree/AGENTS.md) — CLI 用法、結構

## os/posix/tools/ 說明

此crate包含**124+ POSIX.1-2008工具**，原始碼在 `tools/src/bin/`（134個`.rs`檔），所有binary targets註冊於 `tools/Cargo.toml`。

### 建置與測試
```sh
cd os/posix/tools && cargo build && cargo test
```

### 文件
- `_doc/plan.md` — 完整專案規劃
- `_doc/todo2.md` — 剩餘工具狀態
- `_doc/v0.x.md` — 各版本詳細文件（v0.8 ∼ v0.19）
