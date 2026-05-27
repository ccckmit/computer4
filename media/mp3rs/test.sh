set -x
cargo build
cargo test
cargo run -- data/test.mp3
cargo run -- data/test.mp3 -o data/test.wav
cargo run -- data/test.wav
cargo run -- data/test.wav -o data/test2.mp3

