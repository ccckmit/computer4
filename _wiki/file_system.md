# 檔案系統 (File System)

## 概述

檔案系統是作業系統中用於管理持久化資料的機制，控制資料如何在儲存裝置上儲存、組織、讀取與寫入。本專案包含多個檔案系統相關元件的實作：`database/inodefs/`（Inode 為基礎的虛擬檔案系統）、`xv6-rust-octopus` 的 xv6 檔案系統（使用 inode + 日誌）、`database/btree/`（可作為 KV 儲存引擎）。

## 檔案系統的基本概念

### 檔案 (File)

檔案的抽象：
- **路徑 (path)：** 檔案在目錄樹中的位置（如 `/home/user/doc.txt`）
- **內容 (content)：** 檔案儲存的資料
- **中繼資料 (metadata)：** 大小、權限、時間戳、擁有者

### 目錄 (Directory)

目錄是一種特殊檔案，包含從檔名到 inode 編號的映射：
- 目錄樹的根為 `/`
- 每個檔案/目錄由絕對路徑唯一識別

### Inode (Index Node)

inode 是檔案系統中用於描述檔案或目錄中繼資料的資料結構，包含：
- 檔案大小
- 權限與擁有者
- 時間戳（建立、修改、存取）
- 資料區塊指標（直接、間接、雙重間接）

## inodefs：本專案的自製檔案系統

`database/inodefs/` 是一個 Inode 為基礎的虛擬檔案系統。

### 核心結構

```rust
// inode 結構 (簡化)
struct Inode {
    id: u64,              // inode 編號
    mode: FileType,       // 檔案型別（一般檔案、目錄）
    size: u64,            // 檔案大小
    blocks: Vec<u64>,     // 資料區塊列表
    ctime: u64,           // 建立時間
    mtime: u64,           // 修改時間
}

// 目錄項目
struct DirEntry {
    name: String,         // 檔名
    inode_id: u64,        // 對應的 inode
}

// 檔案系統
struct InodeFs {
    inodes: HashMap<u64, Inode>,    // inode 表
    blocks: HashMap<u64, Vec<u8>>,  // 資料區塊
    root: u64,                       // 根目錄 inode
    next_inode: u64,                 // 下一個可用的 inode 編號
    next_block: u64,                // 下一個可用的區塊編號
}
```

### API

```rust
impl InodeFs {
    pub fn new() -> Self;
    pub fn create_file(&mut self, parent: u64, name: &str) -> Result<u64>;
    pub fn create_dir(&mut self, parent: u64, name: &str) -> Result<u64>;
    pub fn read_file(&self, inode: u64) -> Result<&[u8]>;
    pub fn write_file(&mut self, inode: u64, data: &[u8]) -> Result<()>;
    pub fn delete(&mut self, parent: u64, name: &str) -> Result<()>;
    pub fn list_dir(&self, inode: u64) -> Result<Vec<String>>;
    pub fn lookup(&self, parent: u64, name: &str) -> Result<u64>;
}
```

## xv6 的檔案系統

xv6 實作了一個傳統的 Unix-like 檔案系統，包含日誌 (journaling) 以確保崩潰復原。

### 磁碟佈局

```
┌─────────┬──────────┬──────────┬──────────┬──────────┐
│ Boot    │ Super    │ Log      │ Inode    │ Bitmap   │ Data     │
│ Block   │ Block    │ Blocks   │ Blocks   │ Blocks   │ Blocks   │
│ (1)     │ (1)      │ (N)      │ (M)      │ (B)      │ (D)      │
└─────────┴──────────┴──────────┴──────────┴──────────┴──────────┘
```

- **Super block：** 檔案系統中繼資料（區塊總數、inode 總數、各區域大小）
- **Log blocks：** 日誌區塊（用於崩潰復原）
- **Inode blocks：** 存放 inode 陣列
- **Bitmap blocks：** 記錄哪些資料區塊已被使用
- **Data blocks：** 存放實際檔案內容與目錄資料

### 檔案資料結構

```c
// xv6 inode (簡化)
struct inode {
    uint type;      // 檔案型別 (T_DIR, T_FILE, T_DEV)
    uint major;     // 主裝置號（僅 T_DEV）
    uint minor;     // 次裝置號（僅 T_DEV）
    short nlink;    // 連結數
    uint size;      // 檔案大小（位元組）
    uint addrs[NDIRECT+1]; // 資料區塊指標
    // addrs[0..11]: 直接區塊
    // addrs[12]:    間接區塊
};

// xv6 目錄項目
struct dirent {
    ushort inum;    // inode 編號
    char name[DIRSIZ]; // 檔名（14 字元）
};
```

### 區塊定址方式

xv6 使用混合定址：「直接區塊 + 間接區塊」：

```
inode->addrs:
  [0]  → data block (直接)
  [1]  → data block (直接)
  ...
  [11] → data block (直接)
  [12] → 間接區塊 → [data block, data block, ...]
                    (256 個額外的區塊指標)
```

最大檔案大小 = 12 × BSIZE + 256 × BSIZE = 268 × 1024 = 268KB（BSIZE=1024）

### 日誌 (Journaling / Logging)

xv6 使用先寫日誌 (write-ahead logging) 確保崩潰復原：

```
1. 開始交易 (begin_trans)
2. 將所有修改寫入日誌區塊
3. 提交交易 (commit_trans) — 寫入日誌標頭
4. 將日誌區塊複製到實際位置 (install)
5. 清除日誌標頭
```

若系統在步驟 3 前崩潰：忽略日誌（無修改生效）
若系統在步驟 3 後步驟 4 前崩潰：重新安裝日誌
若系統在步驟 4 後步驟 5 前崩潰：重新安裝日誌（冪等操作）

### 檔案操作流程

**讀取檔案：**
```
open("/foo/bar", O_RDONLY)
  → 從根目錄 inode 開始
  → 查詢 "foo" 目錄項目 → 取得 foo 的 inode
  → 查詢 "bar" 目錄項目 → 取得 bar 的 inode
  → 從 inode 的 addrs 中讀取資料區塊
```

**寫入檔案：**
```
write(fd, buf, n)
  → 使用 inode 的 addrs 找到或分配資料區塊
  → 將 buf 內容寫入區塊
  → 更新 inode 的 size
  → 記錄到日誌
```

## inodefs vs xv6 檔案系統

| 特性 | inodefs | xv6 FS |
|---|---|---|
| 實作語言 | Rust | Rust (移植) |
| 持久性 | 無（純記憶體） | 有（區塊裝置模擬） |
| 日誌 | 無 | 有（write-ahead log） |
| 間接區塊 | 無（直接向量） | 單層間接 |
| 路徑解析 | 提供 lookup() | 完整路徑走訪 |
| Bitmap | 無（HashMap 管理） | 有（固定位元圖） |
| 快取 | 無 | Buffer cache (BCACHE) |
| 同步 | 無 | 睡眠鎖 (sleeplock) |

## 參考資料

- xv6 教材 Chapter 6: File system
- inodefs 實作：`database/inodefs/`
- Andrew S. Tanenbaum, *Modern Operating Systems*, Chapter 4: File Systems
- Unix 檔案系統設計 (Ken Thompson, Dennis Ritchie)
