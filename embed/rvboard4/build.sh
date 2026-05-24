#!/bin/bash
set -e

echo "Building rvboard4..."

# Compile assembly
riscv64-unknown-elf-gcc -c -o src/boot.o src/boot.S -O2 --entry=_start -march=rv32i -mabi=ilp32

# Link
riscv64-unknown-elf-ld -m elf32lriscv -o target/rvboard4.elf src/boot.o -T linker/rvboard32.ld --entry=_start

echo "Built: target/rvboard4.elf"