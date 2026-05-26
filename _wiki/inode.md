# Inode

## 概述

Inode (Index Node) 是 Unix 檔案系統中用於儲存檔案中繼資料 (metadata) 的核心資料結構。每個檔案與目錄都對應一個唯一的 inode，記錄其型別、大小、權限、時間戳、資料區塊位置等資訊。本專案的 `database/inodefs/` crate 實作了一個 Inode 為基礎的虛擬檔案系統。

## Inode 的內容

### 中繼資料欄位

```rust
struct Inode {
    id: u64,             // inode 編號（檔案系統內唯一）
    mode: FileType,      // 檔案型別與權限
    nlink: u32,          // 硬連結計數
    uid: u32,            // 所有者使用者 ID
    gid: u32,            // 所有者群組 ID
    size: u64,           // 檔案大小（位元組）
    blocks: Vec<u64>,    // 資料區塊索引陣列
    atime: u64,          // 最後存取時間
    mtime: u64,          // 最後修改時間
    ctime: u64,          // 中繼資料變更時間
}
```

### 與目錄項目的關係

```
目錄 (/home/user):
  doc.txt → inode #42
  photo.jpg → inode #73
  music/ → inode #88

Inode #42:
  型別: 一般檔案
  大小: 4096 位元組
  區塊: [100, 101, 102, ...]
  連結數: 1

Inode #88:
  型別: 目錄
  大小: 512 位元組
  區塊: [200]
  連結數: 2
```

## 本專案的 inodefs

`database/inodefs/` 使用 HashMap 實作記憶體中的 inode 檔案系統。

### 資料結構

```rust
pub struct InodeFs {
    inodes: HashMap<u64, Inode>,     // inode 表
    blocks: HashMap<u64, Vec<u8>>,   // 資料區塊儲存
    next_inode: u64,                  // 下一個可用的 inode 編號
    next_block: u64,                  // 下一個可用的區塊編號
    root: u64,                        // 根目錄的 inode 編號
}
```

### API

```
create_file(parent_dir_inode, name) → inode
create_dir(parent_dir_inode, name) → inode
read_file(inode) → data
write_file(inode, data)
delete(parent_dir_inode, name)
list_dir(inode) → [names]
lookup(parent_dir_inode, name) → inode
link(parent, old_name, new_name) → hard link
symlink(target, link_path) → symbolic link
stat(inode) → metadata
```

### 本專案 inodefs vs Unix inode

| 特性 | Unix inode | inodefs |
|---|---|---|
| 區塊指標 | 直接 + 間接 + 雙重間接 | Vec<u64>（直接索引） |
| 硬連結 | 支援 (nlink) | 支援 |
| 符號連結 | 支援 | 支援 |
| 權限 | 9 位元 (rwx) | 有 (FileType) |
| 時間戳 | atime/mtime/ctime | atime/mtime/ctime |
| 持久性 | 磁碟 | 記憶體（虛擬） |
| inode 數量限制 | 固定（mkfs 時決定） | 無限制（HashMap） |
| 區塊定址 | 定長陣列 + 間接 | 動態 Vec |

## 相關檔案

- `database/inodefs/src/lib.rs` — inode 檔案系統實作
- `os/xv6-rust-octopus/kernel/src/file.rs` — xv6 的 inode 實作

## 參考資料

- Unix 檔案系統設計 (Thompson, Ritchie, 1974)
- Maurice J. Bach, *The Design of the UNIX Operating System*
- xv6 教材 Chapter 6: File system
