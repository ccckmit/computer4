# HDL 與 EDA

## 概述

HDL (Hardware Description Language, 硬體描述語言) 是用於描述數位電路行為與結構的語言，常見的有 Verilog 與 VHDL。EDA (Electronic Design Automation, 電子設計自動化) 指使用軟體工具自動化 IC 設計流程的技術。本專案的 EDA 子系統涵蓋從高階描述到物理設計的完整流程，包含 ruhdl（硬體描述）、verilog4（Verilog 轉換）、ic4（IC 設計）、synthesis（邏輯合成）、ruspice（類比模擬）。

## HDL 的基本概念

### 硬體描述 vs 程式執行

不同於軟體程式的循序執行，硬體描述語言描述的是平行執行的邏輯電路：

```verilog
// Verilog: 兩條 always block 平行執行
always @(posedge clk) begin
    q <= d;          // 在時脈上升邊緣更新 q
end

always @(posedge clk) begin
    count <= count + 1;  // 計數器同樣在時脈邊緣更新
end
```

### 組合邏輯 vs 循序邏輯

| 特性 | 組合邏輯 | 循序邏輯 |
|---|---|---|
| 輸出決定於 | 目前輸入 | 目前輸入 + 過去狀態 |
| 記憶元件 | 無需 | D 型正反器 (DFF) |
| 程式碼風格 | `assign` / `always @(*)` | `always @(posedge clk)` |
| 範例 | 加法器、多工器、解碼器 | 計數器、暫存器、狀態機 |

### 三態邏輯

數位電路的訊號值：

```
0 (Low)  — 邏輯 0
1 (High) — 邏輯 1
X (Unknown) — 未知值
Z (High impedance) — 高阻抗（三態輸出）
```

本專案的 ruHDL 支援 L/H/X 三態。

## 邏輯合成

將高階 HDL 描述轉換為閘級 netlist：

```
HDL (Verilog/VHDL)
    │
    ▼
Elaboration (展開實體化)
    │
    ▼
邏輯最佳化 (Boolean minimization)
    │
    ▼
技術映射 (Technology mapping)
    │
    ▼
閘級 netlist (AND, OR, NOT, DFF ...)
```

### 布林代數最佳化

**卡諾圖 (Karnaugh Map)：** 用於 2~4 變數的邏輯化簡：

```
AB\C | 0 | 1
─────┼───┼───
 00  │ 0 │ 0
 01  │ 1 │ 1
 11  │ 1 │ 1
 10  │ 0 │ 0
→ F = A·B
```

**Quine-McCluskey 演算法：** 可程式化實作的邏輯化簡演算法，適用於多變數。

### 技術映射 (Technology Mapping)

將最佳化後的邏輯映射到特定製程的標準元件庫 (standard cell library)：

```
最佳化後: F = ¬(A·B + C·D)
    ↓ 映射到 2-input NAND 庫
實現: F = NAND(NAND(A,B), NAND(C,D))
```

## EDA 流程

```
┌─────────────┐
│ RTL 設計    │ ← Verilog/VHDL/ruHDL
└──────┬──────┘
       │
┌──────▼──────┐
│ 邏輯合成    │ ← synthesis crate
└──────┬──────┘
       │
┌──────▼──────┐
│ 實體設計    │
│ 晶片規劃    │ ← ic4 (floorplanning)
│ 元件佈置    │ ← ic4 (placement)
│ 繞線        │ ← ic4 (routing)
└──────┬──────┘
       │
┌──────▼──────┐
│ 模擬驗證    │
│ 數位模擬    │ ← ruhdl Sim
│ 類比模擬    │ ← ruspice (SPICE)
└──────┬──────┘
       │
┌──────▼──────┐
│ 輸出        │
│ Verilog/    │
│ SVG/ASCII   │
└─────────────┘
```

## 本專案的 EDA 元件

### ruhdl — 硬體描述函式庫

ruHDL 以 Rust 型別系統描述數位電路，無需專用語法。

```rust
use ruhdl::prelude::*;

// 建立邏輯閘
let a = wire("a");
let b = wire("b");
let y = wire("y");
let mut gate = And::new(a, b, y);
gate.eval();
```

### verilog4 — Verilog 轉換器

`compiler/verilog4/` 將 Verilog 解析並轉換為 ruHDL：

```rust
pub fn parse_verilog(input: &str) -> Result<Ast>;
pub fn gen_ruhdl(ast: &Ast) -> String;
```

### ic4 — IC 設計工具

`compiler/ic4/` 提供完整的 IC 設計功能：

- **邏輯合成：** 卡諾圖化簡、Quine-McCluskey 演算法
- **實體設計：** 晶片規劃 (floorplan)、元件佈置 (placement)、繞線 (routing)
- **視覺化：** ASCII 佈局圖與 SVG 輸出
- **可選 GUI：** `gui` feature 啟用 eframe 圖形介面

### synthesis — 邏輯合成引擎

`compiler/synthesis/` 從 HDL 描述到閘級 netlist 的獨立引擎：

```rust
// 合成引擎
pub struct Synthesizer {
    elaborator: Elaborator,    // HDL → netlist
    optimizer: Optimizer,      // 布林最佳化
    tech_mapper: TechMapper,   // 技術映射
}
```

### ruspice — 類比模擬

`compiler/ruspice/` 實作 SPICE-like 類比電路模擬（詳見〈SPICE〉專文）。

## 標準元件庫 (Standard Cell Library)

邏輯合成後需要映射到實際可製造的元件：

```verilog
// 標準元件庫範例
module NAND2 (input A, B, output Y);
    assign Y = ~(A & B);
endmodule

module DFF (input CLK, D, output Q, QN);
    // D 型正反器
endmodule

module FA (input A, B, CI, output S, CO);
    // 全加器
endmodule
```

## 相關檔案

- `eda/ruhdl/src/` — ruHDL 核心（訊號、閘、模擬、CPU）
- `eda/verilog4/src/lib.rs` — Verilog 解析與轉換
- `eda/ic4/src/` — IC 設計（合成、實體設計、視覺化）
- `eda/synthesis/src/` — 邏輯合成引擎
- `eda/ruspice/src/` — 類比電路模擬

## 參考資料

- IEEE 1364 (Verilog 標準)
- IEEE 1076 (VHDL 標準)
- Synthesis of Digital Circuits：https://en.wikipedia.org/wiki/Logic_synthesis
- Andrew B. Kahng, *VLSI Physical Design: From Graph Partitioning to Timing Closure*
