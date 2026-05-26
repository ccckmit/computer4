use crate::ast::*;
use crate::parse::eval_const;
use std::collections::HashSet;

pub fn gen_module(m: &Module) -> String {
    let mut out = String::new();
    let pub_name = to_pascal(&m.name);

    // collect all decls
    let mut wires: Vec<&VarDecl> = Vec::new();
    let mut regs: Vec<&VarDecl> = Vec::new();
    let mut gate_insts: Vec<&GateInst> = Vec::new();
    let mut sub_insts: Vec<&ModuleInst> = Vec::new();
    let mut assigns: Vec<(&Expr, &Expr)> = Vec::new();
    let mut always_blocks: Vec<&AlwaysBlock> = Vec::new();
    let mut initial_blocks: Vec<&Vec<Stmt>> = Vec::new();
    let mut integers: Vec<&String> = Vec::new();

    for item in &m.items {
        match item {
            ModuleItem::Wire(v) => wires.push(v),
            ModuleItem::Reg(v) => regs.push(v),
            ModuleItem::Integer(n) => integers.push(n),
            ModuleItem::GateInst(g) => gate_insts.push(g),
            ModuleItem::ModuleInst(s) => sub_insts.push(s),
            ModuleItem::Assign { lhs, rhs } => assigns.push((lhs, rhs)),
            ModuleItem::Always(a) => always_blocks.push(a),
            ModuleItem::Initial(s) => initial_blocks.push(s),
        }
    }

    // determine used external module names for sub-instantiations
    let _sub_mod_names: HashSet<&str> = sub_insts.iter().map(|s| s.module_name.as_str()).collect();

    // determine width of each signal
    let decls = build_decl_map(&m.ports, &wires, &regs, &integers);
    let sizes = build_size_map(&decls);

    // ----- struct fields -----
    out.push_str(&format!("#[derive(Debug, Clone)]\n"));
    out.push_str(&format!("pub struct {} {{\n", pub_name));

    // public port fields
    for p in &m.ports {
        let field_name = to_snake(&p.name);
        let field_type = port_type(&p, &sizes);
        let vis = match p.direction {
            PortDir::Input | PortDir::Inout => "pub ",
            PortDir::Output => "pub ",
        };
        out.push_str(&format!("    {}{}: {},\n", vis, field_name, field_type));
    }

    // internal wire/reg fields (only those that aren't already ports)
    for v in &wires {
        if !is_port(&m.ports, &v.name) {
            let ft = var_type(&v, &sizes);
            out.push_str(&format!("    {}: {},\n", to_snake(&v.name), ft));
        }
    }
    for v in &regs {
        if !is_port(&m.ports, &v.name) {
            let ft = var_type(&v, &sizes);
            out.push_str(&format!("    {}: {},\n", to_snake(&v.name), ft));
        }
    }

    // gate sub-component fields
    for g in &gate_insts {
        let rust_gate = verilog_gate_to_rust(&g.gate_type);
        let fname = if g.instance_name.is_empty() { format!("gate_{}", to_snake(&g.gate_type)) } else { to_snake(&g.instance_name) };
        out.push_str(&format!("    {}: {},\n", fname, rust_gate));
    }

    // sub-module fields
    for s in &sub_insts {
        let sn = to_pascal(&s.module_name);
        let fname = to_snake(&s.instance_name);
        out.push_str(&format!("    {}: {},\n", fname, sn));
    }

    out.push_str("}\n\n");

    // ----- impl new() -----
    out.push_str(&format!("impl {} {{\n", pub_name));
    out.push_str(&format!("    pub fn new(\n"));

    // constructor params: just the port signals
    let params: Vec<String> = m.ports.iter().map(|p| {
        let pt = port_type(&p, &sizes);
        format!("        {}: {}", to_snake(&p.name), pt)
    }).collect();
    if params.is_empty() {
        out.push_str("    ) -> Self {\n");
    } else {
        out.push_str(&params.join(",\n"));
        out.push_str(",\n    ) -> Self {\n");
    }

    // create internal wires
    for v in &wires {
        if !is_port(&m.ports, &v.name) {
            let fname = to_snake(&v.name);
            let w = width_val(&v.width);
            if w > 1 {
                out.push_str(&format!("        let {} = bus(\"{}\", {});\n", fname, fname, w));
            } else {
                out.push_str(&format!("        let {} = wire(\"{}\");\n", fname, fname));
            }
        }
    }
    for v in &regs {
        if !is_port(&m.ports, &v.name) {
            let fname = to_snake(&v.name);
            let w = width_val(&v.width);
            if w > 1 {
                out.push_str(&format!("        let {} = bus(\"{}\", {});\n", fname, fname, w));
            } else {
                out.push_str(&format!("        let {} = wire(\"{}\");\n", fname, fname));
            }
        }
    }

    out.push_str(&format!("        {} {{\n", pub_name));

    // init port fields (clone so originals can be passed to sub-components)
    for p in &m.ports {
        let n = to_snake(&p.name);
        out.push_str(&format!("            {}: {}.clone(),\n", n, n));
    }
    for v in &wires {
        if !is_port(&m.ports, &v.name) {
            let n = to_snake(&v.name);
            out.push_str(&format!("            {}: {}.clone(),\n", n, n));
        }
    }
    for v in &regs {
        if !is_port(&m.ports, &v.name) {
            let n = to_snake(&v.name);
            out.push_str(&format!("            {}: {}.clone(),\n", n, n));
        }
    }

    // init gate sub-components
    for g in &gate_insts {
        let rust_gate = verilog_gate_to_rust(&g.gate_type);
        let fname = if g.instance_name.is_empty() { format!("gate_{}", to_snake(&g.gate_type)) } else { to_snake(&g.instance_name) };
        let args: Vec<String> = g.inputs.iter().chain(g.outputs.iter()).map(|e| {
            let v = expr_to_var(e, &sizes);
            format!("{}.clone()", v)
        }).collect();
        out.push_str(&format!("            {}: {}::new({}),\n", fname, rust_gate, args.join(", ")));
    }

    // init sub-modules
    for s in &sub_insts {
        let sn = to_pascal(&s.module_name);
        let fname = to_snake(&s.instance_name);
        let args: Vec<String> = s.connections.iter().map(|c| {
            match c {
                Conn::ByOrder(e) | Conn::ByName { wire: e, .. } => {
                    let v = expr_to_var(e, &sizes);
                    let w = expr_width(e, &sizes);
                    if w > 1 {
                        format!("{}.clone()", v)
                    } else {
                        format!("{}.clone()", v)
                    }
                }
            }
        }).collect();
        out.push_str(&format!("            {}: {}::new({}),\n", fname, sn, args.join(", ")));
    }

    out.push_str("        }\n");
    out.push_str("    }\n\n");

    // ----- eval() method -----
    out.push_str("    pub fn eval(&mut self) {\n");

    // eval sub-components first
    for g in &gate_insts {
        let fname = if g.instance_name.is_empty() { format!("gate_{}", to_snake(&g.gate_type)) } else { to_snake(&g.instance_name) };
        out.push_str(&format!("        self.{}.eval();\n", fname));
    }
    for s in &sub_insts {
        out.push_str(&format!("        self.{}.eval();\n", to_snake(&s.instance_name)));
    }

    // eval assign statements
    for (lhs, rhs) in &assigns {
        let lhs_expr = gen_expr_to_set(lhs, rhs, &sizes, &decls, 0, "");
        if !lhs_expr.is_empty() {
            out.push_str(&lhs_expr);
        }
    }

    // eval always blocks
    for ab in &always_blocks {
        for s in &ab.stmts {
            gen_stmt(&mut out, s, &sizes, &decls, 4);
        }
    }

    out.push_str("    }\n");

    // ----- run() method from initial blocks -----
    if !initial_blocks.is_empty() {
        out.push_str("    pub fn run(&mut self) {\n");
        for block in &initial_blocks {
            for s in *block {
                gen_stmt(&mut out, s, &sizes, &decls, 8);
            }
        }
        out.push_str("    }\n");
    }

    out.push_str("}\n\n");

    out
}

