# computer4 Wiki 索引

本專案相關的領域知識與專有名詞說明。

## 詞項

| 頁面 | 摘要 |
|---|---|---|
| [RISC-V](riscv.md) | 開放指令集架構，用於所有 OS crate（模擬器、核心、BSP、xv6/xv7 移植） |
| [LLVM IR](llvm_ir.md) | 編譯器管線 rustc4 → lli4 的中間表示式 |
| [ruHDL](ruhdl.md) | 自製 Rust 硬體描述語言，包含邏輯閘、CPU 模擬與 Verilog 前端 |
| [LSM-Tree](lsm_tree.md) | Log-structured merge-tree 儲存引擎，用於 lsm/ 與 db6/ |
| [全文檢索](full_text_search.md) | 支援中日韓(CJK)的全文檢索引擎，用於 fts/ 與 db6/ |
| [Swiss Table](swiss_table.md) | Google 高效能雜湊表演算法，Robin Hood hashing 探測 |
| [Patricia Trie](patricia_trie.md) | 壓縮前綴樹 (Radix Tree)，支援前綴搜尋與最長前綴匹配 |
| [LLRB 樹](llrb_tree.md) | 左傾紅黑樹，紅黑樹的簡化變體，用於 redblacktree/ |
| [ELF 格式](elf_format.md) | 可執行與可鏈結格式，用於 rv4/ 與 objdump/ |
| [編譯器](compiler.md) | 將高階語言轉換為機器碼的程式，以 rustc4→lli4 管線說明 |
| [虛擬機](virtual_machine.md) | 模擬實體機器的軟體層，包含 RISC-V 模擬器與 LLVM IR 直譯器 |
| [中間碼](intermediate_representation.md) | 編譯器內部的程式表示法，以 LLVM IR 為核心說明 |
| [RSA](rsa.md) | 非對稱加密演算法，用於 keygen/ 與 ssl4/ |
| [密碼學](cryptography.md) | 加密、簽章、憑證、SSL/TLS 的理論與實作 |
| [作業系統](operating_system.md) | 管理硬體資源的系統軟體，包含 mini-riscv-os、xv6/xv7、rvboard4 |
| [排程](scheduling.md) | 作業系統決定下一個執行行程的策略，xv6/xv7 的 round-robin 實作 |
| [虛擬記憶體](virtual_memory.md) | 為每個行程提供獨立位址空間的硬體機制，RISC-V SVM/MSU 模式 |
| [檔案系統](file_system.md) | 檔案資料的組織與存取方法，xv6/xv7 的 inode 層級目錄結構 |
| [行程與執行緒](process_thread.md) | 行程與執行緒的區別及本專案的實作（xv6 行程表、xv7 RISC-V 排程） |
| [競爭情況與互斥鎖](race_condition.md) | 並行程式中的同步問題與解決方案（spinlock、mutex、sleep lock） |
| [資料庫](database.md) | 結構化資料的儲存與查詢系統，db6 的引擎架構 |
| [JPEG](jpeg.md) | 廣泛使用的有損影像壓縮標準，media/jpeg/ 完整編解碼器實作 |
| [BTree](btree.md) | 平衡多路搜尋樹，用於 database/btree/ 引擎 |
| [SPICE](spice.md) | 類比電路模擬器，用於 eda/ruspice/ |
| [MPEG](mpeg.md) | 動態影像壓縮標準，media/mpeg1/ 解碼器與 media/mp3/ 音訊編解碼 |
| [Rust 語言](rust_lang.md) | 系統程式語言，本專案所有 crate 的實作語言、所有權與程式風格 |
| [Inode](inode.md) | Unix 檔案系統的中繼資料結構，用於 database/inodefs/ 虛擬檔案系統 |
| [SQL](sql.md) | 關聯式資料庫查詢語言，db6/ 的 SQL 解析器、規劃器、執行器 |
| [HDL & EDA](hdl_eda.md) | 硬體描述語言與電子設計自動化，ruhdl、verilog4、ic4、synthesis、ruspice |
| [遊戲引擎](game_engine.md) | 遊戲開發核心框架，game4 的 WebSocket 伺服器 + JS 前端 |
| [瀏覽器](browser.md) | 網頁瀏覽器的核心元件，本專案的三種實作 (browser4、browser5、md4browser) |
| [HTML](html.md) | 超文字標記語言，web/ 中 browser4 與 browser5 的 HTML 解析實作 |
| [CSS](css.md) | 層疊樣式表，xdom4 的 CSS 選擇器與 browser5 的樣式計算 |
| [JavaScript](javascript.md) | 網頁動態語言，js4 自製引擎與 boa_engine 的完整實作 |
| [DOM](dom.md) | 文件物件模型，xdom4 的 Node/Element/Document 與 CSS 選擇器 |
| [HTTP](http.md) | 超文字傳輸協定，browser4/5 的 reqwest 使用與 WebSocket 對比 |
| [SSL/TLS](ssl_tls.md) | 安全傳輸層協定，ssl4 的 rustls/tokio-rustls 實作 |

## 參見

- [AGENTS.md](../AGENTS.md) — 各 crate 建置/測試說明
- `database/db6/AGENTS.md` — db6 架構筆記
- `math4/AGENTS.md` — 數學函式庫慣例
