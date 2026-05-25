use synthesis::{Module, Direction, BitWidth, Expression, BinaryOp, Literal, Statement, synthesize};

fn main() {
    println!("=== 8-bit Up Counter Example ===\n");

    let mod_counter = Module::new("counter8b")
        .port("clk", Direction::Input, BitWidth::Bit)
        .port("rst", Direction::Input, BitWidth::Bit)
        .port("en", Direction::Input, BitWidth::Bit)
        .port("q", Direction::Output, BitWidth::Bits(8))
        .signal("count_reg", BitWidth::Bits(8))
        .always_posedge("clk", vec![
            Statement::If(
                Expression::Signal("rst".to_string()),
                vec![Statement::Assign(
                    Expression::Signal("count_reg".to_string()),
                    Expression::Literal(Literal::Bits(0, 8)),
                )],
                Some(vec![Statement::If(
                    Expression::Signal("en".to_string()),
                    vec![Statement::Assign(
                        Expression::Signal("count_reg".to_string()),
                        Expression::BinaryOp(
                            BinaryOp::Add,
                            Box::new(Expression::Signal("count_reg".to_string())),
                            Box::new(Expression::Literal(Literal::Bits(1, 8))),
                        ),
                    )],
                    None,
                )]),
            ),
            Statement::Assign(
                Expression::Signal("q".to_string()),
                Expression::Signal("count_reg".to_string()),
            ),
        ]);

    let result = synthesize(mod_counter);
    println!("Module: {}", result.name);
    println!("Ports: {:?}", result.ports.iter().map(|p| &p.name).collect::<Vec<_>>());
    println!("Cells: {}", result.cells.len());
    println!("Nets: {}", result.nets.len());
    println!("Always blocks: 1");
    println!();
    println!("{}", result.print_verilog());
    println!("{}", result.print_dot());
}