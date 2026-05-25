use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Input,
    Output,
    Inout,
}

#[derive(Debug, Clone)]
pub enum BitWidth {
    Bit,
    Bits(usize),
}

impl BitWidth {
    pub fn width(&self) -> usize {
        match self {
            BitWidth::Bit => 1,
            BitWidth::Bits(n) => *n,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Bits(u64, usize),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Literal(Literal),
    Signal(String),
    Concat(Vec<Expression>),
    Repeat(Box<Expression>, usize),
    Slice(String, Option<usize>, Option<usize>),
    BinaryOp(BinaryOp, Box<Expression>, Box<Expression>),
    UnaryOp(UnaryOp, Box<Expression>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOp {
    Add, Sub, Mul, Div, Mod,
    And, Or, Xor, Shl, Shr, Eq, Ne, Lt, Le, Gt, Ge,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Not, Neg,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Expression, Expression),
    If(Expression, Vec<Statement>, Option<Vec<Statement>>),
    Case(Expression, Vec<(Vec<Expression>, Vec<Statement>)>),
    For(String, Expression, Expression, Vec<Statement>),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub struct Port {
    pub name: String,
    pub direction: Direction,
    pub width: BitWidth,
}

#[derive(Debug, Clone)]
pub struct SignalDecl {
    pub name: String,
    pub width: BitWidth,
    pub init: Option<Expression>,
}

#[derive(Debug, Clone)]
pub struct AlwaysBlock {
    pub sensitivity: Vec<SignalKind>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum SignalKind {
    Signal(String),
    PosEdge(String),
    NegEdge(String),
    Star,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub ports: Vec<Port>,
    pub signals: Vec<SignalDecl>,
    pub always_blocks: Vec<AlwaysBlock>,
    pub assigns: Vec<(Expression, Expression)>,
    pub instances: Vec<Instance>,
}

#[derive(Debug, Clone)]
pub struct Instance {
    pub module_name: String,
    pub instance_name: String,
    pub connections: HashMap<String, Expression>,
}

#[derive(Debug, Clone)]
pub struct Elaborated {
    pub name: String,
    pub ports: Vec<Port>,
    pub cells: Vec<Cell>,
    pub nets: Vec<Net>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Net {
    pub name: String,
    pub width: usize,
}

impl Net {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), width: 1 }
    }
    pub fn with_width(name: &str, width: usize) -> Self {
        Self { name: name.to_string(), width }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellKind {
    Not,
    And,
    Or,
    Xor,
    Nand,
    Nor,
    Xnor,
    Mux,
    Dff,
    Dffr,
    Dffs,
    Dffsr,
    Buf,
    Concat,
    Slice,
    Const,
}

#[derive(Debug, Clone)]
pub struct Cell {
    pub name: String,
    pub kind: CellKind,
    pub inputs: Vec<Expression>,
    pub outputs: Vec<Net>,
}

impl Cell {
    pub fn not(name: &str, input: Expression, output: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Not, inputs: vec![input], outputs: vec![output] }
    }
    pub fn and(name: &str, a: Expression, b: Expression, out: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::And, inputs: vec![a, b], outputs: vec![out] }
    }
    pub fn or(name: &str, a: Expression, b: Expression, out: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Or, inputs: vec![a, b], outputs: vec![out] }
    }
    pub fn xor(name: &str, a: Expression, b: Expression, out: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Xor, inputs: vec![a, b], outputs: vec![out] }
    }
    pub fn mux(name: &str, sel: Expression, a: Expression, b: Expression, out: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Mux, inputs: vec![sel, a, b], outputs: vec![out] }
    }
    pub fn dff(name: &str, clk: Expression, d: Expression, q: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Dff, inputs: vec![clk, d], outputs: vec![q] }
    }
    pub fn buf(name: &str, input: Expression, output: Net) -> Self {
        Self { name: name.to_string(), kind: CellKind::Buf, inputs: vec![input], outputs: vec![output] }
    }
    pub fn const_(name: &str, val: u64, width: usize, out: Net) -> Self {
        Self {
            name: name.to_string(),
            kind: CellKind::Const,
            inputs: vec![Expression::Literal(Literal::Bits(val, width))],
            outputs: vec![out],
        }
    }
}

