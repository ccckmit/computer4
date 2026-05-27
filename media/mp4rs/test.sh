set -x
cargo build
cargo test
cargo run -- data/test.mp4

# Extract frame 200 (non-keyframe) as Annex-B
cargo run -- data/test.mp4 annex-b 0 200 data/frame200.h264

# Convert to JPG via ffmpeg
ffmpeg -y -f h264 -i data/frame200.h264 -update 1 -frames:v 1 data/frame200.jpg 2>/dev/null || \
ffmpeg -y -i data/test.mp4 -vf "select=eq(n\,200)" -vframes 1 data/frame200.jpg

echo "Files written:"
ls -la data/frame200.h264 data/frame200.jpg
