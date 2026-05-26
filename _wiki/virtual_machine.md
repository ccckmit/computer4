# 虛擬機 (Virtual Machine)

## 概述

虛擬機 (VM) 是一種軟體實作的機器抽象層，模擬真實硬體或虛擬執行環境的行為。本專案包含兩台虛擬機：`rv4`（RISC-V 指令集模擬器）與 `lli4`（LLVM IR 直譯器）。此外，`js4` 也是一台 JavaScript 語言的虛擬機。

## 虛擬機的分類

### 系統虛擬機 (System VM)

模擬完整的硬體系統（CPU、記憶體、I/O 設備），可執行作業系統。例如 VMware、QEMU 全系統模擬、VirtualBox。

本專案：QEMU 用於執行 RISC-V OS crate（mini-riscv-os、rvboard4）。

### 行程虛擬機 (Process VM)

模擬單一程式的執行環境，提供與真實硬體不同的指令集或執行模型。例如 JVM (Java Virtual Machine)、CLR (.NET)、Wasm (WebAssembly)。

本專案：
- `rv4`：RISC-V 行程虛擬機，載入 ELF 可執行檔並以直譯方式執行 RISC-V 指令
- `lli4`：LLVM IR 虛擬機，以直譯方式執行 LLVM 中間碼
- `js4`：JavaScript 虛擬機，直譯執行 JavaScript AST

## rv4：RISC-V 虛擬機

`compiler/rv4/` 實作了一個完整的 RISC-V 行程虛擬機。

### 架構

```
┌─────────────────────────────────────┐
│  rv4::run_elf(elf_data)             │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ ELF 載入器   │ → .text, .data    │
│  └──────┬──────┘  載入到記憶體       │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ Vm 核心      │ ← 指令讀取/解碼/執行│
│  │  regs[32]    │                    │
│  │  pc          │                    │
│  │  memory      │                    │
│  └──────┬──────┘                    │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ 系統呼叫     │ ← ecall 處理       │
│  └─────────────┘                    │
└─────────────────────────────────────┘
```

### CPU 狀態

```rust
pub struct Vm {
    regs: [u64; 32],     // 32 個通用暫存器
    pc: u64,              // 程式計數器
    is_rv32: bool,        // RV32 模式（暫存器截斷）
    ecall_exit: bool,     // 系統呼叫退出旗標
    exit_code: i32,       // 退出碼
}
```

暫存器 x0 恆為 0（RISC-V 規格）。

### 指令執行循環

```
loop:
    1. 從 PC 指向的記憶體讀取指令
    2. 判斷為 16 位元壓縮指令或 32 位元一般指令
    3. 解碼 opcode、funct3、funct7、暫存器編號
    4. 執行對應操作
    5. 更新 PC
    6. 檢查邊界與步數限制
```

### 支援的指令集

- **RV32I / RV64I：** LUI、AUIPC、JAL、JALR、分支指令、載入/儲存、算術運算、移位、比較
- **RV64I 擴充：** ADDIW、SLLIW、SRLIW、SRAIW、ADDW、SUBW 等 64 位元專屬指令
- **RV64M：** MUL、MULH、DIV、REM 等乘除法
- **RV64C：** 壓縮指令（16 位元）
- **系統指令：** ECALL（系統呼叫）、EBREAK（除錯中斷）

### 記憶體模型

記憶體以 `HashMap<u64, u8>` 實作（稀疏儲存）：

```rust
pub struct Memory {
    data: HashMap<u64, u8>,
    // load8/load16/load32/load64
    // store8/store16/store32/store64
}
```

僅使用到的位址才會佔用空間，允許在有限的主機記憶體中模擬大位址空間。

### 系統呼叫

透過 RISC-V 的 ECALL 指令實作 Linux-like 系統呼叫：
- a7 暫存器存放系統呼叫編號
- a0-a5 存放參數
- 回傳值寫入 a0

支援的系統呼叫：read、write、open、close、exit、brk

