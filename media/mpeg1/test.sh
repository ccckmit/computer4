set -x
cargo build
cargo run -- data/test.mpg # 會印出 mpg 的基本資訊
cargo run -- data/test.mpg 200 data/frame200.ppm # 會將 mpg 的第 200 個影格抽出來
ffmpeg -i data/frame200.ppm data/frame200.jpg

