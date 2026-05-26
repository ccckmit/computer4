# ELF 格式

## 概述

ELF（Executable and Linkable Format，可執行與可鏈結格式）是 Linux、Unix 及多數類 Unix 系統的標準二進位檔案格式，也適用於裸機 RISC-V 系統。本專案的編譯器工具鏈（`rv4`、`objdump`）深度依賴 ELF 格式來載入、分析與執行二進位程式。

## ELF 檔案結構

一個 ELF 檔案由以下部分組成：

```
┌─────────────────────┐
│ ELF Header          │ ← 固定 64 位元組 (ELF64)
├─────────────────────┤
│ Program Headers     │ ← 載入資訊（連結器/載入器使用）
│ (optional)          │
├─────────────────────┤
│ Section Headers     │ ← 鏈結資訊（連結器/除錯器使用）
│ (optional)          │
├─────────────────────┤
│ Section Data        │ ← 程式碼、資料、符號表等
│ + String Tables     │
└─────────────────────┘
```

## 本專案的 ELF 實作：objdump

`compiler/objdump/` crate 提供完整的 ELF 解析器。

### ELF 標頭 (64 位元)

```rust
pub struct ElfHeader64 {
    pub magic: [u8; 4],        // 魔術數字: [0x7f, 'E', 'L', 'F']
    pub class: ElfClass,       // 32 或 64 位元
    pub endianness: Endianness, // Little 或 Big endian
    pub version: u8,           // ELF 版本（通常為 1）
    pub os_abi: u8,            // OS/ABI 識別
    pub abi_version: u8,
    pub e_type: u16,           // 檔案型別（可重定位、可執行、共享庫）
    pub e_machine: u16,        // 架構（386、x86-64、RISC-V、ARM 等）
    pub e_entry: u64,          // 程式進入點位址
    pub e_phoff: u64,          // Program header table 偏移
    pub e_shoff: u64,          // Section header table 偏移
    pub e_flags: u32,          // 處理器特定旗標
    pub e_ehsize: u16,         // ELF header 大小
    pub e_phentsize: u16,      // 每個 program header 大小
    pub e_phnum: u16,          // Program header 數量
    pub e_shentsize: u16,      // 每個 section header 大小
    pub e_shnum: u16,          // Section header 數量
    pub e_shstrndx: u16,       // Section name string table 索引
}
```

### 魔術數字檢查

所有 ELF 檔案以 4 個位元組開頭：

```rust
pub const ELF_MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
```

### 支援的架構

```rust
pub fn machine_name(machine: u16) -> &'static str {
    match machine {
        3   => "Intel 386",
        8   => "MIPS R3000",
        20  => "PowerPC",
        40  => "ARM 32-bit",
        62  => "AMD x86-64",
        183 => "ARM 64-bit",
        243 => "RISC-V",
        // ... 更多架構
    }
}
```

### 區段標頭

```rust
pub struct SectionHeader64 {
    pub sh_name: u32,       // 區段名稱（指向 string table 的索引）
    pub sh_type: SectionType, // 區段型別
    pub sh_flags: u64,      // 區段旗標（WRITE、ALLOC、EXECINSTR）
    pub sh_addr: u64,       // 虛擬位址
    pub sh_offset: u64,     // 在檔案中的偏移
    pub sh_size: u64,       // 區段大小
    pub sh_link: u32,       // 關聯的區段索引
    pub sh_info: u32,       // 額外資訊
    pub sh_addralign: u64,  // 對齊要求
    pub sh_entsize: u64,    // 若為固定大小表格的項目大小
}
```

### 區段型別

```rust
pub enum SectionType {
    Null,          // 無效區段
    ProgramBits,   // 程式碼/資料（.text, .data, .rodata）
    SymbolTable,   // 符號表 (.symtab)
    StringTable,   // 字串表 (.strtab, .shstrtab)
    Rela,          // 重定位資訊（含加數）
    SymbolTableHash, // 符號表雜湊
    Dynamic,       // 動態鏈結資訊
    Note,          // 附註
    NoBits,        // .bss（不佔檔案空間）
    Rel,           // 重定位資訊（不含加數）
    // ... 更多型別
}
```

### 符號表

```rust
pub struct Symbol64 {
    pub st_name: u32,     // 符號名稱（指向 string table）
    pub st_info: u8,      // 繫結 + 型別
    pub st_other: u8,     // 可見性
    pub st_shndx: u16,    // 所屬區段
    pub st_value: u64,    // 符號值（位址或偏移）
    pub st_size: u64,     // 符號大小
}
```

符號繫結：

```rust
pub enum SymbolBinding {
    Local,   // 區域符號（僅對所屬目標檔案可見）
    Global,  // 全域符號（可被其他目標檔案參照）
    Weak,    // 弱符號（可被同名的全域符號覆蓋）
}
```

符號型別：

