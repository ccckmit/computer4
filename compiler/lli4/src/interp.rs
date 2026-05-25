use std::collections::HashMap;
use crate::ir::*;

#[derive(Clone)]
struct Frame {
    func_idx: usize,
    block_idx: usize,
    instr_idx: usize,
    locals: HashMap<String, i64>,
    pending_result: Option<String>,
}

pub struct Interp {
    prog: Program,
    frames: Vec<Frame>,
    memory: HashMap<u64, i64>,
    next_addr: u64,
    output: String,
}

impl Interp {
    pub fn new(prog: Program) -> Self {
        Interp {
            prog,
            frames: Vec::new(),
            memory: HashMap::new(),
            next_addr: 1,
            output: String::new(),
        }
    }

    pub fn run(&mut self) {
        let main_idx = self.prog.functions.iter().position(|f| f.name == "main");
        let main_idx = match main_idx {
            Some(i) => i,
            None => return,
        };
        self.frames.push(Frame {
            func_idx: main_idx,
            block_idx: 0,
            instr_idx: 0,
            locals: HashMap::new(),
            pending_result: None,
        });
        self.exec_loop();
    }

    fn exec_loop(&mut self) {
        loop {
            if self.frames.is_empty() { break; }
            let f = self.frames.last().unwrap().clone();
            let func = &self.prog.functions[f.func_idx];
            if f.block_idx >= func.blocks.len() { break; }
            let block = &func.blocks[f.block_idx];
            if f.instr_idx >= block.instrs.len() { break; }

            let instr = block.instrs[f.instr_idx].clone();
            self.exec_instr(&instr);
        }
    }

    fn exec_instr(&mut self, instr: &Instruction) {
        match instr {
            Instruction::Alloca { result, ty: _ } => {
                let addr = self.alloc();
                self.set_local(result, addr as i64);
                self.advance_frame();
            }
            Instruction::Store { val, ptr, .. } => {
                let ptr_val = self.resolve(ptr) as u64;
                let val_val = self.resolve(val);
                self.memory.insert(ptr_val, val_val);
                self.advance_frame();
            }
            Instruction::Load { result, ty: _, ptr } => {
                let ptr_val = self.resolve(ptr) as u64;
                let val = self.memory.get(&ptr_val).copied().unwrap_or(0);
                self.set_local(result, val);
                self.advance_frame();
            }
            Instruction::Add { result, lhs, rhs, .. } => {
                let v = self.resolve(lhs).wrapping_add(self.resolve(rhs));
                self.set_local(result, v);
                self.advance_frame();
            }
            Instruction::Sub { result, lhs, rhs, .. } => {
                let v = self.resolve(lhs).wrapping_sub(self.resolve(rhs));
                self.set_local(result, v);
                self.advance_frame();
            }
            Instruction::Mul { result, lhs, rhs, .. } => {
                let v = self.resolve(lhs).wrapping_mul(self.resolve(rhs));
                self.set_local(result, v);
                self.advance_frame();
            }
            Instruction::SDiv { result, lhs, rhs, .. } => {
                let r = self.resolve(rhs);
                if r == 0 { self.set_local(result, 0); }
                else { self.set_local(result, self.resolve(lhs) / r); }
                self.advance_frame();
            }
            Instruction::SRem { result, lhs, rhs, .. } => {
                let r = self.resolve(rhs);
                if r == 0 { self.set_local(result, 0); }
                else { self.set_local(result, self.resolve(lhs) % r); }
                self.advance_frame();
            }
            Instruction::ICmp { result, cond, lhs, rhs, .. } => {
                let l = self.resolve(lhs);
                let r = self.resolve(rhs);
                let val = match cond {
                    ICmpCond::Eq => l == r,
                    ICmpCond::Ne => l != r,
                    ICmpCond::Slt => l < r,
                    ICmpCond::Sgt => l > r,
                    ICmpCond::Sle => l <= r,
                    ICmpCond::Sge => l >= r,
                };
                self.set_local(result, if val { 1 } else { 0 });
                self.advance_frame();
            }
            Instruction::And { result, lhs, rhs, .. } => {
                self.set_local(result, self.resolve(lhs) & self.resolve(rhs));
                self.advance_frame();
            }
            Instruction::Or { result, lhs, rhs, .. } => {
                self.set_local(result, self.resolve(lhs) | self.resolve(rhs));
                self.advance_frame();
            }
            Instruction::Xor { result, lhs, rhs, .. } => {
                self.set_local(result, self.resolve(lhs) ^ self.resolve(rhs));
                self.advance_frame();
            }
            Instruction::Call { result, name, args, .. } => {
                self.handle_call(result, name, args);
            }
            Instruction::Ret { val } => {
                let ret_val = val.as_ref().map(|v| self.resolve(v));
                self.frames.pop();
                if let Some(caller) = self.frames.last_mut() {
                    if let Some(rv) = ret_val {
                        caller.locals.insert("__retval".into(), rv);
                        if let Some(ref result_name) = caller.pending_result.clone() {
                            caller.locals.insert(result_name.clone(), rv);
                            caller.pending_result = None;
                        }
                    }
                    self.advance_frame_inner();
                }
            }
            Instruction::Br(target) => {
                if let Some(frame) = self.frames.last_mut() {
                    let func = &self.prog.functions[frame.func_idx];
                    if let Some(idx) = func.blocks.iter().position(|b| b.label == *target) {
                        frame.block_idx = idx;
                        frame.instr_idx = 0;
                    }
                }
            }
            Instruction::BrCond(cond, t, f) => {
                let val = self.resolve(cond);
                let target = if val != 0 { t } else { f };
                if let Some(frame) = self.frames.last_mut() {
                    let func = &self.prog.functions[frame.func_idx];
                    if let Some(idx) = func.blocks.iter().position(|b| b.label == *target) {
                        frame.block_idx = idx;
                        frame.instr_idx = 0;
                    }
                }
            }
            Instruction::GetElementPtr { result, ptr, indices } => {
                let ptr_val = self.resolve(ptr);
                if indices.len() == 2 {
                    let idx2 = self.resolve(&indices[1]);
                    self.set_local(result, ptr_val + idx2);
                } else if indices.len() == 1 {
                    let idx = self.resolve(&indices[0]);
                    self.set_local(result, ptr_val + idx);
                } else {
                    self.set_local(result, ptr_val);
                }
                self.advance_frame();
            }
        }
    }

