# ruHDL

## 概述

ruHDL (Rust Hardware Description Language) 是一個純 Rust 的硬體描述函式庫，用於在軟體中模擬數位電路。不同於 VHDL 或 Verilog 等傳統 HDL，ruHDL 完全以 Rust 的型別系統和 trait 來描述電路結構，無需外部工具或專用語法。本專案的 EDA 子系統以 ruHDL 為基礎，向上延伸至邏輯合成與類比電路模擬。

## 核心概念

### 訊號 (Signal) 與電線 (Wire)

ruHDL 的基礎單元是 `WireRef` — 對 `Signal` 的共享參照：

```rust
pub struct Signal {
    pub name: String,
    pub level: Level,
}

pub enum Level {
    L,  // Low (0)
    H,  // High (1)
    X,  // Unknown (don't care)
}
```

三態邏輯 (L/H/X) 允許模擬未知狀態的傳播：
- X AND L = L（只要有一輸入為 L，AND 輸出為 L）
- X AND H = X（若另一輸入未知則輸出未知）
- X OR H = H（只要有一輸入為 H，OR 輸出為 H）
- X OR L = X

透過 `wire()` 函式建立新 Wire，`get()` 與 `set()` 讀取與寫入訊號值。

### 邏輯閘 (Gate)

ruHDL 使用 `binary_gate!` 巨集定義五種基本邏輯閘：

```rust
binary_gate!(And, and);
binary_gate!(Or, or);
binary_gate!(Xor, xor);
binary_gate!(Nand, nand);
binary_gate!(Nor, nor);
```

每個閘由三個 WireRef 組成（a、b 輸入，y 輸出），提供 `new()` 與 `eval()` 方法：
- `new(a, b, y)` — 連接輸入輸出 Wire
- `eval()` — 根據目前輸入計算輸出。只有在輸出改變時才更新，避免不必要的訊號傳播

反相器 `Not` 為獨立結構（僅一個輸入）。

### 加法器 (Adder)

ruHDL 從基本閘逐步建構出加法器：
- `HalfAdder` — 半加器（2 輸入 → sum + carry）
- `FullAdder` — 全加器（3 輸入 → sum + carry）
- `Adder4` — 4 位元漣波加法器
- `Adder8` — 8 位元漣波加法器
- `RippleAdder` — 可配置位元寬度的漣波加法器

### 順序邏輯 (Sequential)

- `DFF` — D 型正反器（D Flip-Flop），具備 clk/d/q 埠
- `Register` — 多 bit 暫存器（由多個 DFF 組成）
- `Counter` — 計數器（在時脈上升邊緣遞增）

### 多工器與解碼器 (Mux & Decoder)

- `Mux2` — 2 對 1 多工器
- `Mux4` — 4 對 1 多工器
- `Decoder2x4` — 2 對 4 解碼器

### 模擬引擎 (Sim)

`Sim` 結構是 ruHDL 的模擬核心，負責事件驅動的電路模擬：

```rust
pub struct Sim {
    pub time: u64,
    events: Vec<Event>,
}
```

運作方式：
1. 註冊所有電路元件及其互連關係
2. 事件佇列按時間排序
3. 每個時間點觸發所有對應事件（訊號變化）
4. 變化傳播到受影響的閘，更新其輸出
5. 持續直到無事件剩餘或達到指定時間

### CPU 模擬

ruHDL 內建一個教育用途的 CPU 模型：
- `CPU` 結構包含運算邏輯單元 (ALU)、暫存器檔案、控制單元
- `program_5factorial()` — 示範程式（計算 5!）
- 支援的指令集為簡化版，包含算術、載入/儲存、分支

## 訊號模擬的三態邏輯真值表

### AND
| AND | L | H | X |
|---|---|---|---|
| L | L | L | L |
| H | L | H | X |
| X | L | X | X |

### OR
| OR | L | H | X |
|---|---|---|---|
| L | L | H | X |
| H | H | H | H |
| X | X | H | X |

### XOR
| XOR | L | H | X |
|---|---|---|---|
| L | L | H | X |
| H | H | L | X |
| X | X | X | X |

### NOT
| IN | OUT |
|---|---|
| L | H |
| H | L |
| X | X |

## 視覺化 (Visualization)