### 安全限制

- PC 上限 0x20000
- 最大 50000 執行步數
- 未對齊 PC 檢查

## lli4：LLVM IR 虛擬機

`compiler/lli4/` 實作了一個執行 LLVM IR 的直譯式虛擬機。

### 架構

```
┌─────────────────────────────────────┐
│  lli4::interpret(ir_source)         │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ IR 解析器    │ → Program 結構     │
│  └──────┬──────┘                    │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ Interp 核心  │ ← 框架堆疊式執行   │
│  │  frames[]    │                     │
│  │  memory      │                    │
│  │  output      │                    │
│  └──────┬──────┘                    │
│         │                           │
│  ┌──────▼──────┐                    │
│  │ 內建函式     │ ← printf, putchar  │
│  └─────────────┘                     │
└─────────────────────────────────────┘
```

### 運作模型

不同於 rv4 的暫存器式模型，lli4 使用堆疊框架式執行：

```rust
struct Frame {
    func_idx: usize,       // 目前執行的函式
    block_idx: usize,      // 目前的基本區塊
    instr_idx: usize,      // 目前的指令位置
    locals: HashMap<String, i64>,  // 區域變數
}
```

1. 從 `main()` 函式開始
2. 每個函式呼叫建立新 Frame 並推入堆疊
3. 每個 frame 追蹤目前指令位置（類似程式計數器）
4. `Ret` 指令彈出 frame
5. Frame 堆疊為空時執行結束

### 記憶體管理

- **alloca：** 在 `memory` 雜湊表中分配新的整數位址（從 1 開始遞增）
- **store/load：** 直接讀寫 `HashMap<u64, i64>`
- 無垃圾回收（手動 alloc/free，但本實作無 free）

## 虛擬機效能比較

| 特性 | rv4 | lli4 | QEMU (使用者模式) | JVM |
|---|---|---|---|---|
| 執行對象 | RISC-V 機器碼 | LLVM IR | RISC-V 機器碼 | Java bytecode |
| 執行方式 | 逐指令直譯 | 逐指令直譯 | 二進位翻譯 (TCG) | JIT 編譯 |
| 原始層級 | 低（指令級） | 低（IR 級） | 低（指令級） | 中（位元組碼） |
| 啟動速度 | 快 | 快 | 中等 | 慢（需 JIT warmup） |
| 執行速度 | 慢（10^4-10^5 inst/s） | 慢 | 中等 | 快（接近原生） |
| 狀態模型 | 暫存器式 | 堆疊框架式 | 暫存器式 | 堆疊式 |
| 型別系統 | 無（原始位元組） | 有（i8/i32/i64） | 無 | 有（強型別） |

## js4：JavaScript 虛擬機

`web/js4/` 實作了一個 JavaScript 語言的直譯器，作為 browser5 的 JS 引擎。

### 管線

```
JavaScript 原始碼
    ↓  tokenizer
Token 串流
    ↓  parser (Pratt parsing)
AST (抽象語法樹)
    ↓  interpreter (樹走訪直譯)
執行結果
```

### 支援的特性
- let 變數宣告
- 函式定義與呼叫（含閉包）
- if/else、while 控制流
- try/catch 例外處理
- 陣列與物件字面值
- console.log 輸出
- 算術與邏輯運算

## 相關檔案

- `compiler/rv4/src/vm.rs` — RISC-V VM 核心（498 行）
- `compiler/rv4/src/memory.rs` — 稀疏記憶體模型
- `compiler/lli4/src/interp.rs` — LLVM IR 直譯器（308 行）
- `web/js4/src/` — JavaScript 直譯器

## 參考資料

- RISC-V 規格：https://riscv.org/technical/specifications/
- LLVM Language Reference：https://llvm.org/docs/LangRef.html
- James Smith, Ravi Nair, *Virtual Machines: Versatile Platforms for Systems and Processes*
- QEMU：https://www.qemu.org/
