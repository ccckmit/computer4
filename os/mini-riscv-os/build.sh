#!/bin/bash
set -e
cd /Users/Shared/ccc/project/computer4/os/mini-riscv-os

TARGET=riscv32imac-unknown-none-elf
DEPS=target/$TARGET/release/deps

# Build Rust lib
cargo build --release --target $TARGET 2>&1

# Find and extract object from rlib
RLIB=$(ls $DEPS/libmini_riscv_os-*.rlib 2>/dev/null | head -1)
if [ -z "$RLIB" ]; then
    echo "No rlib found"
    exit 1
fi

# Extract object files from rlib
ar x $RLIB

# Find the object file
OBJ=$(ls *.o 2>/dev/null | head -1)
if [ -z "$OBJ" ]; then
    echo "No object file extracted"
    exit 1
fi

# Compile assembly
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -c start.s -o start.o
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -c sys.s -o sys.o

# Link everything
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -T os.ld -o os.elf start.o sys.o $OBJ

echo "Build complete: os.elf"
ls -la os.elf