```rust
pub enum SymbolType {
    NoType,   // 未指定
    Object,   // 資料物件（變數）
    Func,     // 函式
    Section,  // 區段符號
    File,     // 原始檔名
    Common,   // 通用符號
    TLS,      // 執行緒區域儲存
}
```

### Program Header（程式載入資訊）

```rust
pub struct ProgramHeader64 {
    pub p_type: u32,   // 區段型別（LOAD、DYNAMIC、INTERP 等）
    pub p_flags: u32,  // 存取權限（R、W、X）
    pub p_offset: u64, // 在檔案中的偏移
    pub p_vaddr: u64,  // 虛擬位址
    pub p_paddr: u64,  // 實體位址（通常等於虛擬位址）
    pub p_filesz: u64, // 檔案中的大小
    pub p_memsz: u64,  // 記憶體中的大小（>= p_filesz）
    pub p_align: u64,  // 對齊
}
```

## rv4 中的 ELF 載入

`compiler/rv4/src/elf.rs` 從 ELF 可執行檔中載入程式碼與資料：

```rust
pub fn run_elf(data: &[u8]) -> Result<i32, String> {
    let loaded = elf::load(data)?;
    let mut mem = rv4::memory::Memory::new();
    let entry = elf::apply_to_memory(&loaded, &mut mem)?;

    let mut vm = rv4::vm::Vm::new();
    vm.set_pc(entry);         // 設定程式進入點
    vm.set_rv32(detect_rv32(&loaded));  // 自動偵測 32/64 位元

    vm.run(&mut mem)?;
    Ok(vm.exit_code())
}
```

載入流程：
1. `elf::load(data)` — 解析 ELF 標頭與 program headers
2. `elf::apply_to_memory(&loaded, &mut mem)` — 將 LOAD 類型的 segment 載入到模擬器的記憶體空間
3. `detect_rv32(&loaded)` — 掃描程式碼判斷是 RV32 還是 RV64

### RV32/RV64 自動偵測

```rust
fn detect_rv32(loaded: &LoadedElf) -> bool {
    for seg in &loaded.segments {
        for i in (0..seg.data.len()).step_by(4).take(100) {
            if i + 4 <= seg.data.len() {
                let inst = u32::from_le_bytes([...]);
                // 若找到 64 位元專屬指令 opcode (0x1b, 0x3b) → RV64
                if inst & 0x7f == 0x1b || inst & 0x7f == 0x3b {
                    return false;
                }
            }
        }
    }
    true // 預設 RV32
}
```

偵測原理：
- RV64 新增了 `ADDIW`、`ADDW`、`LD`、`SD` 等指令
- 這些指令的 opcode 為 `0x1b`（運算-立即-寬）或 `0x3b`（運算-寬）
- RV32 沒有這些 opcode，可作為判別依據

## 與本專案其他元件的關係

### compiler 工具鏈

```
rustc4 (Rust → .ir)    → 產出 LLVM IR，不直接產生 ELF
rv4 (RISC-V 模擬器)    → 輸入 ELF，解析並載入執行
objdump (ELF 分析器)   → 輸入 ELF，輸出人類可讀的區段/符號資訊
```

### ELF 在 OS crate 中的角色

- **mini-riscv-os**：QEMU 載入 ELF 核心 (`-kernel os.elf`)
- **xv6-rust-octopus**：QEMU 載入 ELF (`-kernel target/.../octopos`)
- **rvboard4**：QEMU 載入 ELF 二進位

QEMU 的 `-kernel` 選項要求 ELF 格式可執行檔。

## 主要差異：objdump vs rv4 的 ELF 載入

| 特性 | objdump | rv4/elf.rs |
|---|---|---|
| 目的 | 分析與除錯 | 載入執行 |
| 解析深度 | 完整（區段、符號、program headers） | 僅 program headers（LOAD segment） |
| ELF32 支援 | 解析但回傳錯誤（僅處理 ELF64） | 完整（指令層級 32/64 偵測） |
| 依賴 | scroll、thiserror | 無（純手動解析） |
| 錯誤處理 | 完整錯誤枚舉 | 簡化 Result |
| 使用場景 | objdump CLI 工具 | rv4 模擬器內部 |

## 相關檔案

### objdump (ELF 解析器)
- `compiler/objdump/src/lib.rs` — 完整 ELF64 解析（642 行含測試）
- `compiler/objdump/src/main.rs` — CLI 介面（clap）

### rv4 (ELF 載入器)
- `compiler/rv4/src/elf.rs` — ELF 載入與記憶體映射
- `compiler/rv4/src/lib.rs` — run_elf() 進入點
- `compiler/rv4/src/vm.rs` — RISC-V 指令執行

## 參考資料

- ELF 規格 (TIS 1995)：https://refspecs.linuxbase.org/elf/elf.pdf
- Linux man page：`man 5 elf`
- RISC-V ELF psABI：https://github.com/riscv-non-isa/riscv-elf-psabi-doc
