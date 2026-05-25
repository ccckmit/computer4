#!/usr/bin/env bash
set -x
cargo clean 2>&1
cargo build 2>&1
cargo test 2>&1
echo "=== Running examples via binary ==="
for f in examples_ll/*.ir; do
    name=$(basename "$f" .ir)
    echo "--- $name ---"
    ./target/debug/lli4 "$f"
done
