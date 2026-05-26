#!/bin/bash
cd /Users/Shared/ccc/project/computer4/embed/rvboard4

./build.sh

echo ""
echo "=== Running in QEMU ==="
echo ""

# Run QEMU with serial output to file
rm -f /tmp/rvboard4_serial.txt
timeout 1 qemu-system-riscv32 -kernel target/rvboard4.elf -machine virt -serial file:/tmp/rvboard4_serial.txt 2>/dev/null || true

# Small delay to ensure file is written
sleep 0.1

# Show only the actual program output (first line)
if [ -f /tmp/rvboard4_serial.txt ]; then
    grep -m1 "hello rvboard4" /tmp/rvboard4_serial.txt || echo "(no output captured)"
fi

echo ""
echo "=== Done ==="