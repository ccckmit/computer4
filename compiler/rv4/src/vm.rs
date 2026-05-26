use crate::memory::Memory;

pub struct Vm {
    regs: [u64; 32],
    pc: u64,
    is_rv32: bool,
    ecall_exit: bool,
    exit_code: i32,
}

impl Vm {
    pub fn new() -> Self {
        let mut regs = [0u64; 32];
        regs[2] = 0x0ffffff0;
        Vm { regs, pc: 0, is_rv32: false, ecall_exit: false, exit_code: 0 }
    }

    pub fn set_pc(&mut self, pc: u64) { self.pc = pc; }
    pub fn set_rv32(&mut self, v: bool) { self.is_rv32 = v; }
    pub fn exit_code(&self) -> i32 { self.exit_code }

    pub fn run(&mut self, mem: &mut Memory) -> Result<(), String> {
        let mut steps = 0u64;
        loop {
            if self.ecall_exit { return Ok(()); }
            self.step(mem)?;
            steps += 1;
            if self.pc > 0x20000 {
                eprintln!("rv4: PC out of range at step {}: {:#x}", steps, self.pc);
                eprintln!("rv4:  ra={:#x} sp={:#x} a0={:#x} a1={:#x} a2={:#x}", 
                    self.gpr(1), self.gpr(2), self.gpr(10), self.gpr(11), self.gpr(12));
                return Err("PC out of range".to_string());
            }
            if steps > 50000 {
                eprintln!("rv4: exceeded 50000 steps (PC={:#x})", self.pc);
                eprintln!("rv4: registers: a0={:#x} a1={:#x} a2={:#x} a3={:#x} sp={:#x} ra={:#x}", 
                    self.gpr(10), self.gpr(11), self.gpr(12), self.gpr(13), self.gpr(2), self.gpr(1));
                return Err("step limit".to_string());
            }
        }
    }

    fn step(&mut self, mem: &mut Memory) -> Result<(), String> {
        if self.pc & 1 != 0 {
            return Err(format!("Misaligned PC: {:#x}", self.pc));
        }
        static STEP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let s = STEP.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if s < 30 {
            eprintln!("TRACE[{}] PC={:#x} sp={:#x} ra={:#x} a0={:#x}", s, self.pc, self.gpr(2), self.gpr(1), self.gpr(10));
        }
        let low = mem.load16(self.pc)?;
        if low & 0x3 != 0x3 {
            self.exec_compressed(low, mem)?;
        } else {
            let high = mem.load16(self.pc + 2)?;
            self.exec_normal(low as u32 | (high as u32) << 16, mem)?;
        }
        Ok(())
    }

    fn gpr(&self, r: u32) -> u64 {
        if r == 0 { 0 } else { self.regs[r as usize] }
    }

    fn set_gpr(&mut self, r: u32, v: u64) {
        if r != 0 {
            self.regs[r as usize] = if self.is_rv32 { v as u32 as u64 } else { v };
        }
    }

    fn rd(inst: u32) -> u32 { (inst >> 7) & 0x1f }
    fn rs1_val(&self, inst: u32) -> u64 { self.gpr((inst >> 15) & 0x1f) }
    fn rs2_val(&self, inst: u32) -> u64 { self.gpr((inst >> 20) & 0x1f) }

    fn i_imm(inst: u32) -> i64 { ((inst as i32) >> 20) as i64 }

    fn s_imm(inst: u32) -> i64 {
        let v = (((inst >> 25) << 5) | ((inst >> 7) & 0x1f)) as i64;
        (v << 52) >> 52
    }

    fn b_imm(inst: u32) -> i64 {
        let v = (((inst >> 7) & 0x1e) | ((inst >> 20) & 0x7e0) | ((inst << 4) & 0x800) | ((inst >> 19) & 0x1000)) as i64;
        (v << 51) >> 51
    }

    fn u_imm(inst: u32) -> i64 { (inst as i64) & 0xfffff000 }

    fn j_imm(inst: u32) -> i64 {
        let v = (((inst >> 21) & 0x3ff) | ((inst >> 10) & 0x400) | ((inst >> 1) & 0x7f800) | ((inst << 11) & 0x80000000)) as i64;
        (v << 32) >> 32
    }

