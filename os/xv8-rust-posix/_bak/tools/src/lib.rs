# xv8-tools — POSIX 工具 for xv8

此 crate 包含可交叉編譯到 `riscv64gc-unknown-none-elf` 的 POSIX 工具。

所有工具使用 `user::*` wrappers（`no_std`）而非標準 library。

## 編譯

```sh
cargo build --release --target riscv64gc-unknown-none-elf
```

## 工具列表

| 工具 | 說明 |
|------|------|
| cp | 複製檔案 |
| mv | 搬移/重新命名 |
| chmod | 變更權限 |
| chown | 變更擁有者 |
| touch | 更新時間戳 |
| head | 顯示開頭行 |
| tail | 顯示結尾行 |
| sort | 排序 |
| uniq | 去除相鄰重複 |
| cut | 擷取欄位 |
| tr | 字元轉換 |
| mkdir | 建立目錄 |
| rmdir | 刪除空目錄 |
| ln | 連結 |
| readlink | 讀取連結目標 |
| symlink | 建立符號連結 |
| dirname | 目錄名稱 |
| basename | 檔案名稱 |
| test | 條件測試 |
| expr | 表達式求值 |
| printf | 格式化輸出 |
| printenv | 顯示環境變數 |
| env | 執行環境 |
| id | 顯示身份 |
| uname | 系統資訊 |
| pwd | 目前目錄 |
| stat | 檔案狀態 |
| wc | 位元組/行/字計數 |
| tee | 資料分流 |
| nohup | 忽略 SIGHUP |
| nice | 調整優先序 |
| kill | 傳送信號 |