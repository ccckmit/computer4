# 建立 fs.img（如還沒有）
qemu-img create target/fs.img 2G
./mkfs.sh
# 執行
cargo run --release