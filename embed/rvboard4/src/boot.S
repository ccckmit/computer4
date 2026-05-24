.section .text.start, "ax"
.global _start
_start:
    lui a0, %hi(msg)
    addi a0, a0, %lo(msg)
    jal ra, print
loop:
    j loop

.global putchar
putchar:
    li a7, 1
    ecall
    ret

.global print
print:
    addi sp, sp, -16
    sw ra, 0(sp)
    mv s0, a0
1:
    lbu a0, 0(s0)
    beqz a0, 2f
    jal ra, putchar
    addi s0, s0, 1
    j 1b
2:
    lw ra, 0(sp)
    addi sp, sp, 16
    ret

.section .rodata
msg:
    .string "Hello rvboard4\n"