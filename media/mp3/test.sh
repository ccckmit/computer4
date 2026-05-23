#!/bin/bash
set -e

echo "--- Building Project ---"
cargo build --release

echo -e "\n--- Testing MP3 to PCM Decode ---"
if [ -f "data/test.mp3" ]; then
    ./target/release/mpeg_codec decode data/test.mp3 output.pcm
    if [ -f "output.pcm" ]; then
        echo "SUCCESS: output.pcm created."
        ls -lh output.pcm
    else
        echo "FAILURE: output.pcm not created."
        exit 1
    fi
else
    echo "SKIP: data/test.mp3 not found."
    exit 1
fi

echo -e "\n--- Testing MP3 Info ---"
./target/release/mpeg_codec info data/test.mp3

echo -e "\n--- Verifying with FFmpeg (PCM -> MP3) ---"
# Note: we need to specify the format, sample rate, and channels for PCM since it's raw.
# Based on previous output: 1 channel, 22050 Hz, s16le
ffmpeg -y -f s16le -ar 22050 -ac 1 -i output.pcm data/test2.mp3

if [ -f "data/test2.mp3" ]; then
    echo "SUCCESS: data/test2.mp3 created via FFmpeg."
    ls -lh data/test2.mp3
else
    echo "FAILURE: FFmpeg failed to create data/test2.mp3."
    exit 1
fi

echo -e "\nAll tests passed!"