pub struct Elaborator {
    cell_counter: usize,
    net_counter: usize,
}

impl Elaborator {
    pub fn new() -> Self {
        Self { cell_counter: 0, net_counter: 0 }
    }

    fn new_cell_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.cell_counter);
        self.cell_counter += 1;
        name
    }

    fn new_net_name(&mut self, prefix: &str) -> String {
        let name = format!("n_{}_{}", prefix, self.net_counter);
        self.net_counter += 1;
        name
    }

    pub fn elaborate(&mut self, module: &Module) -> Elaborated {
        let mut cells = Vec::new();
        let mut nets = HashMap::new();

        for sig in &module.signals {
            let w = sig.width.width();
            nets.insert(sig.name.clone(), Net::with_width(&sig.name, w));
        }

        for port in &module.ports {
            let w = port.width.width();
            if !nets.contains_key(&port.name) {
                nets.insert(port.name.clone(), Net::with_width(&port.name, w));
            }
        }

        for &(ref lhs, ref rhs) in &module.assigns {
            let out_net = self.resolve_to_net(lhs, &mut cells, &nets, &mut HashMap::new());
            let (expr_cells, expr_out) = self.compile_expression(rhs, &nets, &mut HashMap::new());
            cells.extend(expr_cells);
            cells.push(Cell::buf(&self.new_cell_name("buf"), Expression::Signal(expr_out.name.clone()), out_net));
        }

        for always in &module.always_blocks {
            let (always_cells, _) = self.compile_always(always, &nets);
            cells.extend(always_cells);
        }

        let net_list: Vec<Net> = nets.into_values().collect();
        Elaborated { name: module.name.clone(), ports: module.ports.clone(), cells, nets: net_list }
    }

    fn resolve_to_net(&mut self, expr: &Expression, cells: &mut Vec<Cell>, nets: &HashMap<String, Net>, memo: &mut HashMap<String, Net>) -> Net {
        match expr {
            Expression::Signal(name) => {
                nets.get(name).cloned().unwrap_or_else(|| Net::new(name))
            }
            Expression::Slice(name, _hi, _lo) => {
                nets.get(name).cloned().unwrap_or_else(|| Net::new(name))
            }
            _ => {
                let (expr_cells, expr_out) = self.compile_expression(expr, nets, memo);
                cells.extend(expr_cells);
                expr_out
            }
        }
    }

    fn compile_expression(&mut self, expr: &Expression, nets: &HashMap<String, Net>, memo: &mut HashMap<String, Net>) -> (Vec<Cell>, Net) {
        let mut cells = Vec::new();
        let out = match expr {
            Expression::Literal(Literal::Bool(true)) => {
                let n = Net::new(&self.new_net_name("const"));
                cells.push(Cell::const_(&self.new_cell_name("const"), 1, 1, n.clone()));
                n
            }
            Expression::Literal(Literal::Bool(false)) => {
                let n = Net::new(&self.new_net_name("const"));
                cells.push(Cell::const_(&self.new_cell_name("const"), 0, 1, n.clone()));
                n
            }
            Expression::Literal(Literal::Bits(val, width)) => {
                let n = Net::with_width(&self.new_net_name("const"), *width);
                cells.push(Cell::const_(&self.new_cell_name("const"), *val, *width, n.clone()));
                n
            }
            Expression::Literal(Literal::String(_)) => {
                Net::new(&self.new_net_name("str"))
            }
            Expression::Signal(name) => {
                nets.get(name).cloned().unwrap_or_else(|| Net::new(name))
            }
            Expression::Slice(name, _hi, _lo) => {
                nets.get(name).cloned().unwrap_or_else(|| Net::new(name))
            }
            Expression::BinaryOp(op, a, b) => {
                let (ca, na) = self.compile_expression(a, nets, memo);
                let (cb, nb) = self.compile_expression(b, nets, memo);
                cells.extend(ca);
                cells.extend(cb);
                let out_net = Net::new(&self.new_net_name("binop"));
                let cell = match op {
                    BinaryOp::And => Cell::and(&self.new_cell_name("and"), Expression::Signal(na.name.clone()), Expression::Signal(nb.name.clone()), out_net.clone()),
                    BinaryOp::Or  => Cell::or(&self.new_cell_name("or"), Expression::Signal(na.name.clone()), Expression::Signal(nb.name.clone()), out_net.clone()),
                    BinaryOp::Xor => Cell::xor(&self.new_cell_name("xor"), Expression::Signal(na.name.clone()), Expression::Signal(nb.name.clone()), out_net.clone()),
                    _ => Cell::buf(&self.new_cell_name("binop"),
                                   Expression::BinaryOp(*op, Box::new(Expression::Signal(na.name.clone())), Box::new(Expression::Signal(nb.name.clone()))),
                                   out_net.clone()),
                };
                cells.push(cell);
                out_net
            }
            Expression::UnaryOp(op, a) => {
                let (ca, na) = self.compile_expression(a, nets, memo);
                cells.extend(ca);
                let out_net = Net::new(&self.new_net_name("unop"));
                if *op == UnaryOp::Not {
                    cells.push(Cell::not(&self.new_cell_name("not"), Expression::Signal(na.name.clone()), out_net.clone()));
                } else {
                    cells.push(Cell::buf(&self.new_cell_name("unop"), expr.clone(), out_net.clone()));
                }
                out_net
            }
            Expression::Concat(exprs) => {
                let cur = Net::new(&self.new_net_name("concat"));
                for e in exprs {
                    let (mut cc, _ne) = self.compile_expression(e, nets, memo);
                    cells.append(&mut cc);
                }
                cur
            }
            Expression::Repeat(inner, _count) => {
                let (mut cc, _ne) = self.compile_expression(inner, nets, memo);
                cells.extend(cc);
                nets.values().next().cloned().unwrap_or_else(|| Net::new("tmp"))
            }
        };
        (cells, out)
    }

    fn compile_always(&mut self, always: &AlwaysBlock, nets: &HashMap<String, Net>) -> (Vec<Cell>, Vec<Net>) {
        let mut cells = Vec::new();
        let mut reg_outputs = Vec::new();

        if always.sensitivity.len() == 1 {
            if let SignalKind::PosEdge(sig) = &always.sensitivity[0] {
                for stmt in &always.body {
                    let (stmt_cells, out_nets) = self.compile_stmt(stmt, nets);
                    for n in out_nets {
                        cells.push(Cell::dff(&self.new_cell_name("dff"),
                                             Expression::Signal(sig.clone()),
                                             Expression::Signal(n.name.clone()),
                                             n));
                    }
                    cells.extend(stmt_cells);
                }
            }
        }

        for stmt in &always.body {
            let (stmt_cells, out_nets) = self.compile_stmt(stmt, nets);
            cells.extend(stmt_cells);
            reg_outputs.extend(out_nets);
        }

        (cells, reg_outputs)
    }

    fn compile_stmt(&mut self, stmt: &Statement, nets: &HashMap<String, Net>) -> (Vec<Cell>, Vec<Net>) {
        let mut cells = Vec::new();
        let mut outputs = Vec::new();
        match stmt {
            Statement::Assign(lhs, rhs) => {
                let lhs_net = match lhs {
                    Expression::Signal(n) => nets.get(n).cloned().unwrap_or_else(|| Net::new(n)),
                    Expression::Slice(n, _, _) => nets.get(n).cloned().unwrap_or_else(|| Net::new(n)),
                    _ => Net::new(&self.new_net_name("assign")),
                };
                let (rc, rn) = self.compile_expression(rhs, nets, &mut HashMap::new());
                cells.extend(rc);
                cells.push(Cell::buf(&self.new_cell_name("assign"), Expression::Signal(rn.name.clone()), lhs_net.clone()));
                outputs.push(lhs_net);
            }
            Statement::If(cond, then_b, else_b) => {
                let (rc, _rn) = self.compile_expression(cond, nets, &mut HashMap::new());
                cells.extend(rc);
                for stmt in then_b {
                    let (tc, tn) = self.compile_stmt(stmt, nets);
                    cells.extend(tc);
                    outputs.extend(tn);
                }
                if let Some(else_b) = else_b {
                    for stmt in else_b {
                        let (ec, en) = self.compile_stmt(stmt, nets);
                        cells.extend(ec);
                        outputs.extend(en);
                    }
                }
            }
            Statement::Block(stmts) => {
                for s in stmts {
                    let (sc, sn) = self.compile_stmt(s, nets);
                    cells.extend(sc);
                    outputs.extend(sn);
                }
            }
            _ => {}
        }
        (cells, outputs)
    }
}

