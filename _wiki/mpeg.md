# MPEG

## 概述

MPEG (Moving Picture Experts Group) 是動態影像壓縮的國際標準系列，由 ISO/IEC 制訂。MPEG 利用影片中時間域與空間域的冗餘資訊達到高效壓縮。本專案包含兩個 MPEG 相關 crate：`media/mpeg1/`（MPEG-1 視訊解碼器）與 `media/mp3/`（MPEG-1 Audio Layer III 編解碼器）。

## MPEG 標準系列

| 標準 | 正式名稱 | 說明 |
|---|---|---|
| MPEG-1 | ISO/IEC 11172 | VCD 品質 (352×240) |
| MPEG-2 | ISO/IEC 13818 | DVD、廣播電視 |
| MPEG-4 | ISO/IEC 14496 | DivX、Xvid、H.264 (AVC) |
| MPEG-7 | ISO/IEC 15938 | 多媒體內容描述 |
| MPEG-21 | ISO/IEC 21000 | 多媒體框架 |

## MPEG-1 視訊壓縮

### GOP (Group of Pictures)

MPEG-1 的畫面分為三種：

```
I  B  B  P  B  B  P  B  B  I  B  B  P
├── GOP ─────┤├── GOP ─────┤
```

| 畫面類型 | 編碼方式 | 是否參照其他畫面 | 壓縮率 |
|---|---|---|---|
| I-frame (Intra) | JPEG-like（僅空間壓縮） | 無 | 最低 |
| P-frame (Predictive) | 前向預測 | 參照前一個 I/P | 中 |
| B-frame (Bidirectional) | 雙向預測 | 參照前後 I/P | 最高 |

典型 GOP 結構：`I B B P B B P B B`（M=3, N=9）
- M = I/P 之間的 B 畫面數 + 1
- N = GOP 總長度

### 編碼流程

```
原始畫面序列
    │
    ▼
區塊分割: 16×16 巨集區塊 (Macroblock)
    │
    ▼
I-frame:       P/B-frame:
    │             │
    ▼             ▼
DCT         動態估計 (Motion Estimation)
量化           │
Zigzag      找到最佳匹配區塊
Huffman      │
    │         ▼
    │       計算殘差 (差畫面)
    │         │
    │         ▼
    │       DCT + 量化 + Zigzag + Huffman
    ▼             │
    └─────────────┘
        │
        ▼
    MPEG-1 位元串流
```

### 動態估計 (Motion Estimation)

在 P/B 畫面中，搜尋目前巨集區塊在前一畫面中的最佳匹配位置：

```rust
// 簡化版運動補償
fn motion_estimate(block: &[u8], reference: &[u8], search_range: i32) -> (i32, i32) {
    let mut best_mv = (0, 0);
    let mut best_cost = i32::MAX;
    for dy in -search_range..=search_range {
        for dx in -search_range..=search_range {
            let cost = sad(block, reference, dx, dy);
            if cost < best_cost {
                best_cost = cost;
                best_mv = (dx, dy);
            }
        }
    }
    best_mv // 運動向量
}
```

### 解碼器結構

`media/mpeg1/` 的模組結構：

```rust
// 模組分工
mod bitstream;  // 位元串流解析（起始碼偵測）
mod parser;     // 語法元素解析
mod vlc;        // 可變長度解碼 (Huffman tables)
mod idct;       // 反向 DCT
mod motion;     // 運動補償
mod frame;      // 畫面重組與顯示順序
mod decoder;    // 解碼器主控
```

### CLI 使用

```sh
cd media/mpeg1
cargo run input.mpg                    # 顯示影片資訊
cargo run input.mpg 0 frame0.ppm       # 提取第 0 幀為 PPM
cargo run input.mpg 100 frame100.ppm   # 提取第 100 幀
```

## MP3 (MPEG-1 Audio Layer III)

`media/mp3/` 實作 MPEG-1 Audio Layer III 編解碼器（crate 名為 `mpeg_codec`）。

### 音訊壓縮流程

```
PCM 音訊 (44.1kHz, 16-bit, stereo = 1411kbps)
    │
    ▼
子帶分析濾波器組 (Polyphase Filter Bank)
    │
    ▼
改良離散餘弦轉換 (MDCT)
    │
    ▼
心理聲學模型 (Psychoacoustic Model)
    │  ┌──────────────────────────────┐
    │  │ 計算遮蔽閾值 (masking        │
    │  │ threshold)                   │
    │  │ 低於閾值的頻率可量化/捨棄    │
    │  └──────────────────────────────┘
    │
    ▼
量化 + Huffman 編碼
    │
    ▼
MP3 位元串流 (典型 128-320kbps)
```

### 位元率

| 模式 | 位元率 | 壓縮比 |
|---|---|---|
| 原始 CD | 1411 kbps | 1:1 |
| MP3 320kbps | 320 kbps | ~4:1 |
| MP3 256kbps | 256 kbps | ~5:1 |
| MP3 192kbps | 192 kbps | ~7:1 |
| MP3 128kbps | 128 kbps | ~11:1 |

## 相關檔案

- `media/mpeg1/src/main.rs` — MPEG-1 解碼器入口
- `media/mpeg1/src/decoder.rs` — 解碼器主控
- `media/mpeg1/src/idct.rs` — 反離散餘弦轉換
- `media/mpeg1/src/motion.rs` — 運動補償
- `media/mp3/src/` — MP3 編解碼器完整實作

## 參考資料

- ISO/IEC 11172 (MPEG-1 標準)
- ITU-T H.262 (MPEG-2 Video)
- MP3 心理聲學模型：ISO/IEC 11172-3
