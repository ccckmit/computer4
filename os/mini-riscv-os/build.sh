#!/bin/bash
set -x
cd /Users/Shared/ccc/project/computer4/os/mini-riscv-os

echo "=== Building Rust mini-riscv-os ==="

# Build Rust lib
cargo build --release --target riscv32imac-unknown-none-elf 2>&1 | grep -E "error|warning|Finished"

# Extract object from library
riscv64-unknown-elf-ar x target/riscv32imac-unknown-none-elf/release/deps/libmini_riscv_os-3008861c6995acbb.a

# Compile assembly
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -c start.s -o start.o
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -c sys.s -o sys.o

# Link
riscv64-unknown-elf-gcc -nostdlib -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -T os.ld -o os.elf start.o sys.o libmini_riscv_os-3008861c6995acbb.a

echo "=== Build complete ==="
ls -la os.elf

echo "=== Running QEMU ==="
timeout 20 qemu-system-riscv32 -nographic -smp 4 -machine virt -bios none -kernel os.elf 2>&1 | tee output.txt

echo "=== Checking output ==="
grep -c "Task0: Created" output.txt && echo "Task0: OK"
grep -c "Task1: Created" output.txt && echo "Task1: OK"
grep -c "timer_handler" output.txt && echo "Preemptive timer: OK"
grep "counter before" output.txt | head -5