    fn exec_normal(&mut self, inst: u32, mem: &mut Memory) -> Result<(), String> {
        let opcode = inst & 0x7f;
        static STEP2: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let s = STEP2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if s < 30 {
            eprintln!("TRACE[{}] PC={:#x} inst={:#010x} opcode={:#04x} a0={:#x} a1={:#x} a2={:#x} a3={:#x}", s, self.pc, inst, opcode, self.gpr(10), self.gpr(11), self.gpr(12), self.gpr(13));
        }
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7f;

        match opcode {
            0x37 => { self.set_gpr(Self::rd(inst), Self::u_imm(inst) as u64); self.pc += 4; }
            0x17 => { let v = self.pc.wrapping_add(Self::u_imm(inst) as u64); self.set_gpr(Self::rd(inst), v); self.pc += 4; }
            0x6f => {
                let r = Self::rd(inst);
                let target = self.pc.wrapping_add(Self::j_imm(inst) as u64);
                static J_CNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let _jc = J_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if _jc < 10 { eprintln!("JMP[{}] JAL x{} <- {:#x} -> {:#x}", _jc, r, self.pc + 4, target); }
                self.set_gpr(r, self.pc + 4);
                self.pc = target;
            }
            0x67 => {
                let r = Self::rd(inst);
                let target = (self.rs1_val(inst).wrapping_add(Self::i_imm(inst) as u64)) & !1;
                static JALR_CNT2: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                let _jc = JALR_CNT2.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if _jc < 10 { eprintln!("JMP[{}] JALR x{} <- {:#x} rs1(x{})={:#x} imm={} -> {:#x}", _jc, r, self.pc + 4, (inst>>15)&0x1f, self.rs1_val(inst), Self::i_imm(inst), target); }
                self.set_gpr(r, self.pc + 4);
                self.pc = target;
            }
            0x63 => {
                let rs1 = self.rs1_val(inst);
                let rs2 = self.rs2_val(inst);
                let imm = Self::b_imm(inst);
                let taken = match funct3 {
                    0 => rs1 == rs2, 1 => rs1 != rs2,
                    4 => (rs1 as i64) < (rs2 as i64), 5 => (rs1 as i64) >= (rs2 as i64),
                    6 => rs1 < rs2, 7 => rs1 >= rs2,
                    _ => return Err(format!("bad branch funct3 {}", funct3)),
                };
                self.pc = if taken { self.pc.wrapping_add(imm as u64) } else { self.pc + 4 };
            }
            0x03 => {
                let r = Self::rd(inst);
                let a = self.rs1_val(inst).wrapping_add(Self::i_imm(inst) as u64);
                let v = match funct3 {
                    0 => sext8(mem.load8(a)?), 1 => sext16(mem.load16(a)?),
                    2 => sext32(mem.load32(a)?),
                    3 if !self.is_rv32 => mem.load64(a)?,
                    4 => mem.load8(a)? as u64, 5 => mem.load16(a)? as u64,
                    6 => mem.load32(a)? as u64,
                    _ => return Err(format!("bad load funct3 {}", funct3)),
                };
                self.set_gpr(r, v);
                self.pc += 4;
            }
            0x23 => {
                let a = self.rs1_val(inst).wrapping_add(Self::s_imm(inst) as u64);
                let v = self.rs2_val(inst);
                match funct3 {
                    0 => mem.store8(a, v as u8)?, 1 => mem.store16(a, v as u16)?,
                    2 => mem.store32(a, v as u32)?,
                    3 if !self.is_rv32 => mem.store64(a, v)?,
                    _ => return Err(format!("bad store funct3 {}", funct3)),
                }
                self.pc += 4;
            }
            0x13 => {
                let r = Self::rd(inst);
                let rs1 = self.rs1_val(inst);
                let imm = Self::i_imm(inst);
                let sh = if self.is_rv32 { imm as u32 & 0x1f } else { imm as u32 & 0x3f } as u64;
                let v = match funct3 {
                    0 => rs1.wrapping_add(imm as u64),
                    1 => rs1 << sh,
                    2 => if (rs1 as i64) < imm { 1 } else { 0 },
                    3 => if rs1 < (imm as u64) { 1 } else { 0 },
                    4 => rs1 ^ (imm as u64),
                    5 => if inst & 0x40000000 != 0 { ((rs1 as i64) >> sh) as u64 } else { rs1 >> sh },
                    6 => rs1 | (imm as u64), 7 => rs1 & (imm as u64),
                    _ => return Err(format!("bad op-imm funct3 {}", funct3)),
                };
                self.set_gpr(r, v);
                self.pc += 4;
            }
            0x1b => {
                if self.is_rv32 { return Err("OP-IMM-32 in RV32".to_string()); }
                let r = Self::rd(inst);
                let rs1 = self.rs1_val(inst) as i32;
                let imm = Self::i_imm(inst) as i32;
                let sh = imm & 0x1f;
                let v = match funct3 {
                    0 => rs1.wrapping_add(imm),
                    1 => rs1 << sh,
                    5 => if inst & 0x40000000 != 0 { rs1 >> sh } else { (rs1 as u32 >> sh) as i32 },
                    _ => return Err(format!("bad op-imm-32 funct3 {}", funct3)),
                };
                self.set_gpr(r, v as i64 as u64);
                self.pc += 4;
            }
            0x33 => {
                let r = Self::rd(inst);
                let rs1 = self.rs1_val(inst);
                let rs2 = self.rs2_val(inst);
                if funct7 & 0x01 != 0 && funct7 >> 1 == 0 {
                    return self.exec_mul(r, funct3, rs1, rs2);
                }
                let sub = funct7 & 0x20 != 0;
                let mask = if self.is_rv32 { 0x1f } else { 0x3f };
                let v = match funct3 {
                    0 => if sub { rs1.wrapping_sub(rs2) } else { rs1.wrapping_add(rs2) },
                    1 => rs1 << (rs2 & mask),
                    2 => if (rs1 as i64) < (rs2 as i64) { 1 } else { 0 },
                    3 => if rs1 < rs2 { 1 } else { 0 },
                    4 => rs1 ^ rs2,
                    5 => if sub { ((rs1 as i64) >> (rs2 & mask)) as u64 } else { rs1 >> (rs2 & mask) },
                    6 => rs1 | rs2, 7 => rs1 & rs2,
                    _ => return Err(format!("bad op funct3 {}", funct3)),
                };
                self.set_gpr(r, v);
                self.pc += 4;
            }
            0x3b => {
                if self.is_rv32 { return Err("OP-32 in RV32".to_string()); }
                let r = Self::rd(inst);
                let rs1 = self.rs1_val(inst) as i32;
                let rs2 = self.rs2_val(inst) as i32;
                if funct7 & 0x01 != 0 && funct7 >> 1 == 0 {
                    return self.exec_mulw(r, funct3, rs1 as u64 as u32, rs2 as u64 as u32);
                }
                let sub = funct7 & 0x20 != 0;
                let sh = rs2 as u32 & 0x1f;
                let v = match funct3 {
                    0 => if sub { rs1.wrapping_sub(rs2) } else { rs1.wrapping_add(rs2) },
                    1 => rs1 << sh,
                    5 => if sub { rs1 >> sh } else { ((rs1 as u32) >> sh) as i32 },
                    _ => return Err(format!("bad op-32 funct3 {}", funct3)),
                };
                self.set_gpr(r, v as i64 as u64);
                self.pc += 4;
            }
            0x0f => { self.pc += 4; }
            0x73 => {
                if funct3 == 0 {
                    let bits = (inst >> 20) & 0xfff;
                    if bits == 0x000 { self.handle_ecall(mem); }
                    else if bits == 0x001 { self.ecall_exit = true; self.exit_code = 0; }
                    else { self.pc += 4; }
                } else { self.pc += 4; }
            }
            _ => return Err(format!("Unimplemented opcode {:#04x} at PC {:#x}", opcode, self.pc)),
        }
        Ok(())
    }