fn verilog_fmt_to_rust(fmt: &str) -> String {
    let mut out = String::new();
    let mut chars = fmt.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('%') => out.push('%'),
                Some('d' | 'D' | 'h' | 'H' | 'b' | 'B' | 'o' | 'O') => out.push_str("{}"),
                Some('s') => out.push_str("{}"),
                _ => out.push_str("{}"),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn gen_stmt(out: &mut String, s: &Stmt, sizes: &SizeMap, decls: &DeclMap, indent: usize) {
    let ind = " ".repeat(indent);
    match s {
        Stmt::BlockingAssign { lhs, rhs } | Stmt::NonBlockingAssign { lhs, rhs } => {
            let lhs_code = gen_expr_to_set(lhs, rhs, sizes, decls, indent, "");
            if !lhs_code.is_empty() {
                out.push_str(&lhs_code);
            }
        }
        Stmt::If { cond, then, else_ } => {
            let cond_str = gen_expr_str(cond, sizes, decls);
            out.push_str(&format!("{}if {} != Level::L {{\n", ind, cond_str));
            for ss in then {
                gen_stmt(out, ss, sizes, decls, indent + 4);
            }
            if !else_.is_empty() {
                out.push_str(&format!("{}}} else {{\n", ind));
                for ss in else_ {
                    gen_stmt(out, ss, sizes, decls, indent + 4);
                }
                out.push_str(&format!("{}}}\n", ind));
            } else {
                out.push_str(&format!("{}}}\n", ind));
            }
        }
        Stmt::Case { expr: _expr, items } => {
            let expr_val = gen_expr_val(_expr, sizes, decls);
            out.push_str(&format!("{}let __case_val = {};\n", ind, expr_val));
            for item in items {
                if item.exprs.is_empty() {
                    // default
                    out.push_str(&format!("{}// default case\n", ind));
                    for ss in &item.stmts {
                        gen_stmt(out, ss, sizes, decls, indent);
                    }
                } else {
                    for e in &item.exprs {
                        let ev = gen_expr_val(e, sizes, decls);
                        out.push_str(&format!("{}if __case_val == {} {{\n", ind, ev));
                        for ss in &item.stmts {
                            gen_stmt(out, ss, sizes, decls, indent + 4);
                        }
                        out.push_str(&format!("{}}}\n", ind));
                    }
                }
            }
        }
        Stmt::For { init, cond, inc, stmts } => {
            gen_stmt(out, init, sizes, decls, indent);
            let cond_str = gen_expr_str(cond, sizes, decls);
            out.push_str(&format!("{}while {} != Level::L {{\n", ind, cond_str));
            for ss in stmts {
                gen_stmt(out, ss, sizes, decls, indent + 4);
            }
            gen_stmt(out, inc, sizes, decls, indent + 4);
            out.push_str(&format!("{}}}\n", ind));
        }
        Stmt::Forever { stmts } => {
            out.push_str(&format!("{}loop {{\n", ind));
            for ss in stmts {
                gen_stmt(out, ss, sizes, decls, indent + 4);
            }
            out.push_str(&format!("{}}}\n", ind));
        }
        Stmt::SysCall { name, args } => {
            if name == "$display" || name == "$monitor" {
                if args.is_empty() {
                    out.push_str(&format!("{}println!();\n", ind));
                } else {
                    let fmt_arg = &args[0];
                    if let Expr::Ident(s) = fmt_arg {
                        if let Some(fmt_str) = s.strip_prefix("__str:") {
                            let rust_fmt = verilog_fmt_to_rust(fmt_str);
                            let rest: Vec<String> = args[1..].iter().map(|e| {
                                let v = gen_expr_val(e, sizes, decls);
                                v
                            }).collect();
                            if rest.is_empty() {
                                out.push_str(&format!("{}println!(\"{}\");\n", ind, rust_fmt));
                            } else {
                                out.push_str(&format!("{}println!(\"{}\", {});\n", ind, rust_fmt, rest.join(", ")));
                            }
                        } else {
                            out.push_str(&format!("{}println!();\n", ind));
                        }
                    } else {
                        out.push_str(&format!("{}println!();\n", ind));
                    }
                }
            } else {
                out.push_str(&format!("{}// unknown syscall: {}\n", ind, name));
            }
        }
        Stmt::SysFinish => {
            out.push_str(&format!("{}return;\n", ind));
        }
        Stmt::DelayStmt { stmt, .. } => {
            // In cycle-based simulation, a delay means "evaluate the design"
            out.push_str(&format!("{}self.eval();\n", ind));
            if let Some(inner) = stmt {
                gen_stmt(out, inner, sizes, decls, indent);
            }
        }
    }
}

fn gen_expr_to_set(lhs: &Expr, rhs: &Expr, sizes: &SizeMap, decls: &DeclMap, indent: usize, _prefix: &str) -> String {
    use crate::parse::eval_const;
    let ind = " ".repeat(indent);
    match lhs {
        Expr::Ident(name) => {
            let lname = to_snake(name);
            let w = sizes.get(&lname).copied().unwrap_or(1);
            if w > 1 {
                let rhs_code = gen_expr_bus_val(rhs, sizes, decls, w);
                    format!("{}u16_to_bus(&self.{}, ({} & {}) as u16);\n", ind, lname, rhs_code, mask(w))
            } else {
                let rhs_code = gen_expr_str(rhs, sizes, decls);
                format!("{}if get(&self.{}) != {} {{ set(&self.{}, {}); }}\n", ind, lname, rhs_code, lname, rhs_code)
            }
        }
        Expr::Select { expr, msb, lsb } => {
            if let Expr::Ident(name) = expr.as_ref() {
                let lname = to_snake(name);
                let msb_v = eval_const(msb);
                let lsb_v = eval_const(lsb);
                let w = (msb_v - lsb_v + 1) as usize;
                // generate code to set a slice
                let rhs_code = gen_expr_bus_val(rhs, sizes, decls, w as u64);
                format!(
                    "{}let __val = {} as u16;\n{}for __i in {}..={} {{\n{}    let __bit = (__val >> (__i - {})) & 1;\n{}    set(&self.{}[__i], if __bit == 1 {{ Level::H }} else {{ Level::L }});\n{}}}\n",
                    ind, rhs_code, ind, lsb_v, msb_v, ind, lsb_v, ind, lname, ind
                )
            } else {
                String::new()
            }
        }
        Expr::BitSelect { expr, bit } => {
            if let Expr::Ident(name) = expr.as_ref() {
                let lname = to_snake(name);
                let bit_v = eval_const(bit);
                let rhs_code = gen_expr_str(rhs, sizes, decls);
                format!("{}if get(&self.{}[{}]) != {} {{ set(&self.{}[{}], {}); }}\n", ind, lname, bit_v, rhs_code, lname, bit_v, rhs_code)
            } else {
                String::new()
            }
        }
        Expr::Concat(items) => {
            // assign to concatenation of signals
            // For now, generate assignments for each element
            let mut code = String::new();
            // compute total width
            let mut total_w = 0u64;
            let mut widths = Vec::new();
            for item in items {
                let w = expr_width(item, sizes);
                widths.push(w);
                total_w += w;
            }
            let rhs_code = gen_expr_bus_val(rhs, sizes, decls, total_w);
            code.push_str(&format!("{}let __concat_val = {};\n", ind, rhs_code));
            let mut offset = 0;
            for (i, item) in items.iter().enumerate() {
                let w = widths[i];
                if w > 1 {
                    if let Expr::Ident(name) = item {
                        let lname = to_snake(name);
                        code.push_str(&format!(
                            "{}for __j in 0..{} {{\n{}    let __b = (__concat_val >> ({} + __j)) & 1;\n{}    set(&self.{}[__j], if __b == 1 {{ Level::H }} else {{ Level::L }});\n{}}}\n",
                            ind, w, ind, offset, ind, lname, ind
                        ));
                    }
                } else {
                    if let Expr::Ident(name) = item {
                        let lname = to_snake(name);
                        code.push_str(&format!(
                            "{}set(&self.{}, if (__concat_val >> {}) & 1 == 1 {{ Level::H }} else {{ Level::L }});\n",
                            ind, lname, offset
                        ));
                    }
                }
                offset += w;
            }
            code
        }
        _ => String::new(),
    }
}

fn gen_expr_str(expr: &Expr, sizes: &SizeMap, decls: &DeclMap) -> String {
    match expr {
        Expr::Number(n) => {
            if n.value == 0 { "Level::L".to_string() }
            else { "Level::H".to_string() }
        }
        Expr::Ident(name) => {
            let n = to_snake(name);
            let w = sizes.get(&n).copied().unwrap_or(1);
            if w > 1 {
                format!("bus_to_u16(&self.{})", n)
            } else {
                format!("get(&self.{})", n)
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let l = gen_expr_str(lhs, sizes, decls);
            let r = gen_expr_str(rhs, sizes, decls);
            let lw = expr_width(lhs, sizes);
            let rw = expr_width(rhs, sizes);
            let w = std::cmp::max(lw, rw);
            match op {
                BinaryOp::Add => {
                    if w > 1 {
                        format!("({} + {}) & {}", l, r, mask(w))
                    } else {
                        format!("{}.xor({})", l, r) // simple bitwise for single bit
                    }
                }
                BinaryOp::Sub => {
                    if w > 1 {
                        format!("({}.wrapping_sub({})) & {}", l, r, mask(w))
                    } else {
                        format!("{}.xor({})", l, r)
                    }
                }
                BinaryOp::BitAnd => {
                    if w > 1 { format!("{} & {}", l, r) } else { format!("{}.and({})", l, r) }
                }
                BinaryOp::BitOr => {
                    if w > 1 { format!("{} | {}", l, r) } else { format!("{}.or({})", l, r) }
                }
                BinaryOp::BitXor => {
                    if w > 1 { format!("{} ^ {}", l, r) } else { format!("{}.xor({})", l, r) }
                }
                BinaryOp::Lt => format!("if {} < {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Leq => format!("if {} <= {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Gt => format!("if {} > {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Geq => format!("if {} >= {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Eq => format!("if {} == {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Neq => format!("if {} != {} {{ 1u16 }} else {{ 0u16 }}", l, r),
                BinaryOp::Shl => format!("{} << {}", l, r),
                BinaryOp::Shr => format!("{} >> {}", l, r),
                BinaryOp::Sshl => format!("{} << {}", l, r),
                BinaryOp::Sshr => format!("{} >> {}", l, r),
                BinaryOp::Mul => {
                    if w > 1 { format!("({} * {}) & {}", l, r, mask(w)) }
                    else { format!("if {} == Level::H && {} == Level::H {{ Level::H }} else {{ Level::L }}", l, r) }
                }
                BinaryOp::Div => format!("{} / {}", l, r),
                BinaryOp::Mod => format!("{} % {}", l, r),
                BinaryOp::LogicalAnd => format!("({} != 0 && {} != 0) as u16", l, r),
                BinaryOp::LogicalOr => format!("({} != 0 || {} != 0) as u16", l, r),
                BinaryOp::BitXnor => {
                    if w > 1 { format!("!({} ^ {})", l, r) }
                    else { format!("{}.xnor({})", l, r) }
                }
            }
        }
        Expr::Unary { op, expr } => {
            let e = gen_expr_str(expr, sizes, decls);
            let w = expr_width(expr, sizes);
            match op {
                UnaryOp::Minus => {
                    if w > 1 { format!("(!{} + 1) & {}", e, mask(w)) }
                    else { format!("{}", e) }
                }
                UnaryOp::BitNot => {
                    if w > 1 { format!("!{}", e) }
                    else { format!("{}.not()", e) }
                }
                UnaryOp::LogicalNot => {
                    if w > 1 { format!("if {} == 0 {{ 1u16 }} else {{ 0u16 }}", e) }
                    else { format!("{}.not()", e) }
                }
                UnaryOp::ReduceAnd => format!("if {} == {} {{ 1u16 }} else {{ 0u16 }}", e, mask(w)),
                UnaryOp::ReduceOr => format!("if {} != 0 {{ 1u16 }} else {{ 0u16 }}", e),
                UnaryOp::ReduceXor => {
                    // compute parity
                    format!("({}).count_ones() & 1", e)
                }
                _ => e,
            }
        }
        Expr::Concat(items) => {
            let mut total_w = 0u64;
            let parts: Vec<String> = items.iter().rev().map(|item| {
                let w = expr_width(item, sizes);
                let s = gen_expr_str(item, sizes, decls);
                let part = if w > 1 {
                    format!("({}) as u64", s)
                } else {
                    format!("if {} == Level::H {{ 1u64 }} else {{ 0u64 }}", s)
                };
                let shifted = if total_w > 0 {
                    format!("({}) << {}", part, total_w)
                } else {
                    part
                };
                total_w += w;
                shifted
            }).collect();
            if parts.is_empty() { "0".to_string() } else { parts.join(" | ") }
        }
        Expr::Replicate { count, expr } => {
            let w = expr_width(expr, sizes);
            let e = gen_expr_str(expr, sizes, decls);
            let total_w = w * count;
            if total_w > 16 {
                format!("({} as u64).wrapping_mul({}u64.pow({}))", e, 2u64, w) // simplified
            } else {
                let mut v = format!("{}", e);
                for _ in 1..*count {
                    v = format!("({}) << {} | ({})", v, w, e);
                }
                v
            }
        }
        Expr::Select { expr, msb, lsb } => {
            let e = gen_expr_str(expr, sizes, decls);
            let msb_v = eval_const(msb);
            let lsb_v = eval_const(lsb);
            let w = msb_v - lsb_v + 1;
            if w > 1 {
                format!("({} >> {}) & {}", e, lsb_v, mask(w))
            } else {
                format!("({} >> {}) & 1", e, lsb_v)
            }
        }
        Expr::BitSelect { expr, bit } => {
            let e = gen_expr_str(expr, sizes, decls);
            let b = eval_const(bit);
            format!("({} >> {}) & 1", e, b)
        }
        Expr::Cond { cond, if_true, if_false } => {
            let c = gen_expr_str(cond, sizes, decls);
            let t = gen_expr_str(if_true, sizes, decls);
            let f = gen_expr_str(if_false, sizes, decls);
            format!("if {} != Level::L {{ {} }} else {{ {} }}", c, t, f)
        }
    }
}

fn gen_expr_val(expr: &Expr, sizes: &SizeMap, decls: &DeclMap) -> String {
    match expr {
        Expr::Number(n) => n.value.to_string(),
        Expr::Ident(name) => {
            let n = to_snake(name);
            let w = sizes.get(&n).copied().unwrap_or(1);
            if w > 1 {
                format!("bus_to_u16(&self.{}) as u64", n)
            } else {
                format!("get(&self.{}) as u64", n)
            }
        }
        Expr::Binary { op, lhs, rhs } => {
            let l = gen_expr_val(lhs, sizes, decls);
            let r = gen_expr_val(rhs, sizes, decls);
            let lw = expr_width(lhs, sizes);
            let rw = expr_width(rhs, sizes);
            let w = std::cmp::max(lw, rw);
            match op {
                BinaryOp::Add => format!("({} + {}) & {}", l, r, mask(w)),
                BinaryOp::Sub => format!("({}.wrapping_sub({})) & {}", l, r, mask(w)),
                BinaryOp::Mul => format!("({} * {}) & {}", l, r, mask(w)),
                BinaryOp::Div => format!("({} / {})", l, r),
                BinaryOp::Mod => format!("({} % {})", l, r),
                BinaryOp::BitAnd => format!("({} & {})", l, r),
                BinaryOp::BitOr => format!("({} | {})", l, r),
                BinaryOp::BitXor => format!("({} ^ {})", l, r),
                BinaryOp::Lt => format!("if {} < {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Leq => format!("if {} <= {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Gt => format!("if {} > {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Geq => format!("if {} >= {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Eq => format!("if {} == {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Neq => format!("if {} != {} {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::Shl => format!("({} << {})", l, r),
                BinaryOp::Shr => format!("({} >> {})", l, r),
                BinaryOp::Sshl => format!("({} << {})", l, r),
                BinaryOp::Sshr => format!("({} >> {})", l, r),
                BinaryOp::LogicalAnd => format!("if {} != 0 && {} != 0 {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::LogicalOr => format!("if {} != 0 || {} != 0 {{ 1 }} else {{ 0 }}", l, r),
                BinaryOp::BitXnor => format!("!({} ^ {})", l, r),
            }
        }
        Expr::Unary { op, expr } => {
            let e = gen_expr_val(expr, sizes, decls);
            let w = expr_width(expr, sizes);
            match op {
                UnaryOp::Minus => format!("((!{}) + 1) & {}", e, mask(w)),
                UnaryOp::BitNot => format!("!{}", e),
                UnaryOp::LogicalNot => format!("if {} == 0 {{ 1 }} else {{ 0 }}", e),
                UnaryOp::ReduceAnd => format!("if {} == {} {{ 1 }} else {{ 0 }}", e, mask(w)),
                UnaryOp::ReduceOr => format!("if {} != 0 {{ 1 }} else {{ 0 }}", e),
                UnaryOp::ReduceXor => format!("{}.count_ones() as u64 & 1", e),
                _ => e,
            }
        }
        Expr::Concat(items) => {
            let parts: Vec<String> = items.iter().rev().map(|item| {
                let w = expr_width(item, sizes);
                let v = gen_expr_val(item, sizes, decls);
                format!("({}) & {}", v, mask(w))
            }).collect();
            let mut total = 0u64;
            let mut res = String::new();
            for (i, p) in parts.iter().enumerate() {
                if i > 0 { res.push_str(" | "); }
                res.push_str(&format!("({} << {})", p, total));
                total += expr_width(&items[items.len() - 1 - i], sizes);
            }
            if res.is_empty() { "0".to_string() } else { res }
        }
        Expr::Select { expr, msb, lsb } => {
            let e = gen_expr_val(expr, sizes, decls);
            let msb_v = eval_const(msb);
            let lsb_v = eval_const(lsb);
            format!("({} >> {}) & {}", e, lsb_v, mask(msb_v - lsb_v + 1))
        }
        Expr::BitSelect { expr, bit } => {
            let e = gen_expr_val(expr, sizes, decls);
            let b = eval_const(bit);
            format!("({} >> {}) & 1", e, b)
        }
        Expr::Replicate { count, expr } => {
            let w = expr_width(expr, sizes);
            let e = gen_expr_val(expr, sizes, decls);
            let mut v = e.clone();
            for _ in 1..*count {
                v = format!("({} << {}) | ({})", v, w, e);
            }
            v
        }
        Expr::Cond { cond, if_true, if_false } => {
            let c = gen_expr_val(cond, sizes, decls);
            let t = gen_expr_val(if_true, sizes, decls);
            let f = gen_expr_val(if_false, sizes, decls);
            format!("if {} != 0 {{ {} }} else {{ {} }}", c, t, f)
        }
    }
}

fn gen_expr_bus_val(expr: &Expr, sizes: &SizeMap, decls: &DeclMap, width: u64) -> String {
    let s = gen_expr_val(expr, sizes, decls);
    if width <= 16 {
        s
    } else {
        s
    }
}

fn expr_to_var(expr: &Expr, _sizes: &SizeMap) -> String {
    match expr {
        Expr::Ident(name) => {
            to_snake(name)
        }
        Expr::BitSelect { expr, bit } => {
            if let Expr::Ident(name) = expr.as_ref() {
                let n = to_snake(name);
                format!("{}[{}]", n, eval_const(bit))
            } else {
                String::new()
            }
        }
        Expr::Select { expr, msb, .. } => {
            if let Expr::Ident(name) = expr.as_ref() {
                let n = to_snake(name);
                let mut v = Vec::new();
                let msb_v = eval_const(msb);
                for i in 0..=msb_v {
                    v.push(format!("{}[{}]", n, i));
                }
                format!("vec![{}]", v.join(", "))
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

// ----- helper types -----

type DeclMap = std::collections::HashMap<String, DeclInfo>;
type SizeMap = std::collections::HashMap<String, u64>;

#[derive(Clone)]
struct DeclInfo {
    kind: DeclKind,
    width: Option<Range>,
}

#[derive(Clone)]
#[allow(dead_code)]
enum DeclKind { Port(PortDir), Wire, Reg, Integer }

fn build_decl_map<'a>(
    ports: &[Port],
    wires: &[&VarDecl],
    regs: &[&VarDecl],
    integers: &[&String],
) -> DeclMap {
    let mut m = DeclMap::new();
    for p in ports {
        m.insert(to_snake(&p.name), DeclInfo { kind: DeclKind::Port(p.direction), width: p.width.clone() });
    }
    for v in wires {
        m.insert(to_snake(&v.name), DeclInfo { kind: DeclKind::Wire, width: v.width.clone() });
    }
    for v in regs {
        m.insert(to_snake(&v.name), DeclInfo { kind: DeclKind::Reg, width: v.width.clone() });
    }
    for n in integers {
        m.insert(to_snake(n), DeclInfo { kind: DeclKind::Integer, width: None });
    }
    m
}

fn build_size_map(decls: &DeclMap) -> SizeMap {
    let mut m = SizeMap::new();
    for (name, info) in decls {
        let w = match &info.kind {
            DeclKind::Integer => 32,
            _ => width_val(&info.width),
        };
        m.insert(name.clone(), w);
    }
    m
}

fn width_val(w: &Option<Range>) -> u64 {
    match w {
        Some(r) if r.msb >= r.lsb => r.msb - r.lsb + 1,
        Some(r) => r.lsb - r.msb + 1,
        None => 1,
    }
}

fn expr_width(e: &Expr, sizes: &SizeMap) -> u64 {
    match e {
        Expr::Number(n) => n.width.unwrap_or_else(|| { let v = n.value; if v == 0 { 1 } else { 64 - v.leading_zeros() as u64 }.max(1) }),
        Expr::Ident(name) => sizes.get(&to_snake(name)).copied().unwrap_or(1),
        Expr::Binary { lhs, rhs, .. } => std::cmp::max(expr_width(lhs, sizes), expr_width(rhs, sizes)),
        Expr::Unary { expr, .. } => expr_width(expr, sizes),
        Expr::Concat(items) => items.iter().map(|i| expr_width(i, sizes)).sum(),
        Expr::Replicate { count, expr } => count * expr_width(expr, sizes),
        Expr::Select { msb, lsb, .. } => eval_const(msb) - eval_const(lsb) + 1,
        Expr::BitSelect { .. } => 1,
        Expr::Cond { if_true, if_false, .. } => std::cmp::max(expr_width(if_true, sizes), expr_width(if_false, sizes)),
    }
}

fn is_port(ports: &[Port], name: &str) -> bool {
    ports.iter().any(|p| p.name == name)
}

fn mask(w: u64) -> String {
    let bits = w.min(16);
    if bits >= 64 { "u64::MAX".to_string() }
    else { format!("{}u64", (1u64 << bits) - 1) }
}

fn port_type(p: &Port, sizes: &SizeMap) -> String {
    let sn = to_snake(&p.name);
    let w = sizes.get(&sn).copied().unwrap_or(1);
    if w > 1 {
        format!("Vec<WireRef>")
    } else {
        "WireRef".to_string()
    }
}

fn var_type(v: &VarDecl, sizes: &SizeMap) -> String {
    let sn = to_snake(&v.name);
    let w = sizes.get(&sn).copied().unwrap_or(1);
    if w > 1 {
        format!("Vec<WireRef>")
    } else {
        "WireRef".to_string()
    }
}

fn verilog_gate_to_rust(gate: &str) -> &str {
    match gate {
        "and" => "And",
        "nand" => "Nand",
        "or" => "Or",
        "nor" => "Nor",
        "xor" => "Xor",
        "xnor" => "Xor", // ruHDL doesn't have Xnor; use Xor + not
        "not" => "Not",
        "buf" => "Not", // buf passes through; Not is closest
        _ => "And",
    }
}

fn to_pascal(s: &str) -> String {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars = p.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn to_snake(s: &str) -> String {
    let mut out = String::new();
    let s = s.replace(|c: char| !c.is_ascii_alphanumeric() && c != '_', "_");
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

pub fn gen_ruhdl(modules: &[Module]) -> String {
    let mut out = String::new();
    out.push_str("use ruhdl::prelude::*;\n\n");
    let mut has_initial = false;
    let mut tb_name = String::new();
    for m in modules {
        let code = gen_module(m);
        let has_init = m.items.iter().any(|item| matches!(item, ModuleItem::Initial(_)));
        if has_init {
            has_initial = true;
            tb_name = to_pascal(&m.name);
        }
        out.push_str(&code);
    }
    if has_initial && !tb_name.is_empty() {
        out.push_str(&format!("fn main() {{\n"));
        out.push_str(&format!("    let mut tb = {}::new();\n", tb_name));
        out.push_str(&format!("    tb.run();\n"));
        out.push_str(&format!("}}\n"));
    }
    out
}