pub struct Optimizer;

impl Optimizer {
    pub fn optimize(elab: &Elaborated) -> Elaborated {
        let mut cells = elab.cells.clone();

        cells = Self::fold_constants(cells);
        cells = Self::remove_buffers(cells);
        cells = Self::merge_cells(cells);

        Elaborated { name: elab.name.clone(), ports: elab.ports.clone(), cells, nets: elab.nets.clone() }
    }

    fn fold_constants(cells: Vec<Cell>) -> Vec<Cell> {
        cells
    }

    fn remove_buffers(cells: Vec<Cell>) -> Vec<Cell> {
        cells.into_iter().filter(|c| !matches!(c.kind, CellKind::Buf)).collect()
    }

    fn merge_cells(cells: Vec<Cell>) -> Vec<Cell> {
        cells
    }
}

pub struct TechMapper;

impl TechMapper {
    pub fn map(elab: &Elaborated) -> Elaborated {
        let mut cells = Vec::new();
        for cell in &elab.cells {
            let mapped = match &cell.kind {
                CellKind::Not | CellKind::And | CellKind::Or | CellKind::Xor |
                CellKind::Nand | CellKind::Nor | CellKind::Xnor | CellKind::Mux |
                CellKind::Dff | CellKind::Buf | CellKind::Const => cell.clone(),
                CellKind::Dffr => {
                    Cell { name: cell.name.clone(), kind: CellKind::Dff, inputs: cell.inputs.clone(), outputs: cell.outputs.clone() }
                }
                CellKind::Dffs => {
                    Cell { name: cell.name.clone(), kind: CellKind::Dff, inputs: cell.inputs.clone(), outputs: cell.outputs.clone() }
                }
                CellKind::Dffsr => {
                    Cell { name: cell.name.clone(), kind: CellKind::Dff, inputs: cell.inputs.clone(), outputs: cell.outputs.clone() }
                }
                CellKind::Concat | CellKind::Slice => cell.clone(),
            };
            cells.push(mapped);
        }
        Elaborated { name: elab.name.clone(), ports: elab.ports.clone(), cells, nets: elab.nets.clone() }
    }
}

