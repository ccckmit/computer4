	.attribute	4, 16
	.attribute	5, "rv64i2p1_m2p0_a2p1_c2p0_zmmul1p0_zaamo1p0_zalrsc1p0_zca1p0"
	.file	"sum.40457018e185a909-cgu.0"
	.section	.text._RNvCs5w75pDFHJlB_3sum6putdec,"ax",@progbits
	.p2align	1
	.type	_RNvCs5w75pDFHJlB_3sum6putdec,@function
_RNvCs5w75pDFHJlB_3sum6putdec:
	addi	sp, sp, -32
	sd	zero, 8(sp)
	sd	zero, 16(sp)
	sw	zero, 24(sp)
	beqz	a0, .LBB0_5
	li	a1, 0
	addi	a2, sp, 28
	li	a6, 19
	li	a4, 10
	beqz	a0, .LBB0_4
.LBB0_2:
	addi	a5, a1, 19
	bltu	a6, a5, .LBB0_7
	divu	a5, a0, a4
	mul	a3, a5, a4
	sub	a0, a0, a3
	ori	a0, a0, 48
	sb	a0, -1(a2)
	addi	a2, a2, -1
	addi	a1, a1, -1
	mv	a0, a5
	bnez	a5, .LBB0_2
.LBB0_4:
	neg	a1, a1
	li	a7, 2
	mv	a0, a2
	#APP
	ecall
	#NO_APP
	j	.LBB0_6
.LBB0_5:
	li	a0, 48
	li	a7, 1
	#APP
	ecall
	#NO_APP
.LBB0_6:
	addi	sp, sp, 32
	ret
.LBB0_7:
.Lpcrel_hi0:
	auipc	a1, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1)
	li	a0, -1
	addi	a2, a1, %pcrel_lo(.Lpcrel_hi0)
	li	a1, 20
	call	_RNvNtCshql4lZlyZYo_4core9panicking18panic_bounds_check
.Lfunc_end0:
	.size	_RNvCs5w75pDFHJlB_3sum6putdec, .Lfunc_end0-_RNvCs5w75pDFHJlB_3sum6putdec

	.section	.text._RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,"ax",@progbits
	.hidden	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.globl	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.p2align	1
	.type	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,@function
_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind:
.LBB1_1:
	j	.LBB1_1
.Lfunc_end1:
	.size	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind, .Lfunc_end1-_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind

	.section	.text._start,"ax",@progbits
	.globl	_start
	.p2align	1
	.type	_start,@function
_start:
	addi	sp, sp, -16
	sd	ra, 8(sp)
.Lpcrel_hi1:
	auipc	a0, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2)
	li	a1, 14
	addi	a0, a0, %pcrel_lo(.Lpcrel_hi1)
	li	a7, 2
	#APP
	ecall
	#NO_APP
	lui	a0, 1
	addi	a0, a0, 954
	call	_RNvCs5w75pDFHJlB_3sum6putdec
	li	a0, 10
	li	a7, 1
	#APP
	ecall
	#NO_APP
	li	a0, 0
	li	a7, 0
	#APP
	ecall
	#NO_APP
	unimp
.Lfunc_end2:
	.size	_start, .Lfunc_end2-_start

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0,@object
	.section	.rodata.str1.1,"aMS",@progbits,1
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0:
	.asciz	"examples/sum.rs"
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0, 16

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1,@object
	.section	.rodata..Lanon.6b24fb73206f11c82a07c17f8b74af2c.1,"a",@progbits
	.p2align	3, 0x0
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1:
	.quad	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0
	.asciz	"\017\000\000\000\000\000\000\000\025\000\000\000\t\000\000"
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1, 24

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2,@object
	.section	.rodata..Lanon.6b24fb73206f11c82a07c17f8b74af2c.2,"a",@progbits
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2:
	.ascii	"sum(1..100) = "
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2, 14

	.ident	"rustc version 1.98.0-nightly (23a3312d9 2026-05-23)"
	.section	".note.GNU-stack","",@progbits