    fn exec_mul(&mut self, rd: u32, funct3: u32, rs1: u64, rs2: u64) -> Result<(), String> {
        let v = match funct3 {
            0 => rs1.wrapping_mul(rs2),
            1 => ((rs1 as i128).wrapping_mul(rs2 as i128) >> 64) as u64,
            2 => ((rs1 as u128).wrapping_mul(rs2 as u128) >> 64) as u64,
            3 => ((rs1 as i128).wrapping_mul(rs2 as i128) >> 64) as u64,
            4 => if rs2 == 0 { 0 } else { (rs1 as i64).wrapping_div(rs2 as i64) as u64 },
            5 => if rs2 == 0 { u64::MAX } else { rs1 / rs2 },
            6 => if rs2 == 0 { rs1 } else { (rs1 as i64).wrapping_rem(rs2 as i64) as u64 },
            7 => if rs2 == 0 { rs1 } else { rs1 % rs2 },
            _ => return Err(format!("bad mul funct3 {}", funct3)),
        };
        self.set_gpr(rd, v);
        self.pc += 4;
        Ok(())
    }

    fn exec_mulw(&mut self, rd: u32, funct3: u32, rs1: u32, rs2: u32) -> Result<(), String> {
        let v: i32 = match funct3 {
            0 => (rs1 as i32).wrapping_mul(rs2 as i32),
            4 => if rs2 == 0 { 0 } else { (rs1 as i32).wrapping_div(rs2 as i32) },
            5 => if rs2 == 0 { 0 } else { (rs1 / rs2) as i32 },
            6 => if rs2 == 0 { rs1 as i32 } else { (rs1 as i32).wrapping_rem(rs2 as i32) },
            7 => if rs2 == 0 { rs1 as i32 } else { (rs1 % rs2) as i32 },
            _ => return Err(format!("bad mulw funct3 {}", funct3)),
        };
        self.set_gpr(rd, v as i64 as u64);
        self.pc += 4;
        Ok(())
    }