impl Default for Elaborator {
    fn default() -> Self { Self::new() }
}

pub fn synthesize(module: Module) -> Elaborated {
    let mut elaborator = Elaborator::new();
    let elab = elaborator.elaborate(&module);
    let opt = Optimizer::optimize(&elab);
    TechMapper::map(&opt)
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), ports: Vec::new(), signals: Vec::new(), always_blocks: Vec::new(), assigns: Vec::new(), instances: Vec::new() }
    }

    pub fn port(mut self, name: &str, direction: Direction, width: BitWidth) -> Self {
        self.ports.push(Port { name: name.to_string(), direction, width });
        self
    }

    pub fn signal(mut self, name: &str, width: BitWidth) -> Self {
        self.signals.push(SignalDecl { name: name.to_string(), width, init: None });
        self
    }

    pub fn assign(mut self, lhs: Expression, rhs: Expression) -> Self {
        self.assigns.push((lhs, rhs));
        self
    }

    pub fn always_posedge(mut self, sig: &str, body: Vec<Statement>) -> Self {
        self.always_blocks.push(AlwaysBlock { sensitivity: vec![SignalKind::PosEdge(sig.to_string())], body });
        self
    }
}

impl Elaborated {
    pub fn print_verilog(&self) -> String {
        let mut s = format!("module {} (\n", self.name);
        for (i, port) in self.ports.iter().enumerate() {
            let dir = match port.direction {
                Direction::Input => "input",
                Direction::Output => "output",
                Direction::Inout => "inout",
            };
            let width = if port.width.width() > 1 {
                format!("[{}:0] ", port.width.width() - 1)
            } else { String::new() };
            s.push_str(&format!("  {}{}{}", dir, if dir.len() < 6 { " ".repeat(6 - dir.len()) } else { String::new() }, width));
            s.push_str(&port.name);
            if i < self.ports.len() - 1 { s.push(','); }
            s.push('\n');
        }
        s.push_str(");\n\n");

        for cell in &self.cells {
            s.push_str(&format!("  // {} {}\n", format!("{:?}", cell.kind).to_lowercase(), cell.name));
        }

        s.push_str("endmodule\n");
        s
    }

