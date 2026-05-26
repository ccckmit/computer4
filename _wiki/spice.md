# SPICE

## 概述

SPICE (Simulation Program with Integrated Circuit Emphasis) 是積體電路模擬的業界標準，最初由柏克萊大學於 1970 年代開發。SPICE 可對類比電路進行 DC 分析（直流工作點）、AC 分析（小訊號頻率響應）與暫態分析 (Transient Analysis)。本專案的 `ruspice` crate 實作了一個精簡的 SPICE-like 類比電路模擬器。

## 電路模擬的基本概念

### 電路元件

| 元件 | 符號 | 關係式 |
|---|---|---|
| 電阻 (Resistor) | R | V = I × R |
| 電容 (Capacitor) | C | I = C × dV/dt |
| 電感 (Inductor) | L | V = L × dI/dt |
| 電壓源 (V source) | V | V = V0 (固定) 或 V(t) (時變) |
| 電流源 (I source) | I | I = I0 (固定) 或 I(t) (時變) |
| 二極體 (Diode) | D | I = Is × (e^(V/Vt) - 1) |

### 改良節點分析法 (MNA, Modified Nodal Analysis)

MNA 是 SPICE 的核心演算法：

1. 對每個節點（參考節點除外）建立 KCL 方程式
2. 對電壓源加入額外方程式
3. 形成線性方程組 Gx = b
4. 使用稀疏矩陣求解器解出節點電壓與分支電流

```
MNA 矩陣形式:
┌         ┐ ┌     ┐   ┌     ┐
│ G   B  │ │  v  │   │  i  │
│        │ │     │ = │     │
│ C   D  │ │  j  │   │  e  │
└         ┘ └     ┘   └     ┘

G: 電導矩陣 (conductance matrix)
B, C: 相依性矩陣
D: 電壓源貢獻
v: 節點電壓
j: 電壓源電流
i: 等效電流源
e: 電壓源值
```

### Newton-Raphson 迭代法

非線性元件（二極體、電晶體）需要迭代求解：

```
1. 猜測初始工作點 V0
2. 在 V0 線性化非線性元件: I(V) ≈ I(V0) + dI/dV × (V - V0)
3. 解線性方程組得到 V1
4. 若 |V1 - V0| < ε，收斂
5. 否則 V0 = V1，回到步驟 2
```

## ruspice：本專案的 SPICE 模擬器

`compiler/ruspice/` 使用 Rust 實作的類比電路模擬器。

### 架構

```
┌─────────────────────────────────────┐
│  Netlist (電路網表)                  │
│  R1 n1 n2 1k                        │
│  C1 n2 0 10u                        │
│  V1 n1 0 SIN(0 1 1k)               │
└────────────────┬────────────────────┘
                 │ 解析
┌────────────────▼────────────────────┐
│  Circuit 結構                        │
│  nodes: Vec<Node>                    │
│  elements: Vec<Element>             │
└────────────────┬────────────────────┘
                 │ 分析
┌────────────────▼────────────────────┐
│  Analyser                           │
│  dc_analysis()                      │
│  ac_analysis()                      │
│  transient_analysis()               │
└────────────────┬────────────────────┘
                 │ 輸出
┌────────────────▼────────────────────┐
│  輸出 (ASCII / SVG)                  │
│  波形圖、表格                        │
└─────────────────────────────────────┘
```

### 支援的分析類型

#### DC 分析 (直流分析)

計算電路在穩定狀態下的工作點：

```rust
pub fn dc_analysis(&self) -> Result<HashMap<String, f64>, String> {
    // 將電容視為開路，電感視為短路
    // 解 MNA 方程組
    // 若含非線性元件，使用 Newton-Raphson 迭代
}
```

輸出範例：
```
DC Analysis Results:
V(n1) = 5.000 V
V(n2) = 3.333 V
I(V1) = 1.667 mA
```

#### AC 分析 (交流小訊號分析)

計算電路在不同頻率下的響應（頻率掃描）：

```rust
pub fn ac_analysis(&self, start_freq: f64, stop_freq: f64, points: usize)
    -> Result<Vec<(f64, Complex<f64>)>>
{
    // 在 DC 工作點線性化非線性元件
    // 對每個頻率點求解複數 MNA 方程組
}
```

