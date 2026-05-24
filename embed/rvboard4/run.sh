#!/bin/bash
cd /Users/Shared/ccc/project/computer4/embed/rvboard4

./build.sh && echo "Running in QEMU..." && timeout 2 qemu-system-riscv32 -kernel target/rvboard4.elf -machine virt -nographic 2>&1 | grep -v "^OpenSBI" | grep -v "^  " | grep -v "Platform" | grep -v "Firmware" | grep -v "Runtime" | grep -v "Domain" | grep -v "Boot" | grep -v "^$" || true