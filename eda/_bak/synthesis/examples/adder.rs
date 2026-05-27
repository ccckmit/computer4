use synthesis::{Module, Direction, BitWidth, Expression, BinaryOp, synthesize};

fn main() {
    println!("=== 4-bit Adder Example ===\n");

    let mod_adder = Module::new("adder4b")
        .port("a", Direction::Input, BitWidth::Bits(4))
        .port("b", Direction::Input, BitWidth::Bits(4))
        .port("cin", Direction::Input, BitWidth::Bit)
        .port("sum", Direction::Output, BitWidth::Bits(4))
        .port("cout", Direction::Output, BitWidth::Bit)
        .signal("sum_wire", BitWidth::Bits(4))
        .signal("carry", BitWidth::Bit)
        .assign(
            Expression::Signal("sum".to_string()),
            Expression::BinaryOp(
                BinaryOp::Add,
                Box::new(Expression::Signal("a".to_string())),
                Box::new(Expression::Signal("b".to_string())),
            ),
        )
        .assign(
            Expression::Signal("cout".to_string()),
            Expression::Signal("carry".to_string()),
        );

    let result = synthesize(mod_adder);
    println!("Module: {}", result.name);
    println!("Ports: {:?}", result.ports.iter().map(|p| &p.name).collect::<Vec<_>>());
    println!("Cells: {}", result.cells.len());
    println!("Nets: {}", result.nets.len());
    println!();
    println!("{}", result.print_verilog());
}