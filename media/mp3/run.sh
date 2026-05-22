cargo build --release

# 檢視 MP3 資訊
./target/release/mpeg_codec info data/test.mp3

# 使用 ffmpeg 解碼 MP3 為 PCM (16‑bit LE, 22050 Hz, mono)
ffmpeg -y -i data/test.mp3 -f s16le -ar 22050 -ac 1 data/test.pcm

# 從原始 PCM 編碼為 MP3 (使用 ffmpeg for proper encoding)
ffmpeg -y -f s16le -ar 22050 -ac 1 -i data/test.pcm -b:a 128k data/test2.mp3
