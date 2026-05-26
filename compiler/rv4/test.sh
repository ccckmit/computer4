#!/bin/bash
set -x

# Build examples
for ex in hello fact fib sum; do
    rustc --target riscv64imac-unknown-none-elf --emit obj -C opt-level=z -C overflow-checks=off examples/$ex.rs -o examples/$ex.o
    if [ $? -ne 0 ]; then
        echo "Failed to compile examples/$ex.rs"
        exit 1
    fi
done

# Run examples
for ex in hello fact fib sum; do
    echo "=== $ex ==="
    cargo run -- examples/$ex.o
    if [ $? -ne 0 ]; then
        echo "FAIL: $ex"
        exit 1
    fi
done

echo "All examples passed!"
