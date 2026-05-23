cargo build --release

# 檢視 MP3 資訊
./target/release/mpeg_codec data/test.mp3

# 解碼 MP3 為 PCM (16‑bit LE, 22050 Hz, mono)
./target/release/mpeg_codec data/test.mp3 data/test.pcm

# 從原始 PCM 編碼為 MP3 (使用 ffmpeg for proper encoding)
./target/release/mpeg_codec data/test.pcm data/test.mp3
