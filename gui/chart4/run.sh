set -x
# chart4 — Rust 綁定 plotly.js
# 用法: ./run.sh [子命令]
#    line      折線圖
#    scatter   散點圖
#    bar       長條圖
#    pie       圓餅圖
#    histogram 直方圖
#    multi     混合圖表
#    subplot   雙 Y 軸
#    serve     互動式 Server (http://localhost:8080)
#    all       全部示範（預設）

RUST_BACKTRACE=1 cargo run -- $@
