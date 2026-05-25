.equ STACK_SIZE, 8192

.global _start

_start:
    csrr t0, mhartid
    slli t0, t0, 10
    la   sp, stacks + STACK_SIZE
    add  sp, sp, t0

    csrr a0, mhartid
    bnez a0, park

    j    rust_main

park:
    wfi
    j park

stacks:
    .skip STACK_SIZE * 4