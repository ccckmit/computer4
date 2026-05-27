use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    println!("IC EDA Synthesis Tool v0.1.0");
    println!("Available modes:");
    println!("  --help     Show this help");
    println!("  --bench    Run benchmark synthesis examples");
    println!("  --lib      Show library API reference");

    if args.len() > 1 {
        match args[1].as_str() {
            "--bench" => {
                println!("\nRunning synthesis benchmarks...\n");
                bench_adder();
                bench_counter();
                bench_mux();
            }
            "--lib" => {
                print_lib_ref();
            }
            _ => {}
        }
    }
}

fn bench_adder() {
    use synthesis::{Module, Direction, BitWidth, Expression, BinaryOp, synthesize};

    println!("=== Adder Synthesis ===");
    let mod_adder = Module::new("adder")
        .port("a", Direction::Input, BitWidth::Bits(8))
        .port("b", Direction::Input, BitWidth::Bits(8))
        .port("sum", Direction::Output, BitWidth::Bits(8))
        .assign(
            Expression::Signal("sum".to_string()),
            Expression::BinaryOp(
                BinaryOp::Add,
                Box::new(Expression::Signal("a".to_string())),
                Box::new(Expression::Signal("b".to_string())),
            ),
        );

    let result = synthesize(mod_adder);
    println!("  Module: {}", result.name);
    println!("  Cells: {}", result.cells.len());
    println!("  Nets: {}", result.nets.len());
    println!("  Ports: {}", result.ports.len());
    println!();
}

fn bench_counter() {
    use synthesis::{Module, Direction, BitWidth, Expression, BinaryOp, Statement, SignalKind, AlwaysBlock, synthesize};

    println!("=== Counter Synthesis ===");
    let mod_counter = Module::new("counter")
        .port("clk", Direction::Input, BitWidth::Bit)
        .port("rst", Direction::Input, BitWidth::Bit)
        .port("q", Direction::Output, BitWidth::Bits(8))
        .signal("count_reg", BitWidth::Bits(8))
        .always_posedge("clk", vec![
            Statement::If(
                Expression::Signal("rst".to_string()),
                vec![Statement::Assign(
                    Expression::Signal("count_reg".to_string()),
                    Expression::Literal(synthesis::Literal::Bits(0, 8)),
                )],
                Some(vec![Statement::Assign(
                    Expression::Signal("count_reg".to_string()),
                    Expression::BinaryOp(
                        BinaryOp::Add,
                        Box::new(Expression::Signal("count_reg".to_string())),
                        Box::new(Expression::Literal(synthesis::Literal::Bits(1, 8))),
                    ),
                )]),
            ),
            Statement::Assign(
                Expression::Signal("q".to_string()),
                Expression::Signal("count_reg".to_string()),
            ),
        ]);

    let result = synthesize(mod_counter);
    println!("  Module: {}", result.name);
    println!("  Cells: {}", result.cells.len());
    println!("  Nets: {}", result.nets.len());
    println!();
}

fn bench_mux() {
    use synthesis::{Module, Direction, BitWidth, Expression, Statement, synthesize};

    println!("=== Mux Synthesis ===");
    let mod_mux = Module::new("mux4to1")
        .port("sel", Direction::Input, BitWidth::Bits(2))
        .port("d0", Direction::Input, BitWidth::Bits(4))
        .port("d1", Direction::Input, BitWidth::Bits(4))
        .port("d2", Direction::Input, BitWidth::Bits(4))
        .port("d3", Direction::Input, BitWidth::Bits(4))
        .port("y", Direction::Output, BitWidth::Bits(4))
        .signal("y_int", BitWidth::Bits(4))
        .assign(
            Expression::Signal("y".to_string()),
            Expression::Signal("y_int".to_string()),
        );

    let result = synthesize(mod_mux);
    println!("  Module: {}", result.name);
    println!("  Cells: {}", result.cells.len());
    println!("  Nets: {}", result.nets.len());
    println!();
}

fn print_lib_ref() {
    println!("\n=== Synthesis Library API Reference ===\n");
    println!("Types:");
    println!("  Direction      - Input, Output, Inout");
    println!("  BitWidth       - Bit, Bits(usize)");
    println!("  Expression     - Literal, Signal, BinaryOp, UnaryOp, Concat, etc.");
    println!("  Statement      - Assign, If, Case, For, Block");
    println!("  Module         - HDL module definition");
    println!("  StructElaborated - Post-synthesis netlist");
    println!("  Cell           - Gate-level cell (AND, OR, DFF, etc.)");
    println!("  Net            - Wire/net in the synthesized netlist");
    println!("\nFunctions:");
    println!("  synthesize(Module) -> StructElaborated");
    println!("  Module::new(name) -> Module (builder)");
    println!("  StructElaborated::print_verilog() -> String");
    println!("  StructElaborated::print_dot() -> String");
    println!();
}