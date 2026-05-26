#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

TARGET="riscv32imac-unknown-none-elf"
TOOLCHAIN="riscv64-unknown-elf"
QEMU="qemu-system-riscv32"

CFLAGS="-nostdlib -fno-builtin -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -fno-PIC"
QFLAGS="-nographic -smp 4 -machine virt -bios none"

echo "=== Building mini-riscv-os ==="

echo "[1/4] Building Rust..."
cargo build --release --target "$TARGET"

echo "[2/4] Assembling..."
$TOOLCHAIN-gcc $CFLAGS -c start.s -o start.o
$TOOLCHAIN-gcc $CFLAGS -c sys.s -o sys.o

echo "[3/4] Linking..."
RLIB=$(ls target/$TARGET/release/deps/libmini_riscv_os-*.a 2>/dev/null | head -1)
if [ -z "$RLIB" ]; then
    echo "Error: rlib not found"
    exit 1
fi
$TOOLCHAIN-gcc $CFLAGS -T os.ld -o os.elf start.o sys.o -Wl,--whole-archive "$RLIB" -Wl,--no-whole-archive -lc

echo "[4/4] Running in QEMU..."
echo "Press Ctrl-A then X to exit"
$QEMU $QFLAGS -kernel os.elf