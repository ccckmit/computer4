# 遊戲引擎 (Game Engine)

## 概述

遊戲引擎是提供遊戲開發所需核心功能的軟體框架，包含圖形渲染、物理模擬、音效、輸入處理、遊戲循環、網路通訊等。本專案的 `game4` crate 實作了一個 WebSocket 為基礎的遊戲框架，後端以 Rust 驅動，前端使用 JavaScript 渲染。

## 遊戲循環 (Game Loop)

遊戲循環是所有遊戲的核心模式：

```
幀開始
  │
  ▼
處理輸入 (鍵盤、滑鼠、遊戲手把)
  │
  ▼
更新遊戲狀態 (位置、碰撞、AI)
  │
  ▼
渲染畫面
  │
  ▼
幀結束 (等待下一幀)
```

### 固定時間步長 vs 可變時間步長

```rust
// 固定時間步長 (Fixed timestep)
const FIXED_DT: f64 = 1.0 / 60.0;  // 60 FPS
let mut accumulator = 0.0;

loop {
    let frame_time = get_delta_time();
    accumulator += frame_time;

    while accumulator >= FIXED_DT {
        update(FIXED_DT);  // 物理/邏輯更新
        accumulator -= FIXED_DT;
    }

    render();  // 渲染（可做插值）
}
```

優點：物理模擬穩定、可重現；缺點：可能追趕落後。

## game4 架構

```
┌─────────────────────────────────────┐
│  瀏覽器端 (JavaScript)               │
│  ┌──────────────────────────────┐   │
│  │ game4.js                     │   │
│  │  - Canvas 2D 渲染             │   │
│  │  - WebSocket 客戶端           │   │
│  │  - 輸入收集 (鍵盤/滑鼠)        │   │
│  │  - 音效播放                   │   │
│  └──────────┬───────────────────┘   │
└─────────────┼───────────────────────┘
              │ WebSocket (ws://)
┌─────────────┼───────────────────────┐
│  伺服器端 (Rust)                     │
│  ┌──────────▼───────────────────┐   │
│  │ game4 核心                    │   │
│  │  - WebSocket 伺服器           │   │
│  │  - 遊戲狀態管理               │   │
│  │  - 玩家管理                   │   │
│  │  - 碰撞偵測                   │   │
│  │  - 遊戲邏輯更新               │   │
│  │  - 排行榜/分數                │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

### WebSocket 協定

```rust
// 伺服器 → 用戶端 (JSON)
{
    "type": "state_update",
    "players": [
        {"id": 0, "x": 100, "y": 200, "score": 5},
        {"id": 1, "x": 300, "y": 150, "score": 3}
    ],
    "ball": {"x": 250, "y": 180, "vx": 5, "vy": -3},
    "timestamp": 1234567890
}

// 用戶端 → 伺服器 (JSON)
{
    "type": "input",
    "keys": ["ArrowUp", "Space"],
    "mouse": {"x": 400, "y": 300, "buttons": 1}
}
```

### 遊戲範例

#### Pong

`gui/game4/examples/pong/` — 經典桌球遊戲：

```sh
cd gui/game4
cargo run --example pong
# 瀏覽器開啟 http://localhost:8080
```

- 兩位玩家（W/S 與方向鍵上下）
- 球碰撞物理
- 分數追蹤

#### Assault

`gui/game4/examples/assault/` — 射擊遊戲：

```sh
cd gui/game4
cargo run --example assault
```

## 遊戲引擎 vs 獨立遊戲

| 特性 | game4 | 獨立遊戲 (Bevy) | 專業引擎 (Unity) |
|---|---|---|---|
| 圖形 | Canvas 2D (JS) | WebGPU/Vulkan | DirectX/Vulkan |
| 物理 | 自訂 | Rapier/Havok | PhysX |
| 音效 | Web Audio API | rodio | FMOD/Wwise |
| 網路 | WebSocket | 多種 | Photon/Mirror |
| 場景管理 | 無 | ECS | GameObject |
| 腳本 | JavaScript | Rust | C# |
| 大小 | ~數千行 | ~百萬行 | ~千萬行 |

## 其他遊戲相關元件

本專案中其他與遊戲/媒體相關的 crate：

| crate | 功能 |
|---|---|
| `aplayer4` | 音訊播放器（使用 rodio + crossterm TUI） |
| `win4` | 視窗管理器（eframe/egui，非遊戲專用但可作為遊戲啟動器） |

## 相關檔案

- `gui/game4/src/lib.rs` — game4 入口
- `gui/game4/src/game4.rs` — 核心遊戲循環與 WebSocket 處理
- `gui/game4/examples/pong/` — Pong 遊戲原始檔
- `gui/game4/examples/assault/` — Assault 遊戲原始檔
- `gui/game4/game4.js` — 前端 JavaScript 引擎

## 參考資料

- 遊戲程式設計模式 (Game Programming Patterns)：https://gameprogrammingpatterns.com/
- WebSocket 協定 (RFC 6455)：https://tools.ietf.org/html/rfc6455
- Canvas 2D API：https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API
