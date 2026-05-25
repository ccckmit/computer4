use crate::signal::*;

pub struct CPU {
    pub clk: WireRef,
    program: Vec<u16>,
    regs: [u16; 8],
    pc: u16,
    pub halted: bool,
    prev_clk: Level,
    pub debug_pc: Vec<WireRef>,
    pub debug_ir: Vec<WireRef>,
    pub debug_regs: [Vec<WireRef>; 8],
}

impl CPU {
    pub fn new(program: Vec<u16>, clk: WireRef) -> Self {
        CPU {
            clk,
            program,
            regs: [0; 8],
            pc: 0,
            halted: false,
            prev_clk: Level::L,
            debug_pc: bus("cpu_pc", 16),
            debug_ir: bus("cpu_ir", 16),
            debug_regs: [
                bus("cpu_r0", 16), bus("cpu_r1", 16), bus("cpu_r2", 16), bus("cpu_r3", 16),
                bus("cpu_r4", 16), bus("cpu_r5", 16), bus("cpu_r6", 16), bus("cpu_r7", 16),
            ],
        }
    }

    pub fn eval(&mut self) {
        u16_to_bus(&self.debug_pc, self.pc);
        if (self.pc as usize) < self.program.len() {
            u16_to_bus(&self.debug_ir, self.program[self.pc as usize]);
        }
        for i in 0..8 {
            u16_to_bus(&self.debug_regs[i], self.regs[i]);
        }
    }

    pub fn tick(&mut self) {
        let clk_val = get(&self.clk);
        if self.prev_clk == Level::L && clk_val == Level::H {
            self.execute();
        }
        self.prev_clk = clk_val;
        self.eval();
    }

    pub fn reg(&self, i: usize) -> u16 {
        self.regs[i]
    }

    fn execute(&mut self) {
        if self.halted || (self.pc as usize) >= self.program.len() {
            self.halted = true;
            return;
        }

        let instr = self.program[self.pc as usize];
        let opcode = ((instr >> 12) & 0xF) as u8;

        let mut next_pc = self.pc.wrapping_add(1);

        match opcode {
            0 => {
                let rd = ((instr >> 9) & 0x7) as usize;
                let imm9 = instr & 0x1FF;
                self.regs[rd] = imm9;
            }
            1 | 2 | 3 => {
                let rd = ((instr >> 9) & 0x7) as usize;
                let rs = ((instr >> 6) & 0x7) as usize;
                let rt = ((instr >> 3) & 0x7) as usize;
                let a = self.regs[rs];
                let b = self.regs[rt];
                self.regs[rd] = match opcode {
                    1 => a.wrapping_add(b),
                    2 => a.wrapping_sub(b),
                    3 => a.wrapping_mul(b),
                    _ => unreachable!(),
                };
            }
            4 | 5 => {
                let rs = ((instr >> 9) & 0x7) as usize;
                let rt = ((instr >> 6) & 0x7) as usize;
                let target = instr & 0x3F;
                let eq = self.regs[rs] == self.regs[rt];
                if (opcode == 4 && eq) || (opcode == 5 && !eq) {
                    next_pc = target;
                }
            }
            6 => {
                self.halted = true;
                return;
            }
            _ => {}
        }

        self.pc = next_pc;
    }
}

fn encode_loadi(rd: u16, imm: u16) -> u16 {
    (0 << 12) | ((rd & 0x7) << 9) | (imm & 0x1FF)
}

fn encode_r(op: u16, rd: u16, rs: u16, rt: u16) -> u16 {
    (op << 12) | ((rd & 0x7) << 9) | ((rs & 0x7) << 6) | ((rt & 0x7) << 3)
}

fn encode_branch(op: u16, rs: u16, rt: u16, addr: u16) -> u16 {
    (op << 12) | ((rs & 0x7) << 9) | ((rt & 0x7) << 6) | (addr & 0x3F)
}

