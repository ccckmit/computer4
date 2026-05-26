# 中間碼 (Intermediate Representation, IR)

## 概述

中間表示式 (IR) 是編譯器中位於原始語言與目標語言之間的抽象程式表示。IR 的設計目標是平衡兩個需求：足夠高階以保留原始語言的語意資訊（便於最佳化），足夠低階以接近目標機器（便於碼生成）。本專案的 IR 採用 LLVM IR 的子集，作為 rustc4（編譯器）與 lli4（直譯器）之間的橋樑。

## IR 的種類

### 高階 IR (HIR / High-Level IR)

接近原始語言，保留結構體、類別、控制流等高階結構。例如 GCC 的 GENERIC、Rust 的 HIR。

### 中階 IR (MIR / Mid-Level IR)

三地址碼 (three-address code) 形式，每個指令執行單一操作。控制流以基本區塊與 CFG 表示。例如 LLVM IR、GCC 的 GIMPLE。

### 低階 IR (LIR / Low-Level IR)

接近目標機器，考慮暫存器配置與指令排程。例如 LLVM MachineInstr、Rust 的後端 IR。

本專案的 IR 屬於中階 IR。

## 三地址碼 (Three-Address Code)

IR 的基本形式，每個指令最多包含三個運算元：

```
result = operand1 OP operand2
```

範例：
```
%c = add i32 %a, %b    ; c = a + b
%d = mul i32 %c, 2     ; d = c * 2
store i32 %d, i32* %ptr  ; *ptr = d
```

## SSA 形式 (Static Single Assignment)

SSA 是 IR 的重要屬性：**每個變數僅被賦值一次**。

### 優點
- def-use 鏈（定義-使用鏈）隱含在變數名稱中
- 簡化資料流分析（可達定義、活躍變數）
- 簡化最佳化（常數傳播、死碼消除）

### 範例

非 SSA：
```
x = 1
x = x + 2
y = x * 3
```

SSA：
```
x1 = 1
x2 = add x1, 2
y1 = mul x2, 3
```

### Phi 節點

控制流匯合處需要 phi 節點來合併來自不同路徑的值：

```
entry:
  br i1 %cond, label %then, label %else

then:
  %x1 = add i32 0, 1
  br label %merge

else:
  %x2 = sub i32 0, 1
  br label %merge

merge:
  %x3 = phi i32 [%x1, %then], [%x2, %else]
  ret i32 %x3
```

phi 節點語意：根據控制來自哪個前驅基本區塊，選取對應的值。

本專案的 LLVM IR 實作不支援 phi 節點。

## 本專案的 IR 設計

`compiler/lli4/src/ir.rs` 定義了 IR 的資料結構。

### 型別系統

```rust
pub enum LlvmType {
    I8,                        // 8 位元整數
    I32,                       // 32 位元整數
    I1,                        // 1 位元布林
    Void,                      // 無型別
    Pointer(Box<LlvmType>),    // 指標
    Array(u64, Box<LlvmType>), // 陣列 [N x T]
}
```

### 運算元

```rust
pub enum Operand {
    Int(i64),         // 整數立即值
    Bool(bool),       // 布林立即值
    Local(String),    // 區域變數 (%name)
    Global(String),   // 全域變數 (@name)
}
```

### 指令集

| 指令 | 說明 | 語意 |
|---|---|---|
| `Alloca` | 堆疊分配 | `%ptr = alloca i32` |
| `Store` | 記憶體寫入 | `store i32 %val, i32* %ptr` |
| `Load` | 記憶體讀取 | `%val = load i32, i32* %ptr` |
| `Add/Sub/Mul` | 算術 | `%r = add i32 %a, %b` |
| `SDiv/SRem` | 除法/餘數 | `%r = sdiv i32 %a, %b` |
| `ICmp` | 整數比較 | `%r = icmp slt i32 %a, %b` |
| `And/Or/Xor` | 位元運算 | `%r = and i32 %a, %b` |
| `Call` | 函式呼叫 | `%r = call i32 @foo(%a)` |
| `Ret` | 回傳 | `ret i32 %r` |
| `Br` | 無條件分支 | `br label %target` |
| `BrCond` | 條件分支 | `br i1 %cond, label %t, label %f` |
| `GetElementPtr` | 指標運算 | `%r = getelementptr [5 x i32], %ptr, i32 0, i32 %idx` |

### 與標準 LLVM IR 的差異

| 特性 | 標準 LLVM IR | 本專案 IR |
|---|---|---|
| 型別 | i8/i16/i32/i64/f32/f64/ptr/vec/struct/array | i8/i32/i1/ptr/array, 無浮點 |
| SSA | 強制 | 選擇性（名稱可覆蓋） |
| Phi 節點 | 支援 | 不支援 |
| 例外處理 | landingpad/invoke/resume | 不支援 |
| 內聯組合語言 | inline asm | 不支援 |
| Metadata | !dbg, !prof 等 | 不支援 |
| volatile/atomic | load/store 屬性 | 不支援 |
| 對齊屬性 | load/store align | 不支援 |
| getelementptr | 完整（結構+陣列+指標） | 僅陣列索引 |

## IR 文字格式

本專案的 IR 以文字格式儲存在 `.ir` 檔案中，易於閱讀與除錯：

```llvm
@msg = global [13 x i8] c"Hello, World\00"

define i32 @main() {
entry:
    %msg_ptr = getelementptr [13 x i8], [13 x i8]* @msg, i32 0, i32 0
    call void @puts(i8* %msg_ptr)
    ret i32 0
}

declare void @puts(i8*)
```

格式規則：
- 註解以 `;` 開頭
- 全域符號以 `@` 前綴
- 區域符號以 `%` 前綴
- 基本區塊以 `label:` 標記
- 型別標註在運算元之前

## IR 的最佳化

標準 LLVM 的最佳化 pass 舉例：

| Pass | 說明 |
|---|---|
| `-mem2reg` | 將 alloca/load/store 提升為 SSA 暫存器 |
| `-gvn` | 全域值編號 (Global Value Numbering) |
| `-instcombine` | 指令合併與簡化 |
| `-simplifycfg` | 簡化控制流圖 |
| `-licm` | 迴圈不變式外提 |
| `-dce` | 死碼消除 |

本專案的 IR 無最佳化階段。若要加入，需在 `codegen` 與 `interpret` 之間插入 pass。

## IR 的其他用途

除作為編譯器中間表示外，IR 還可用於：
- **靜態分析：** 安全性檢查、程式驗證
- **原始碼轉換：** 程式碼現代化、語言移植
- **連結時最佳化 (LTO)：** 跨編譯單元的最佳化
- **教育：** 學習編譯器原理（本專案的主要用途）

## 相關檔案

- `compiler/lli4/src/ir.rs` — IR 資料結構定義（65 行）
- `compiler/lli4/src/parser.rs` — IR 文字格式解析器
- `compiler/lli4/src/interp.rs` — IR 直譯器
- `compiler/rustc4/src/codegen.rs` — Rust → IR 生成器

## 參考資料

- LLVM Language Reference Manual：https://llvm.org/docs/LangRef.html
- 靜態單賦值 (SSA) 簡介：https://en.wikipedia.org/wiki/Static_single_assignment_form
- Appel, *Modern Compiler Implementation in ML/Java/C*