    fn handle_ecall(&mut self, mem: &mut Memory) {
        match self.gpr(17) {
            0 => { self.exit_code = self.gpr(10) as i32; self.ecall_exit = true; }
            1 => { print!("{}", self.gpr(10) as u8 as char); self.pc += 4; }
            2 => {
                let addr = self.gpr(10);
                let len = self.gpr(11) as usize;
                for i in 0..len {
                    match mem.load8(addr + i as u64) {
                        Ok(b) => print!("{}", b as char),
                        Err(_) => break,
                    }
                }
                self.pc += 4;
            }
            _ => { eprintln!("rv4: unknown ECALL a7={}", self.gpr(17)); self.ecall_exit = true; self.exit_code = 1; }
        }
    }

    fn exec_compressed(&mut self, inst: u16, mem: &mut Memory) -> Result<(), String> {
        let op = inst & 0x3;
        let funct3 = ((inst >> 13) & 0x7) as u32;
        let rd = ((inst >> 7) & 0x1f) as u32;
        let rs2 = ((inst >> 2) & 0x1f) as u32;
        let rdp = (((inst >> 2) & 0x7) + 8) as u32;
        let rs1p = (((inst >> 7) & 0x7) + 8) as u32;
        let rs2p = (((inst >> 2) & 0x7) + 8) as u32;

        match op {
            0 => match funct3 {
                0 => {
                    let nzuimm_9_2 = ((inst >> 7) & 0x0f) << 4
                                    | ((inst >> 11) & 0x03) << 2
                                    | ((inst >> 5) & 0x01) << 1
                                    | ((inst >> 6) & 0x01);
                    let imm = nzuimm_9_2 as u64 * 4;
                    if imm != 0 { self.set_gpr(rdp, self.gpr(2) + imm); }
                    self.pc += 2;
                }
                1 => { self.pc += 2; }
                2 => {
                    let imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 0x4) | ((inst >> 7) & 0x4)) as u64;
                    self.set_gpr(rdp, sext32(mem.load32(self.gpr(rs1p) + imm)?));
                    self.pc += 2;
                }
                3 => {
                    let imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 0x38) | ((inst >> 7) & 0x8)) as u64;
                    self.set_gpr(rdp, mem.load64(self.gpr(rs1p) + imm)?);
                    self.pc += 2;
                }
                4 | 5 => { self.pc += 2; }
                6 => {
                    let imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 0x4) | ((inst >> 7) & 0x4)) as u64;
                    mem.store32(self.gpr(rs1p) + imm, self.gpr(rs2p) as u32)?;
                    self.pc += 2;
                }
                7 => {
                    let imm = (((inst >> 5) & 0x38) | ((inst >> 1) & 0x38) | ((inst >> 7) & 0x8)) as u64;
                    mem.store64(self.gpr(rs1p) + imm, self.gpr(rs2p))?;
                    self.pc += 2;
                }
                _ => { self.pc += 2; }
            },
            1 => match funct3 {
                0 => {
                    let imm = sext6(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f));
                    if rd != 0 { self.set_gpr(rd, self.gpr(rd).wrapping_add(imm as u64)); }
                    self.pc += 2;
                }
                1 => {
                    if self.is_rv32 {
                        self.set_gpr(1, self.pc + 2);
                        self.pc = self.pc.wrapping_add(cj_imm(inst));
                    } else {
                        let imm = sext6(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f));
                        self.set_gpr(rd, (self.gpr(rd) as i32).wrapping_add(imm as i32) as i64 as u64);
                        self.pc += 2;
                    }
                }
                2 => {
                    let imm = sext6(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f));
                    self.set_gpr(rd, imm as u64);
                    self.pc += 2;
                }
                3 => {
                    if rd == 2 {
                        let imm6 = u32::from((inst >> 12) & 0x1) << 5
                                 | u32::from((inst >> 4) & 0x1) << 4
                                 | u32::from((inst >> 3) & 0x1) << 3
                                 | u32::from((inst >> 5) & 0x1) << 2
                                 | u32::from((inst >> 2) & 0x1) << 1
                                 | u32::from((inst >> 6) & 0x1);
                        let imm = (i64::from((imm6 << 26) as i32 >> 26)) * 16;
                        self.set_gpr(2, self.gpr(2).wrapping_add(imm as u64));
                    } else if rd != 0 {
                        let imm = sext6(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f));
                        self.set_gpr(rd, (imm << 12) as u64);
                    }
                    self.pc += 2;
                }
                4 => {
                    let sa = (inst >> 5) & 0x3;
                    let fa = (inst >> 10) & 0x3;
                    let rv1 = self.gpr(rs1p);
                    let rv2 = self.gpr(rs2p);
                    match (fa as u8, sa as u8) {
                        (0, 0) => { let sh = ((inst >> 7) & 0x3c | (inst >> 2) & 0x03) as u64; self.set_gpr(rs1p, rv1 >> sh); self.pc += 2; }
                        (0, 1) => { let sh = ((inst >> 7) & 0x3c | (inst >> 2) & 0x03) as u64; self.set_gpr(rs1p, (rv1 as i64 >> sh) as u64); self.pc += 2; }
                        (1, 0) => { let imm = sext6(((inst >> 7) & 0x20) | ((inst >> 2) & 0x1f)); self.set_gpr(rs1p, rv1 & (imm as u64)); self.pc += 2; }
                        (2, 0) => { self.set_gpr(rs1p, rv1.wrapping_sub(rv2)); self.pc += 2; }
                        (2, 1) => { self.set_gpr(rs1p, rv1 ^ rv2); self.pc += 2; }
                        (2, 2) => { self.set_gpr(rs1p, rv1 | rv2); self.pc += 2; }
                        (2, 3) => { self.set_gpr(rs1p, rv1 & rv2); self.pc += 2; }
                        (3, 0) => { self.set_gpr(rs1p, (rv1 as i32).wrapping_sub(rv2 as i32) as i64 as u64); self.pc += 2; }
                        (3, 1) => { self.set_gpr(rs1p, (rv1 as i32).wrapping_add(rv2 as i32) as i64 as u64); self.pc += 2; }
                        _ => { self.pc += 2; }
                    }
                }
                5 => { self.pc = self.pc.wrapping_add(cj_imm(inst)); }
                6 => { if self.gpr(rs1p) == 0 { self.pc = self.pc.wrapping_add(cb_imm(inst)); } else { self.pc += 2; } }
                7 => { if self.gpr(rs1p) != 0 { self.pc = self.pc.wrapping_add(cb_imm(inst)); } else { self.pc += 2; } }
                _ => { self.pc += 2; }
            },
            2 => match funct3 {
                0 => {
                    let sh = ((inst >> 7) & 0x3c | (inst >> 2) & 0x03) as u64;
                    if rd != 0 { self.set_gpr(rd, self.gpr(rd) << sh); }
                    self.pc += 2;
                }
                1 => { self.pc += 2; }
                2 => {
                    let imm = (((inst >> 7) & 0x20) | ((inst >> 2) & 0x1c) | ((inst >> 4) & 0x4)) as u64;
                    let v = sext32(mem.load32(self.gpr(2) + imm)?);
                    if rd != 0 { self.set_gpr(rd, v); }
                    self.pc += 2;
                }
                3 => {
                    let imm = (((inst >> 7) & 0x20) | ((inst >> 2) & 0x38) | ((inst >> 4) & 0x7)) as u64;
                    let v = mem.load64(self.gpr(2) + imm)?;
                    if rd != 0 { self.set_gpr(rd, v); }
                    self.pc += 2;
                }
                4 => {
                    let fa = (inst >> 12) & 1;
                    let r1 = ((inst >> 7) & 0x1f) as u32;
                    let r2 = ((inst >> 2) & 0x1f) as u32;
                    match (fa, r1 != 0, r2 != 0) {
                        (0, true, false) => {
                            static JR_CNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            let _jc = JR_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if _jc < 10 { eprintln!("JMP[{}] C.JR x{} from PC={:#x} -> {:#x}", _jc, r1, self.pc, self.gpr(r1) & !1); }
                            self.pc = self.gpr(r1) & !1;
                        }
                        (0, false, false) => { self.ecall_exit = true; self.exit_code = 0; }
                        (1, true, false) => {
                            static JALR_CNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            let _jc = JALR_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if _jc < 10 { eprintln!("JMP[{}] C.JALR x{} from PC={:#x} -> {:#x} (ret={:#x})", _jc, r1, self.pc, self.gpr(r1) & !1, self.pc + 2); }
                            let n = self.pc + 2; self.pc = self.gpr(r1) & !1; self.set_gpr(1, n);
                        }
                        (0, true, true) => {
                            static MV_CNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                            let _mc = MV_CNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                            if _mc < 10 { eprintln!("JMP[{}] C.MV x{}=x{} (x{}) from PC={:#x}", _mc, r1, r2, self.gpr(r2), self.pc); }
                            self.set_gpr(r1, self.gpr(r2)); self.pc += 2;
                        }
                        (1, true, true) => { self.set_gpr(r1, self.gpr(r1).wrapping_add(self.gpr(r2))); self.pc += 2; }
                        _ => { self.pc += 2; }
                    }
                }
                5 => { self.pc += 2; }
                6 => {
                    let imm = (((inst >> 7) & 0x3c) | ((inst >> 2) & 0x3)) as u64;
                    mem.store32(self.gpr(2) + imm, self.gpr(rs2) as u32)?;
                    self.pc += 2;
                }
                7 => {
                    let imm = (((inst >> 7) & 0x3e) | ((inst >> 2) & 0x1f)) as u64;
                    mem.store64(self.gpr(2) + imm, self.gpr(rs2))?;
                    self.pc += 2;
                }
                _ => { self.pc += 2; }
            },
            _ => { self.pc += 2; }
        }
        Ok(())
    }
}