pub fn program_5factorial() -> Vec<u16> {
    vec![
        encode_loadi(0, 5),  // R0 = 5   (loop counter)
        encode_loadi(1, 1),  // R1 = 1   (accumulator)
        encode_loadi(2, 1),  // R2 = 1   (constant for decrement)
        encode_r(3, 1, 1, 0), // MUL R1, R1, R0
        encode_r(2, 0, 0, 2), // SUB R0, R0, R2
        encode_branch(5, 0, 7, 3), // BNE R0, R7, 3  (loop)
        encode_branch(6, 0, 0, 0), // HLT
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::Sim;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_cpu_5factorial() {
        let mut sim = Sim::new();
        let cpu = Rc::new(RefCell::new(CPU::new(program_5factorial(), sim.get_clock())));
        let cpu_clk = cpu.clone();
        sim.add_seq(move || cpu_clk.borrow_mut().tick());

        sim.run(50);

        let cpu_ref = cpu.borrow();
        assert!(cpu_ref.halted, "CPU should have halted");
        assert_eq!(cpu_ref.reg(1), 120, "5! should be 120");
        assert_eq!(cpu_ref.reg(0), 0, "R0 counter should be 0");
    }

    #[test]
    fn test_cpu_halt_immediately() {
        let mut sim = Sim::new();
        let cpu = Rc::new(RefCell::new(CPU::new(vec![encode_branch(6, 0, 0, 0)], sim.get_clock())));
        let cpu_clk = cpu.clone();
        sim.add_seq(move || cpu_clk.borrow_mut().tick());

        sim.run(5);

        assert!(cpu.borrow().halted);
    }

    #[test]
    fn test_cpu_add_sub() {
        let program = vec![
            encode_loadi(0, 32), // R0 = 32
            encode_loadi(1, 16), // R1 = 16
            encode_r(1, 2, 0, 1), // ADD R2, R0, R1
            encode_r(2, 3, 0, 1), // SUB R3, R0, R1
            encode_branch(6, 0, 0, 0), // HLT
        ];

        let mut sim = Sim::new();
        let cpu = Rc::new(RefCell::new(CPU::new(program, sim.get_clock())));
        let cpu_clk = cpu.clone();
        sim.add_seq(move || cpu_clk.borrow_mut().tick());

        sim.run(20);

        let regs = cpu.borrow();
        assert!(regs.halted);
        assert_eq!(regs.reg(0), 32);
        assert_eq!(regs.reg(1), 16);
        assert_eq!(regs.reg(2), 48);
        assert_eq!(regs.reg(3), 16);
    }

    #[test]
    fn test_cpu_branch_beq() {
        let program = vec![
            encode_branch(4, 0, 7, 2), // BEQ R0, R7, 2 — branch to HLT
            encode_loadi(1, 42),        // skipped
            encode_branch(6, 0, 0, 0), // HLT
        ];

        let mut sim = Sim::new();
        let cpu = Rc::new(RefCell::new(CPU::new(program, sim.get_clock())));
        let cpu_clk = cpu.clone();
        sim.add_seq(move || cpu_clk.borrow_mut().tick());

        sim.run(10);

        let regs = cpu.borrow();
        assert!(regs.halted);
        assert_eq!(regs.reg(1), 0, "LOADI should be skipped by BEQ");
    }

    #[test]
    fn test_cpu_branch_bne() {
        let program = vec![
            encode_loadi(0, 1),          // R0 = 1
            encode_branch(5, 0, 7, 3),  // BNE R0, R7, 3 — branch to skip LOADI
            encode_loadi(1, 42),         // skipped
            encode_branch(6, 0, 0, 0),  // HLT
        ];

        let mut sim = Sim::new();
        let cpu = Rc::new(RefCell::new(CPU::new(program, sim.get_clock())));
        let cpu_clk = cpu.clone();
        sim.add_seq(move || cpu_clk.borrow_mut().tick());

        sim.run(10);

        let regs = cpu.borrow();
        assert!(regs.halted);
        assert_eq!(regs.reg(0), 1);
        assert_eq!(regs.reg(1), 0, "LOADI should be skipped by BNE");
    }
}
