use synthesis::{Module, Direction, BitWidth, Expression, Statement, synthesize};

fn main() {
    println!("=== 2:1 Multiplexer Example ===\n");

    let mod_mux = Module::new("mux2to1")
        .port("d0", Direction::Input, BitWidth::Bits(1))
        .port("d1", Direction::Input, BitWidth::Bits(1))
        .port("sel", Direction::Input, BitWidth::Bit)
        .port("y", Direction::Output, BitWidth::Bits(1))
        .assign(
            Expression::Signal("y".to_string()),
            Expression::Signal("d0".to_string()),
        );

    let result = synthesize(mod_mux);
    println!("Module: {}", result.name);
    println!("Ports: {:?}", result.ports.iter().map(|p| &p.name).collect::<Vec<_>>());
    println!("Cells: {}", result.cells.len());
    println!();
    println!("{}", result.print_verilog());
}