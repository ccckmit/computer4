use synthesis::{
    Module, Direction, BitWidth, Expression, BinaryOp, Statement,
    Literal, Elaborator, Optimizer, TechMapper, Cell, Net,
};

#[test]
fn test_synthesis_adder() {
    let mod_ = Module::new("test_adder")
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

    let elab = {
        let mut e = Elaborator::new();
        e.elaborate(&mod_)
    };
    let opt = Optimizer::optimize(&elab);
    let mapped = TechMapper::map(&opt);

    assert_eq!(mapped.name, "test_adder");
    assert_eq!(mapped.ports.len(), 3);
}

#[test]
fn test_synthesis_counter() {
    let mod_ = Module::new("test_counter")
        .port("clk", Direction::Input, BitWidth::Bit)
        .port("rst", Direction::Input, BitWidth::Bit)
        .port("q", Direction::Output, BitWidth::Bits(8))
        .signal("cnt", BitWidth::Bits(8))
        .always_posedge("clk", vec![
            Statement::If(
                Expression::Signal("rst".to_string()),
                vec![Statement::Assign(
                    Expression::Signal("cnt".to_string()),
                    Expression::Literal(Literal::Bits(0, 8)),
                )],
                Some(vec![Statement::Assign(
                    Expression::Signal("cnt".to_string()),
                    Expression::BinaryOp(
                        BinaryOp::Add,
                        Box::new(Expression::Signal("cnt".to_string())),
                        Box::new(Expression::Literal(Literal::Bits(1, 8))),
                    ),
                )]),
            ),
            Statement::Assign(
                Expression::Signal("q".to_string()),
                Expression::Signal("cnt".to_string()),
            ),
        ]);

    let elab = {
        let mut e = Elaborator::new();
        e.elaborate(&mod_)
    };
    let opt = Optimizer::optimize(&elab);
    let mapped = TechMapper::map(&opt);

    assert_eq!(mapped.name, "test_counter");
    assert_eq!(mapped.ports.len(), 3);
    assert!(mapped.cells.len() > 0);
}

#[test]
fn test_synthesis_mux() {
    let mod_ = Module::new("test_mux")
        .port("sel", Direction::Input, BitWidth::Bit)
        .port("a", Direction::Input, BitWidth::Bits(4))
        .port("b", Direction::Input, BitWidth::Bits(4))
        .port("y", Direction::Output, BitWidth::Bits(4))
        .always_posedge("sel", vec![
            Statement::If(
                Expression::Signal("sel".to_string()),
                vec![Statement::Assign(
                    Expression::Signal("y".to_string()),
                    Expression::Signal("a".to_string()),
                )],
                Some(vec![Statement::Assign(
                    Expression::Signal("y".to_string()),
                    Expression::Signal("b".to_string()),
                )]),
            )
        ]);

    let elab = {
        let mut e = Elaborator::new();
        e.elaborate(&mod_)
    };
    let opt = Optimizer::optimize(&elab);
    let mapped = TechMapper::map(&opt);

    assert_eq!(mapped.name, "test_mux");
}

#[test]
fn test_net_width() {
    let n1 = Net::new("w1");
    assert_eq!(n1.width, 1);

    let n4 = Net::with_width("w4", 4);
    assert_eq!(n4.width, 4);
}

#[test]
fn test_bitwidth() {
    assert_eq!(BitWidth::Bit.width(), 1);
    assert_eq!(BitWidth::Bits(8).width(), 8);
    assert_eq!(BitWidth::Bits(32).width(), 32);
}

#[test]
fn test_cell_kinds() {
    let c1 = Cell::and("and0",
        Expression::Signal("a".to_string()),
        Expression::Signal("b".to_string()),
        Net::new("out"),
    );
    assert_eq!(c1.kind, synthesis::CellKind::And);

    let c2 = Cell::not("not0",
        Expression::Signal("a".to_string()),
        Net::new("out"),
    );
    assert_eq!(c2.kind, synthesis::CellKind::Not);

    let c3 = Cell::dff("dff0",
        Expression::Signal("clk".to_string()),
        Expression::Signal("d".to_string()),
        Net::new("q"),
    );
    assert_eq!(c3.kind, synthesis::CellKind::Dff);
    assert_eq!(c3.inputs.len(), 2);
    assert_eq!(c3.outputs.len(), 1);
}