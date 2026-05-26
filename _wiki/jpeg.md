# JPEG

## 概述

JPEG (Joint Photographic Experts Group) 是目前最廣泛使用的影像壓縮標準，專門設計用於自然影像（照片、漸層圖）。JPEG 屬於有損壓縮 (lossy compression)，透過犧牲人眼不易察覺的細節來達到高壓縮比（通常 10:1 ~ 20:1 無明顯品質損失）。本專案包含完整的 JPEG 編碼器與解碼器，位於 `media/jpeg/` crate。

## JPEG 壓縮流程

```
原始 RGB 影像
    │
    ▼
色彩空間轉換: RGB → YCbCr
    │
    ▼
色度取樣: 4:2:0（降低色度解析度）
    │
    ▼
區塊分割: 8×8 區塊
    │
    ▼
離散餘弦轉換 (DCT): 空間域 → 頻率域
    │
    ▼
量化 (Quantization): 捨棄高頻資訊
    │
    ▼
Zigzag 掃描: 將 8×8 矩陣轉為 1D 序列
    │
    ▼
熵編碼 (Huffman coding): 無失真壓縮
    │
    ▼
JPEG 二進位檔案
```

## 各步驟詳解

### 1. 色彩空間轉換: RGB → YCbCr

人眼對亮度變化比顏色變化更敏感。JPEG 利用此特性，將 RGB 轉換為 YCbCr：

```
Y  =  0.299R + 0.587G + 0.114B   (亮度)
Cb = -0.169R - 0.331G + 0.500B   (藍色色差)
Cr =  0.500R - 0.419G - 0.081B   (紅色色差)
```

本專案的實作：

```rust
fn ycbcr_to_rgb(y: u8, cb: u8, cr: u8) -> (u8, u8, u8) {
    let y_ = y as i16;
    let cb_ = cb as i16 - 128;
    let cr_ = cr as i16 - 128;
    let r = (y_ + (cr_ * 359 + 128) >> 8).clamp(0, 255) as u8;
    let g = (y_ - (cb_ * 88 + cr_ * 183 + 128) >> 8).clamp(0, 255) as u8;
    let b = (y_ + (cb_ * 454 + 128) >> 8).clamp(0, 255) as u8;
    (r, g, b)
}
```

### 2. 色度取樣 (Chroma Subsampling)

人眼對色度變化的敏感度較低，可以降低色度的解析度：

- **4:4:4：** 無壓縮（每個像素保留完整 YCbCr）
- **4:2:2：** 水平方向減半
- **4:2:0：** 水平與垂直方向各減半（最常見）

本專案使用 4:2:0（每 2×2 像素共用一組 CbCr）。

### 3. 區塊分割

影像被分割為 8×8 像素區塊。若邊界不足，需填補 (padding)：

```rust
let mcu_w = MAX_H * 8;  // 最小編碼單元 (MCU) 寬度
let mcu_h = MAX_V * 8;  // MCU 高度
```

### 4. 離散餘弦轉換 (DCT)

將空間域的像素值轉換為頻率域的係數：

```
正向 DCT (8×8):
F(u,v) = (1/4) × C(u) × C(v) × Σ_x Σ_y f(x,y) × cos((2x+1)uπ/16) × cos((2y+1)vπ/16)

其中 C(0) = 1/√2, C(n≠0) = 1
```

DCT 的意義：
- **DC 係數 (0,0)：** 區塊平均亮度（最低頻）
- **AC 係數 (u>0 或 v>0)：** 細節資訊（越高頻越細微）

```rust
fn forward_dct(block: &[[i16; 8]; 8]) -> [[f64; 8]; 8] {
    let mut out = [[0.0f64; 8]; 8];
    for u in 0..8 {
        for v in 0..8 {
            let mut sum = 0.0;
            for x in 0..8 {
                for y in 0..8 {
                    let px = block[y][x] as f64;
                    let cos_x = ((2 * x + 1) as f64 * u as f64 * std::f64::consts::PI / 16.0).cos();
                    let cos_y = ((2 * y + 1) as f64 * v as f64 * std::f64::consts::PI / 16.0).cos();
                    sum += px * cos_x * cos_y;
                }
            }
            let cu = if u == 0 { 1.0 / 2.0_f64.sqrt() } else { 1.0 };
            let cv = if v == 0 { 1.0 / 2.0_f64.sqrt() } else { 1.0 };
            out[v][u] = 0.25 * cu * cv * sum;
        }
    }
    out
}
```

### 5. 量化 (Quantization)

量化是 JPEG 中唯一的有損步驟。每個 DCT 係數除以對應的量化值後四捨五入：

```rust
fn quantize(dct: &[[f64; 8]; 8], table: &QuantizationTable) -> [[i16; 8]; 8] {
    let mut out = [[0i16; 8]; 8];
    for y in 0..8 {
        for x in 0..8 {
            out[y][x] = (dct[y][x] / table[y][x] as f64).round() as i16;
        }
    }
    out
}
```

亮度量化表 (簡化)：