`viz` 模組提供 ASCII 與動畫視覺化功能：
- `demo_adder4()` — 展示 4 位元加法器運作
- `animate_adder4()` — 逐步動畫顯示加法過程

## ruHDL 生態系統

### verilog4 — Verilog 轉換器

`compiler/verilog4/` 可將 Verilog 程式碼轉換為 ruHDL 模組：
- `parse_verilog(input: &str) -> Result<Ast>` — 解析 Verilog 原始碼
- `gen_ruhdl(ast: &Ast) -> String` — 產生對應的 Rust ruHDL 程式碼

### ic4 — IC 設計工具

`compiler/ic4/` 整合了 ruHDL 並加入：
- **邏輯合成：** 卡諾圖 (Karnaugh map)、Quine-McCluskey 演算法、技術映射 (technology mapping)
- **實體設計：** 晶片規劃 (floorplanning)、元件佈置 (placement)、繞線 (routing)
- **視覺化：** ASCII 與 SVG 格式輸出
- 可選 eframe GUI（需 `gui` feature）

### synthesis — 邏輯合成引擎

`compiler/synthesis/` 從 ruHDL 風格的模組描述出發：
- `Elaborator` — 將高階模組展開為閘級 netlist
- `Optimizer` — 套用布林代數化簡（冗余消除、共用子表達式）
- `TechMapper` — 將 netlist 映射到特定製程的標準元件庫
- 輸出格式：Verilog 或 DOT（Graphviz）

### ruspice — 類比電路模擬器

`compiler/ruspice/` 是 SPICE-like 的類比電路模擬器，與 ruHDL 的數位世界互補：
- **元件：** R（電阻）、C（電容）、L（電感）、V/I 源（電壓/電流源）、D（二極體）
- **分析類型：** DC（直流）、AC（交流小訊號）、Transient（暫態）
- **求解器：** 改良節點分析 (MNA)，使用 `nalgebra` 進行稀疏矩陣求解
- **輸出：** ASCII 與 SVG 波形視覺化

## prelude

ruHDL 提供 `prelude` 模組重新匯出所有常用項目，使用方式：

```rust
use ruhdl::prelude::*;
```

這會匯入所有邏輯閘、加法器、順序元件、多工器、CPU、Sim 以及訊號操作函式。

## 設計哲學

1. **純 Rust：** 無需外部 EDA 工具，所有電路都在 Rust 型別系統中表達
2. **教育優先：** 設計清晰可讀，適合教學與學習數位電路設計
3. **漸進式抽象：** 從單一邏輯閘 → 加法器 → ALU → CPU
4. **事件驅動模擬：** 僅在輸入變化時重新計算，效率較高
5. **三態邏輯：** 支援未知狀態傳播，更接近真實硬體行為

## 與傳統 HDL 的比較

| 特性 | Verilog/VHDL | ruHDL |
|---|---|---|
| 語法 | 專用語言 | Rust 標準語法 |
| 模擬 | 事件驅動模擬器 | Rust 模擬引擎 (Sim) |
| 合成 | 專用合成工具 | ic4 + synthesis crate |
| 型別安全 | 弱 | 強（Rust 編譯器檢查） |
| 類比支援 | Verilog-AMS | ruspice（獨立 crate） |
| 平台相依 | 需安裝工具鏈 | 只需 Rust 工具鏈 |
| 可測試性 | testbench | 標準 Rust #[test] |

## 相關檔案

- `eda/ruhdl/src/signal.rs` — Signal、Level、WireRef 定義
- `eda/ruhdl/src/gate.rs` — 邏輯閘實作（含 binary_gate! 巨集）
- `eda/ruhdl/src/sim.rs` — 事件驅動模擬引擎
- `eda/ruhdl/src/cpu.rs` — CPU 模型
- `eda/ruhdl/src/lib.rs` — prelude 重新匯出
- `eda/ruhdl/src/viz.rs` — 圖形化視覺化

## 參考資料

- ruHDL 設計概念類似於：https://github.com/drom/deil（數位電路 Rust 函式庫）
- 三態邏輯：IEEE 1164（標準多值邏輯系統）
- SPICE：https://bwrcs.eecs.berkeley.edu/Classes/icbook/SPICE/
