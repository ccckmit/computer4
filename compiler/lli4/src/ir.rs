#[derive(Debug, Clone, PartialEq)]
pub enum LlvmType {
    I8, I32, I1, Void,
    Pointer(Box<LlvmType>),
    Array(u64, Box<LlvmType>),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Int(i64),
    Bool(bool),
    Local(String),
    Global(String),
}

#[derive(Debug, Clone)]
pub enum ICmpCond { Eq, Ne, Slt, Sgt, Sle, Sge }

#[derive(Debug, Clone)]
pub enum Instruction {
    Alloca { result: String, ty: LlvmType },
    Store { ty: LlvmType, val: Operand, ptr: Operand },
    Load { result: String, ty: LlvmType, ptr: Operand },
    Add { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    Sub { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    Mul { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    SDiv { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    SRem { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    ICmp { result: String, cond: ICmpCond, ty: LlvmType, lhs: Operand, rhs: Operand },
    And { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    Or { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    Xor { result: String, ty: LlvmType, lhs: Operand, rhs: Operand },
    Call { result: Option<String>, ret_ty: LlvmType, name: String, args: Vec<Operand> },
    Ret { val: Option<Operand> },
    Br(String),
    BrCond(Operand, String, String),
    GetElementPtr { result: String, ptr: Operand, indices: Vec<Operand> },
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub label: String,
    pub instrs: Vec<Instruction>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<(String, LlvmType)>,
    pub ret_ty: LlvmType,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub name: String,
    pub ty: LlvmType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub globals: Vec<GlobalVar>,
    pub functions: Vec<FnDecl>,
}