    fn handle_call(&mut self, result: &Option<String>, name: &str, args: &[Operand]) {
        if name == "printf" {
            if args.len() > 1 {
                self.handle_print(&args[1..]);
            }
            if let Some(r) = result {
                self.set_local(r, (args.len() - 1) as i64);
            }
            self.advance_frame();
            return;
        }
        if name == "print_int" {
            if !args.is_empty() {
                self.handle_print(&args[..1]);
            }
            if let Some(r) = result {
                self.set_local(r, 0);
            }
            self.advance_frame();
            return;
        }

        if let Some(frame) = self.frames.last_mut() {
            frame.pending_result = result.clone();
        }
        let func_idx = self.prog.functions.iter().position(|f| f.name == *name);
        if let Some(idx) = func_idx {
            let func = &self.prog.functions[idx];
            let mut locals = HashMap::new();
            for (i, (pname, _)) in func.params.iter().enumerate() {
                if i < args.len() {
                    locals.insert(pname.clone(), self.resolve(&args[i]));
                }
            }
            self.frames.push(Frame {
                func_idx: idx,
                block_idx: 0,
                instr_idx: 0,
                locals,
                pending_result: None,
            });
        } else if let Some(r) = result {
            self.set_local(r, 0);
        }
    }

    fn handle_print(&mut self, args: &[Operand]) {
        for arg in args {
            let val = self.resolve(arg);
            self.output.push_str(&format!("{}\n", val));
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    fn alloc(&mut self) -> u64 {
        let addr = self.next_addr;
        self.next_addr += 1;
        addr
    }

    fn resolve(&self, op: &Operand) -> i64 {
        match op {
            Operand::Int(v) => *v,
            Operand::Bool(b) => if *b { 1 } else { 0 },
            Operand::Local(name) => {
                for frame in self.frames.iter().rev() {
                    if let Some(v) = frame.locals.get(name) {
                        return *v;
                    }
                }
                0
            }
            Operand::Global(_) => 0,
        }
    }

    fn advance_frame(&mut self) {
        if let Some(f) = self.frames.last_mut() {
            f.instr_idx += 1;
            let blk_len = self.prog.functions[f.func_idx].blocks[f.block_idx].instrs.len();
            if f.instr_idx >= blk_len {
                let num_blocks = self.prog.functions[f.func_idx].blocks.len();
                if f.block_idx + 1 < num_blocks {
                    f.block_idx += 1;
                    f.instr_idx = 0;
                }
            }
        }
    }

    fn advance_frame_inner(&mut self) {
        if let Some(f) = self.frames.last_mut() {
            f.instr_idx += 1;
        }
        if let Some(f) = self.frames.last() {
            let blk_len = self.prog.functions[f.func_idx].blocks[f.block_idx].instrs.len();
            if f.instr_idx >= blk_len {
                let num_blocks = self.prog.functions[f.func_idx].blocks.len();
                if f.block_idx + 1 < num_blocks {
                    let f = self.frames.last_mut().unwrap();
                    f.block_idx += 1;
                    f.instr_idx = 0;
                }
            }
        }
    }

    fn set_local(&mut self, name: &str, val: i64) {
        if let Some(frame) = self.frames.last_mut() {
            frame.locals.insert(name.to_string(), val);
        }
    }
}

pub fn run_program(prog: Program) -> String {
    let mut interp = Interp::new(prog);
    interp.run();
    interp.get_output().to_string()
}
