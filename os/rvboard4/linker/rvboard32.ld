OUTPUT_ARCH(riscv)
ENTRY(_start)

MEMORY {
    ROM (rxai!w) : ORIGIN = 0x80200000, LENGTH = 128K
    RAM (rwxai!w) : ORIGIN = 0x80200000, LENGTH = 128K
}

REGION_ALIAS("text", ROM);
REGION_ALIAS("rodata", ROM);
REGION_ALIAS("data", RAM);
REGION_ALIAS("bss", RAM);

SECTIONS {
    .text.start : {
        *(.text.start)
    } > text

    .text : {
        *(.text)
    } > text

    .rodata : {
        *(.rodata)
    } > rodata

    .data : {
        *(.data)
    } > data

    .bss : {
        *(.bss)
    } > bss

    .stack : {
        . = ALIGN(16);
        _stack_end = .;
    } > bss
}