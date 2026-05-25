#!/usr/bin/env bash
set -x

# Build the compiler
cargo build --release 2>&1

# Test each example
for example in examples_rustc4/*.rs; do
    name=$(basename "$example" .rs)
    ir_file="out/$name.ir"

    echo "=== Compiling $example -> $ir_file ==="
    cargo run --release -- "$example" "$ir_file" 2>&1

    echo "=== Running $name with lli ==="
    lli "$ir_file"
    echo ""
done
