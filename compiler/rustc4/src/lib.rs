pub mod ast;
pub mod lexer;
pub mod parser;
pub mod codegen;

pub fn compile(source: &str) -> String {
    codegen::compile_to_ir(source)
}
