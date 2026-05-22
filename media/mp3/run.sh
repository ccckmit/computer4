cargo build --release

# 檢視 MP3 資訊
./target/release/mpeg_codec info data/test.mp3

# 解碼為原始 PCM
./target/release/mpeg_codec decode data/test.mp3 data/test.pcm

# 從原始 PCM 編碼為 MP3
./target/release/mpeg_codec encode data/test.pcm data/test2.mp3 --bitrate 128 --sr 44100 --ch 2
