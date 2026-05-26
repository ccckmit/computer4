	.attribute	4, 16
	.attribute	5, "rv64i2p1_m2p0_a2p1_c2p0_zmmul1p0_zaamo1p0_zalrsc1p0_zca1p0"
	.file	"fact.298f57c7889ffa8a-cgu.0"
	.section	.text._RNvCs3zdNtEIONsc_4fact4fact,"ax",@progbits
	.p2align	1
	.type	_RNvCs3zdNtEIONsc_4fact4fact,@function
_RNvCs3zdNtEIONsc_4fact4fact:
	mv	a1, a0
	li	a0, 1
	li	a2, 2
	bltu	a1, a2, .LBB0_2
.LBB0_1:
	addi	a3, a1, -1
	mul	a0, a1, a0
	mv	a1, a3
	bgeu	a3, a2, .LBB0_1
.LBB0_2:
	ret
.Lfunc_end0:
	.size	_RNvCs3zdNtEIONsc_4fact4fact, .Lfunc_end0-_RNvCs3zdNtEIONsc_4fact4fact

	.section	.text._RNvCs3zdNtEIONsc_4fact6putdec,"ax",@progbits
	.p2align	1
	.type	_RNvCs3zdNtEIONsc_4fact6putdec,@function
_RNvCs3zdNtEIONsc_4fact6putdec:
	addi	sp, sp, -32
	sd	zero, 8(sp)
	sd	zero, 16(sp)
	sw	zero, 24(sp)
	beqz	a0, .LBB1_5
	li	a1, 0
	addi	a2, sp, 28
	li	a6, 19
	li	a4, 10
	beqz	a0, .LBB1_4
.LBB1_2:
	addi	a5, a1, 19
	bltu	a6, a5, .LBB1_7
	divu	a5, a0, a4
	mul	a3, a5, a4
	sub	a0, a0, a3
	ori	a0, a0, 48
	sb	a0, -1(a2)
	addi	a2, a2, -1
	addi	a1, a1, -1
	mv	a0, a5
	bnez	a5, .LBB1_2
.LBB1_4:
	neg	a1, a1
	li	a7, 2
	mv	a0, a2
	#APP
	ecall
	#NO_APP
	j	.LBB1_6
.LBB1_5:
	li	a0, 48
	li	a7, 1
	#APP
	ecall
	#NO_APP
.LBB1_6:
	addi	sp, sp, 32
	ret
.LBB1_7:
.Lpcrel_hi0:
	auipc	a1, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1)
	li	a0, -1
	addi	a2, a1, %pcrel_lo(.Lpcrel_hi0)
	li	a1, 20
	call	_RNvNtCshql4lZlyZYo_4core9panicking18panic_bounds_check
.Lfunc_end1:
	.size	_RNvCs3zdNtEIONsc_4fact6putdec, .Lfunc_end1-_RNvCs3zdNtEIONsc_4fact6putdec

	.section	.text._RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,"ax",@progbits
	.hidden	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.globl	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.p2align	1
	.type	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,@function
_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind:
.LBB2_1:
	j	.LBB2_1
.Lfunc_end2:
	.size	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind, .Lfunc_end2-_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind

	.section	.text._start,"ax",@progbits
	.globl	_start
	.p2align	1
	.type	_start,@function
_start:
	addi	sp, sp, -16
	sd	ra, 8(sp)
	sd	s0, 0(sp)
	li	a0, 10
	call	_RNvCs3zdNtEIONsc_4fact4fact
	mv	s0, a0
.Lpcrel_hi1:
	auipc	a0, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2)
	li	a1, 5
	addi	a0, a0, %pcrel_lo(.Lpcrel_hi1)
	li	a7, 2
	#APP
	ecall
	#NO_APP
	li	a0, 10
	call	_RNvCs3zdNtEIONsc_4fact6putdec
.Lpcrel_hi2:
	auipc	a0, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.3)
	li	a1, 4
	addi	a0, a0, %pcrel_lo(.Lpcrel_hi2)
	li	a7, 2
	#APP
	ecall
	#NO_APP
	mv	a0, s0
	call	_RNvCs3zdNtEIONsc_4fact6putdec
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
.Lfunc_end3:
	.size	_start, .Lfunc_end3-_start

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0,@object
	.section	.rodata.str1.1,"aMS",@progbits,1
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0:
	.asciz	"examples/fact.rs"
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0, 17

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1,@object
	.section	.rodata..Lanon.6b24fb73206f11c82a07c17f8b74af2c.1,"a",@progbits
	.p2align	3, 0x0
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1:
	.quad	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0
	.asciz	"\020\000\000\000\000\000\000\000\025\000\000\000\t\000\000"
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.1, 24

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2,@object
	.section	.rodata..Lanon.6b24fb73206f11c82a07c17f8b74af2c.2,"a",@progbits
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2:
	.ascii	"fact("
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.2, 5

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.3,@object
	.section	.rodata.cst4,"aM",@progbits,4
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.3:
	.ascii	") = "
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.3, 4

	.ident	"rustc version 1.98.0-nightly (23a3312d9 2026-05-23)"
	.section	".note.GNU-stack","",@progbits
