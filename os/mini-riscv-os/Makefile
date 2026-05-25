CC = riscv64-unknown-elf-gcc
CFLAGS = -nostdlib -fno-builtin -mcmodel=medany -march=rv32ima_zicsr -mabi=ilp32 -fno-PIC

QEMU = qemu-system-riscv32
QFLAGS = -nographic -smp 4 -machine virt -bios none

TARGET = riscv32imac-unknown-none-elf
TARGET_DIR = target/$(TARGET)/release/deps

.PHONY: all clean build run qemu

all: build

build: os.elf

os.elf: start.o sys.o rust.o os.ld
	$(CC) $(CFLAGS) -T os.ld -o os.elf start.o sys.o rust.o

start.o: start.s
	$(CC) $(CFLAGS) -c start.s -o start.o

sys.o: sys.s
	$(CC) $(CFLAGS) -c sys.s -o sys.o

rust.o: src/main.rs Cargo.toml
	cargo build --release --target $(TARGET) 2>&1
	@# Extract object files from rlib
	@cd $(TARGET_DIR) && for f in mini_riscv_os-*.o; do [ -f "$$f" ] && cp "$$f" /Users/Shared/ccc/project/computer4/os/mini-riscv-os/rust.o && break; done || true
	@if [ ! -f rust.o ]; then ar x $(TARGET_DIR)/libmini_riscv_os-*.rlib 2>/dev/null && cat *.o > rust.o 2>/dev/null || echo "Trying alternative..."; fi
	@ls $(TARGET_DIR)/*.o 2>/dev/null | head -1 | xargs -I {} cp {} rust.o 2>/dev/null || true
	@echo "Checking for object files..."
	@ls -la $(TARGET_DIR)/ | head -10

qemu: os.elf
	@echo "Press Ctrl-A and then X to exit QEMU"
	$(QEMU) $(QFLAGS) -kernel os.elf

clean:
	rm -f *.o *.elf 2>/dev/null
	cargo clean