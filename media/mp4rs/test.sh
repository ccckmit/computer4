set -x
cargo build
cargo test
cargo run -- data/test.mp4
# Extract raw frame 200
cargo run -- data/test.mp4 0 200 /tmp/frame200.h264
# Extract as Annex-B stream
cargo run -- data/test.mp4 annex-b 0 200 /tmp/frame200_annexb.h264
echo "Files written:"
ls -la /tmp/frame200.h264 /tmp/frame200_annexb.h264