輸出範例：
```
AC Analysis:
Freq    | V(out)   | Phase
1.0 Hz  | 0.999 V  | -0.1°
10 Hz   | 0.990 V  | -5.7°
100 Hz  | 0.707 V  | -45.0°
1 kHz   | 0.095 V  | -84.3°
```

#### 暫態分析 (Transient Analysis)

計算電路隨時間變化的行為：

```rust
pub fn transient_analysis(&self, tstop: f64, timestep: f64)
    -> Result<Vec<(f64, HashMap<String, f64>)>>
{
    // 使用梯形積分法或後向歐拉法
    // 每個時間步解一次電路方程
    // 對時變源（SIN、PULSE、PWL）求值作為邊界條件
}
```

### 數值積分法

暫態分析需將時域微分方程離散化：

**後向歐拉法 (Backward Euler)：**
```
dV/dt ≈ (V(t) - V(t-Δt)) / Δt
```

**梯形法 (Trapezoidal)：**
```
dV/dt ≈ (2/Δt) × (V(t) - V(t-Δt)) - dV/dt(t-Δt)
```

梯形法精度更高，但可能產生數值振盪。

### 元件模型

#### 二極體模型

```rust
fn diode_current(vd: f64) -> (f64, f64) {
    let vt = 0.02585;  // 熱電壓 @ 300K
    let is = 1e-14;    // 飽和電流
    let id = is * (vd / vt).exp_m1();
    let gd = is / vt * (vd / vt).exp();  // 導納
    (id, gd)
}
```

### 相依套件

| 套件 | 用途 |
|---|---|
| `nalgebra` | 線性代數（矩陣求解） |
| `serde` + `serde_json` | 序列化/反序列化 |

## 使用範例

```rust
use ruspice::Circuit;

fn main() -> Result<(), String> {
    let mut circuit = Circuit::new();

    // 定義節點
    let n1 = circuit.add_node("n1");
    let n2 = circuit.add_node("n2");
    let gnd = circuit.add_node("0");

    // 加入元件
    circuit.add_resistor("R1", n1, n2, 1000.0);
    circuit.add_capacitor("C1", n2, gnd, 1e-6);
    circuit.add_voltage_source("V1", n1, gnd, 5.0);

    // DC 分析
    let dc = circuit.dc_analysis()?;
    println!("{:?}", dc);

    // 暫態分析
    let trans = circuit.transient_analysis(0.01, 1e-6)?;
    for (t, voltages) in trans.iter().step_by(100) {
        println!("t={:.6}s V(n2)={:.3}V", t, voltages["n2"]);
    }

    Ok(())
}
```

## SPICE vs 數位模擬

| 特性 | SPICE (ruspice) | ruHDL (數位) |
|---|---|---|
| 抽象層級 | 類比（連續電壓/電流） | 數位（0/1/X） |
| 狀態表示 | 實數值 (f64) | 三態 (L/H/X) |
| 求解方法 | MNA + Newton-Raphson | 事件驅動 |
| 時間步進 | 連續（需選擇 timestep） | 離散（時脈邊緣） |
| 非線性元件 | 二極體、電晶體 | 無 |
| 典型應用 | 放大器、濾波器、電源 | CPU、邏輯電路 |

## 相關檔案

- `eda/ruspice/src/lib.rs` — 模擬器核心
- `eda/ruspice/src/circuit.rs` — 電路結構與 MNA
- `eda/ruspice/examples/basic.rs` — 基本使用範例
- `eda/ruspice/run.sh` — 執行範例
- `eda/ruspice/test.sh` — 測試腳本

## 參考資料

- L. W. Nagel, "SPICE2: A Computer Program to Simulate Semiconductor Circuits", UC Berkeley, 1975
- 改良節點分析：https://en.wikipedia.org/wiki/Modified_nodal_analysis
- Newton-Raphson method：https://en.wikipedia.org/wiki/Newton%27s_method
- 梯形積分法：https://en.wikipedia.org/wiki/Trapezoidal_rule