fn sext8(v: u8) -> u64 { (v as i8) as i64 as u64 }
fn sext16(v: u16) -> u64 { (v as i16) as i64 as u64 }
fn sext32(v: u32) -> u64 { (v as i32) as i64 as u64 }
fn sext6(v: u16) -> i64 { ((v as i64) << 58) >> 58 }
fn cj_imm(inst: u16) -> u64 {
    let mut v = 0u64;
    v |= (((inst >> 3) & 0x7) as u64) << 1;
    v |= (((inst >> 11) & 1) as u64) << 4;
    v |= (((inst >> 2) & 1) as u64) << 5;
    v |= (((inst >> 7) & 1) as u64) << 6;
    v |= (((inst >> 6) & 1) as u64) << 7;
    v |= (((inst >> 9) & 0x3) as u64) << 8;
    v |= (((inst >> 8) & 1) as u64) << 10;
    if inst & 0x1000 != 0 { v.wrapping_sub(2048) } else { v }
}

fn cb_imm(inst: u16) -> u64 {
    let mut v = 0u64;
    v |= ((inst as u64 >> 12) & 1) << 8;
    v |= ((inst as u64 >> 6) & 1) << 7;
    v |= ((inst as u64 >> 5) & 1) << 6;
    v |= ((inst as u64 >> 2) & 1) << 5;
    v |= ((inst as u64 >> 11) & 1) << 4;
    v |= ((inst as u64 >> 10) & 1) << 3;
    v |= ((inst as u64 >> 4) & 1) << 2;
    v |= ((inst as u64 >> 3) & 1) << 1;
    if v & 0x100 != 0 { v | 0xfffffffffffffe00 } else { v }
}
