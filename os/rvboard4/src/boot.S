.section .text.start, "ax"
.global _start
_start:
    call hello
    # Halt forever
1:
    j 1b

.global putchar
putchar:
    li a7, 1
    ecall
    ret