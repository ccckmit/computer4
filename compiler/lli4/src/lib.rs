pub mod ir;
pub mod parser;
pub mod interp;

pub fn interpret(source: &str) -> String {
    let prog = parser::parse_ir(source);
    interp::run_program(prog)
}