    pub fn print_dot(&self) -> String {
        let mut s = String::from("digraph circuit {\n  rankdir=LR;\n  node [shape=box];\n");
        for cell in &self.cells {
            let label = format!("{:?}", cell.kind);
            s.push_str(&format!("  \"{}\" [label=\"{}\"];\n", cell.name, label));
            for out in &cell.outputs {
                s.push_str(&format!("  \"{}\" -> \"{}_out\";\n", cell.name, out.name));
            }
        }
        s.push_str("}\n");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_adder() {
        let mod_adder = Module::new("adder")
            .port("a", Direction::Input, BitWidth::Bits(8))
            .port("b", Direction::Input, BitWidth::Bits(8))
            .port("sum", Direction::Output, BitWidth::Bits(8))
            .signal("sum_internal", BitWidth::Bits(8))
            .assign(Expression::Signal("sum".to_string()),
                    Expression::BinaryOp(BinaryOp::Add,
                                         Box::new(Expression::Signal("a".to_string())),
                                         Box::new(Expression::Signal("b".to_string()))));

        let result = synthesize(mod_adder);
        assert_eq!(result.name, "adder");
        assert_eq!(result.ports.len(), 3);
    }

    #[test]
    fn test_registered_adder() {
        let mod_reg = Module::new("reg_adder")
            .port("clk", Direction::Input, BitWidth::Bit)
            .port("a", Direction::Input, BitWidth::Bits(8))
            .port("b", Direction::Input, BitWidth::Bits(8))
            .port("q", Direction::Output, BitWidth::Bits(8))
            .signal("sum_wire", BitWidth::Bits(8))
            .assign(Expression::Signal("sum_wire".to_string()),
                    Expression::BinaryOp(BinaryOp::Add,
                                         Box::new(Expression::Signal("a".to_string())),
                                         Box::new(Expression::Signal("b".to_string()))))
            .always_posedge("clk", vec![
                Statement::Assign(
                    Expression::Signal("q".to_string()),
                    Expression::Signal("sum_wire".to_string()),
                )
            ]);

        let result = synthesize(mod_reg);
        assert_eq!(result.name, "reg_adder");
    }

    #[test]
    fn test_mux() {
        let mod_mux = Module::new("mux2to1")
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

        let result = synthesize(mod_mux);
        assert_eq!(result.name, "mux2to1");
    }
}