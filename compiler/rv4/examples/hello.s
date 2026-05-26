	.attribute	4, 16
	.attribute	5, "rv64i2p1_m2p0_a2p1_c2p0_zmmul1p0_zaamo1p0_zalrsc1p0_zca1p0"
	.file	"hello.12bf9e1e413be47-cgu.0"
	.section	.text._RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,"ax",@progbits
	.hidden	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.globl	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind
	.p2align	1
	.type	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind,@function
_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind:
.LBB0_1:
	j	.LBB0_1
.Lfunc_end0:
	.size	_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind, .Lfunc_end0-_RNvCseLSQwpavqd5_7___rustc17rust_begin_unwind

	.section	.text._start,"ax",@progbits
	.globl	_start
	.p2align	1
	.type	_start,@function
_start:
.Lpcrel_hi0:
	auipc	a0, %pcrel_hi(.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0)
	li	a1, 14
	addi	a0, a0, %pcrel_lo(.Lpcrel_hi0)
	li	a7, 2
	#APP
	ecall
	#NO_APP
	li	a7, 0
	li	a0, 0
	#APP
	ecall
	#NO_APP
	unimp
.Lfunc_end1:
	.size	_start, .Lfunc_end1-_start

	.type	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0,@object
	.section	.rodata..Lanon.6b24fb73206f11c82a07c17f8b74af2c.0,"a",@progbits
.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0:
	.ascii	"Hello, World!\n"
	.size	.Lanon.6b24fb73206f11c82a07c17f8b74af2c.0, 14

	.ident	"rustc version 1.98.0-nightly (23a3312d9 2026-05-23)"
	.section	".note.GNU-stack","",@progbits
