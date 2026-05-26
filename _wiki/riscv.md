# RISC-V

## 概述

RISC-V 是一個開放原始碼的指令集架構 (ISA)，由加州大學柏克萊分校於 2010 年開發。與 ARM 和 x86 不同，RISC-V 完全免費且可自由實作，無需授權費用。本專案大量使用 RISC-V，涵蓋模擬器、作業系統核心、板級支援套件 (BSP) 以及 xv6/xv7 教學作業系統移植。

## 本專案的 RISC-V 元件

### 基本規格

- **XLEN 支援：** RV32I（32 位元）與 RV64I（64 位元）混合
- **擴充：** I（基礎整數）、M（整數乘除法）、A（原子操作）、F（單精度浮點）、D（雙精度浮點）、C（壓縮指令）
- **特權等級：** Machine (M)、Supervisor (S)、User (U)

### rv4 — RISC-V 模擬器

`compiler/rv4/` 是一個純 Rust 實作的 RISC-V 模擬器，入口為 `rv4::run_elf(data: &[u8])`。

**核心資料結構：**

```rust
pub struct Vm {
    regs: [u64; 32],    // 32 個通用暫存器（x0 恆為 0）
    pc: u64,             // 程式計數器
    is_rv32: bool,       // RV32 模式（暫存器截斷為 32 位元）
    ecall_exit: bool,    // 遇到 ecall 時是否退出
    exit_code: i32,      // 程式結束碼
}
```

**指令解碼：**
- 使用 RISC-V 壓縮指令擴充 (C)：前 2 位元組若最低 2 位元 ≠ 0x3，則視為壓縮指令（16 位元），否則為一般指令（32 位元）
- 支援指令：LUI、AUIPC、JAL、JALR、BEQ/BNE/BLT/BGE/BLTU/BGEU、LB/LH/LW/LBU/LHU/SB/SH/SW、ADDI/SLTI/SLTIU/ANDI/ORI/XORI、SLLI/SRLI/SRAI、ADD/SUB/SLL/SLT/SLTU/SRL/SRA/OR/AND、MUL/MULH/MULHSU/MULHU/DIV/DIVU/REM/REMU、ADDIW/SLLIW/SRLIW/SRAIW/ADDW/SUBW/SLLW/SRLW/SRAW/MULW/DIVW/DIVUW/REMW/REMUW

**系統呼叫：**
- 透過 ECALL 指令實作
- Linux-like 系統呼叫編號（ecall 觸發時，a7 暫存器存放系統呼叫號碼）
- 基本 IO（read/write/open/close/exit/brk）

**記憶體模型：**
- 自訂 `Memory` 結構，以 `HashMap<u64, u8>` 實作（稀疏記憶體）
- `load16/load32/load64` 與 `store16/store32/store64` 介面

**安全限制：**
- PC 上限 0x20000，防止 runaway 執行
- 指令執行上限 50000 步
- 對齊檢查（PC 必須為偶數）

### mini-riscv-os — 最小 RISC-V 核心

`os/mini-riscv-os/` 是一個極簡的 RISC-V 作業系統核心。

**建置特點：**
- 使用 `riscv32.json` 目標定義 (target.json)
- `#![no_std]` + `crate-type = ["staticlib"]`
- 組合語言啟動：`start.s`（開機進入點）、`sys.s`（系統呼叫中斷處理）
- 自訂鏈結腳本 `os.ld`
- `build.sh`：RUSTFLAGS 包含 `-C link-arg=-Tos.ld` 與 `-C target-cpu=generic-rv32`
- `run.sh`：透過 QEMU 執行，使用 `-machine virt -bios none -nographic` 等選項

### rvboard4 — RISC-V 板級支援

`os/rvboard4/` 為自製 RISC-V 開發板的 BSP。

**特色：**
- 同樣為 `#![no_std]` staticlib
- 提供 `hello()` 函式，透過 UART 輸出字串
- 包含啟動組合語言 `boot.S`
- `linker/` 目錄存放鏈結腳本
- 隨附模擬器 `os/rvboard4/simulator/`，依賴 SDL2，提供圖形化模擬環境

### xv6-rust-octopus — xv6 移植

`os/xv6-rust-octopus/` 是經典 xv6 教學作業系統的 Rust 移植版。

**架構：**
- Cargo workspace，包含三個 crate：kernel (`octopos`)、user (`user`)、mkfs
- 目標：RISC-V 64 位元（使用 nightly Rust）
- `rust-toolchain.toml` 鎖定特定 nightly 版本
- 使用者程式超過 15 個：init、sh、echo、cat、ls、rm、mkdir、ln、wc、kill 等

### xv7-rust-octopus — 具網路功能的 xv6

xv6 的增強版本，加入網路支援：
- 使用者程式包含 `udp`
- 測試二進位檔包含 `_net` 系列
- `setup_net.sh` 用於設定 TAP 網路設備

## RISC-V 在本專案的技術細節

### 目標 JSON

os 核心使用自訂 target.json 而非 Rust 內建的 riscv32imac-unknown-none-elf：

```json
{
    "arch": "riscv32",
    "cpu": "generic-rv32",
    "features": "+m,+a,+c",
    "llvm-target": "riscv32",
    "os": "none",
    "relocation-model": "static",
    "target-pointer-width": "32"
}
```

### QEMU 使用方式

所有 OS crate 透過 QEMU 執行。典型 flags：
- `-machine virt` — 使用 virt 開發板
- `-bios none` — 不使用 OpenSBI
- `-nographic` — 序列輸出
- `-kernel <elf>` — 載入核心
- `-m 256M` / `-m 128M` — 記憶體大小
- `-device virtio-net` — xv7 的網路設備

### 特色與限制

1. **自動 RV32/RV64 偵測：** rv4 的 `detect_rv32()` 透過掃描 ELF 中的指令對應，檢查是否存在 32 位元專屬指令模式（opcode 0x1b 或 0x3b）
2. **RV32 截斷模式：** 在 RV32 模式下，所有 GPR 寫入均截斷為 32 位元 (`v as u32 as u64`)
3. **壓縮指令：** 支援完整的 RVC（16 位元壓縮指令）解碼
4. **無 MMU：** mini-riscv-os 與 rvboard4 不使用分頁，直接實體記憶體存取
5. **SPIKE 相容：** xv6/xv7 使用 RustSBI（RISC-V Supervisor Binary Interface）作為 bootloader

## 相關檔案

- `compiler/rv4/src/vm.rs` — 指令解碼與執行主循環
- `compiler/rv4/src/memory.rs` — 稀疏記憶體模型
- `compiler/rv4/src/elf.rs` — ELF 載入器
- `os/mini-riscv-os/riscv32.json` — 目標定義
- `os/mini-riscv-os/start.s` — 組合語言啟動
- `os/mini-riscv-os/sys.s` — 系統呼叫中斷

## 參考資料

- RISC-V 規格書 v2.2：https://riscv.org/technical/specifications/
- xv6 原始 xv6：https://pdos.csail.mit.edu/6.828/
- RustSBI：https://github.com/rustsbi/rustsbi
