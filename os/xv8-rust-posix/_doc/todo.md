# xv8-rust-posix — v1.1 POSIX 工具整合

## 現況

### 已完成工具

| 工具 | 檔案 | 狀態 |
|------|------|------|
| cp | `bin/cp_xv8.rs` | ✅ 完成 |
| chmod | `bin/chmod_xv8.rs` | ✅ 完成 |
| touch | `bin/touch_xv8.rs` | ✅ 完成 |
| mv | `bin/mv_xv8.rs` | ✅ 完成 |
| head | `bin/head_xv8.rs` | ✅ 完成 |
| tail | `bin/tail_xv8.rs` | ✅ 完成（已重寫，移除 Vec 依賴） |
| sort | `bin/sort_xv8.rs` | ✅ 完成（固定 1000 行） |
| uniq | `bin/uniq_xv8.rs` | ✅ 完成 |
| cut | `bin/cut_xv8.rs` | ✅ 完成 |
| tr | `bin/tr_xv8.rs` | ✅ 完成 |
| chown | `bin/chown_xv8.rs` | ✅ 完成（已重寫，移除 Vec 依賴） |
| mkdir | `bin/mkdir.rs` | ✅ 完成 |
| rm | `bin/rm.rs` | ✅ 完成 |
| wc | `bin/wc.rs` | ✅ 完成 |
| cat | `bin/cat.rs` | ✅ 完成 |
| dirname | `bin/dirname.rs` | ✅ 完成 |
| basename | `bin/basename.rs` | ✅ 完成 |
| readlink | `bin/readlink.rs` | ✅ 完成 |
| symlink | `bin/symlink.rs` | ✅ 完成 |
| test | `bin/test.rs` | ✅ 完成 |
| expr | `bin/expr.rs` | ✅ 完成 |
| printf | `bin/printf.rs` | ✅ 完成 |
| printenv | `bin/printenv.rs` | ✅ 完成（墊片，無環境變數支援） |
| env | `bin/env.rs` | ✅ 完成（墊片，無環境變數支援） |

### 待加入工具

#### 低優先

| 工具 | 說明 |
|------|------|
| nohup | 忽略 SIGHUP |
| stat | 檔案狀態 |
| id | 顯示身份 |
| uname | 系統資訊 |
| pwd | 目前目錄 |

---

## 二進位檔注册

所有工具註冊於 `user/Cargo.toml` 的 `[[bin]]` 區塊：

```toml
[[bin]]
name = "cp_xv8"
path = "bin/cp_xv8.rs"

[[bin]]
name = "chmod_xv8"
path = "bin/chmod_xv8.rs"

... etc
```

---

## 編譯與測試

```sh
# 交叉編譯
cargo build --release --package user

# 測試
./test.sh  # 15 tests PASS
```

---

## 换电脑後繼續

1. 確認所有 `bin/*.rs` 檔案存在
2. 確認 `user/Cargo.toml` 的 `[[bin]]` 項目完整
3. 執行 `cargo build --release --package user` 編譯
4. 執行 `./test.sh` 驗證（15 tests PASS）
5. 繼續加入剩餘工具（nohup, stat, id, uname, pwd）