```
16, 11, 10, 16, 24,  40,  51,  61
12, 12, 14, 19, 26,  58,  60,  55
14, 13, 16, 24, 40,  57,  69,  56
14, 17, 22, 29, 51,  87,  80,  62
18, 22, 37, 56, 68,  109, 103, 77
24, 35, 55, 64, 81,  104, 113, 92
49, 64, 78, 87, 103, 121, 120, 101
72, 92, 95, 98, 112, 100, 103, 99
```

數值越大 → 捨棄越多高頻資訊 → 壓縮率越高但畫質越低。

品質因數 (Quality Factor, Q)：
- Q=100：量化表全為 1（無失真）
- Q=95：量化值 ×2
- Q=75：量化值 ×2（預設）
- Q=50：量化值 ×4
- Q=25：量化值 ×8
- Q=10：量化值 ×16（低品質）

### 6. Zigzag 掃描

將 8×8 量化後的係數矩陣轉換為 1D 序列，從低頻到高頻：

```
原始:          Zigzag:
[ 0,0  0,1 ...    0,0 → 0,1 → 0,2 → 1,1 → 0,3 ...
  1,0  1,1 ...
  ...
]
```

這樣做使得高頻係數（多為零）集中在序列尾部，提升壓縮效率。

```rust
const ZIGZAG: [(usize, usize); 64] = [
    (0,0),(0,1),(1,0),(2,0),(1,1),(0,2),(0,3),(1,2),
    (2,1),(3,0),(4,0),(3,1),(2,2),(1,3),(0,4),(0,5),
    (1,4),(2,3),(3,2),(4,1),(5,0),(6,0),(5,1),(4,2),
    // ... 63 個項目
];
```

### 7. 熵編碼 (Huffman Coding)

JPEG 支援兩種熵編碼：Huffman 編碼（預設）與算術編碼（較少使用）。

Huffman 編碼的步驟：
1. 將 DC 係數以差值編碼 (DPCM)：`diff = DC_current - DC_previous`
2. 將 AC 係數以行程長度編碼 (RLE)：`(skip_zeros, next_nonzero_value)`
3. 對差值/行程長度進行 Huffman 編碼

DC 差異的 Huffman 表：
```
類別 (bit 數)    差異範圍        Huffman 碼
0                0             00
1                -1, 1         010
2                -3..-2, 2..3  011
3                -7..-4, 4..7  100
...
```

## 本專案的 JPEG 實作

### JpegEncoder

```rust
pub struct JpegEncoder {
    width: usize,
    height: usize,
    y_quant: QuantizationTable,     // 亮度量化表
    c_quant: QuantizationTable,     // 色度量化表
    dc_y_table: HuffmanTable,       // DC 亮度 Huffman 表
    dc_c_table: HuffmanTable,       // DC 色度 Huffman 表
    ac_y_table: HuffmanTable,       // AC 亮度 Huffman 表
    ac_c_table: HuffmanTable,       // AC 色度 Huffman 表
}

impl JpegEncoder {
    pub fn new(width: usize, height: usize) -> Self;
    pub fn encode(&self, y_data: &[u8], cb_data: &[u8], cr_data: &[u8]) -> Vec<u8>;
}
```

編碼輸出：完整的 JPEG 檔案（含 SOI、APP0、DQT、SOF、DHT、SOS、EOI 等 marker）。

### JpegDecoder

```rust
pub struct JpegDecoder { /* ... */ }

impl JpegDecoder {
    pub fn new() -> Self;
    pub fn decode(&mut self, data: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), String>;
}
```

解碼輸出：三個分量的 Y、Cb、Cr 資料。

### 支援的 JPEG 子集

- Baseline JPEG（非 progressive）
- 灰階（1 component）與 YCbCr（3 components）
- 4:2:0、4:2:2、4:4:4 取樣
- 標準 Huffman 表

## 壓縮率範例

| 品質因數 | 壓縮比 | 檔案大小 (1920×1080) | 視覺品質 |
|---|---|---|---|
| 無壓縮 (BMP) | 1:1 | 5.9 MB | 原始 |
| Q=95 | ~5:1 | 1.2 MB | 幾乎無失真 |
| Q=75 | ~15:1 | 400 KB | 良好 |
| Q=50 | ~25:1 | 240 KB | 可接受 |
| Q=25 | ~40:1 | 150 KB | 有明顯瑕疵 |
| Q=10 | ~60:1 | 100 KB | 低品質 |

## 相關檔案

- `media/jpeg/src/lib.rs` — JPEG 影像結構與公用函式
- `media/jpeg/src/encoder.rs` — 編碼器 (267 行)
- `media/jpeg/src/decoder.rs` — 解碼器 (330 行)
- `media/jpeg/src/main.rs` — CLI 工具

## 參考資料

- ITU-T T.81 | ISO/IEC 10918-1 (JPEG 標準)
- JPEG 教學：https://en.wikipedia.org/wiki/JPEG
- 離散餘弦轉換：https://en.wikipedia.org/wiki/Discrete_cosine_transform
- Huffman coding：https://en.wikipedia.org/wiki/Huffman_coding
