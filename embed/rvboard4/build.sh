#!/bin/bash
set -e
cd /Users/Shared/ccc/project/computer4/embed/rvboard4

# Put rustup cargo first in PATH
export PATH="/Users/cccuser/.cargo/bin:$PATH"

echo "Building rvboard4..."

# Compile Rust library
RUSTUP_TOOLCHAIN=nightly cargo build --release -p rvboard4

# Compile assembly (provides putchar)
riscv64-unknown-elf-gcc -c -o src/boot.o src/boot.S -O2 --entry=_start -march=rv32i -mabi=ilp32

# Link everything
riscv64-unknown-elf-ld -m elf32lriscv -o target/rvboard4.elf src/boot.o target/riscv32i-unknown-none-elf/release/librvboard4.a -T linker/rvboard32.ld --entry=_start

echo "Built: target/rvboard4.elf"