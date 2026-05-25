use std::collections::HashMap;
use crate::ast::*;

struct Codegen {
    output: String,
    reg_counter: u64,
    label_counter: u64,
    vars: HashMap<String, (String, Type)>,
    current_fn: Option<String>,
    fn_return_type: Type,
    has_print: bool,
}

impl Codegen {
    fn new() -> Self {
        Codegen {
            output: String::new(),
            reg_counter: 0,
            label_counter: 0,
            vars: HashMap::new(),
            current_fn: None,
            fn_return_type: Type::Unit,
            has_print: false,
        }
    }

    fn next_reg(&mut self) -> String {
        let n = self.reg_counter;
        self.reg_counter += 1;
        format!("%{}", n)
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let n = self.label_counter;
        self.label_counter += 1;
        format!("{}{}", prefix, n)
    }

    fn llvm_type(&self, ty: &Type) -> &str {
        match ty {
            Type::I32 => "i32",
            Type::I64 => "i64",
            Type::Bool => "i1",
            Type::Unit => "void",
        }
    }

    fn llvm_func_ret_type(&self, func: &Function) -> String {
        if func.name == "main" {
            "i32".to_string()
        } else {
            self.llvm_type(&func.ret_type).to_string()
        }
    }

    fn infer_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_) => Type::I32,
            Expr::Bool(_) => Type::Bool,
            Expr::Ident(name) => {
                if let Some((_, ty)) = self.vars.get(name) {
                    ty.clone()
                } else {
                    panic!("undefined variable '{}'", name);
                }
            }
            Expr::Binary(op, _, _) => match op {
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge | BinOp::And | BinOp::Or => Type::Bool,
                _ => Type::I32,
            },
            Expr::Unary(op, _) => match op {
                UnaryOp::Not => Type::Bool,
                UnaryOp::Neg => Type::I32,
            },
            Expr::Call(name, _) => {
                if name == "print_int" {
                    Type::Unit
                } else {
                    Type::I32
                }
            }
        }
    }

    fn stmts_always_return(&self, stmts: &[Stmt]) -> bool {
        let mut i = 0;
        while i < stmts.len() {
            if self.stmt_always_returns(&stmts[i]) {
                return i == stmts.len() - 1;
            }
            i += 1;
        }
        false
    }

    fn stmt_always_returns(&self, stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Return(_) => true,
            Stmt::If { then_body, else_body, .. } => {
                if let Some(eb) = else_body {
                    self.stmts_always_return(then_body) && self.stmts_always_return(eb)
                } else {
                    false
                }
            }
            Stmt::Block(stmts) => self.stmts_always_return(stmts),
            Stmt::While { .. } => false,
            _ => false,
        }
    }

    fn generate_prologue(&mut self) {
        self.output.push_str("; ModuleID = 'rustc4'\n");
        self.output.push_str("target triple = \"arm64-apple-darwin\"\n\n");
        self.output.push_str("declare i32 @printf(i8*, ...)\n\n");
        self.output.push_str("@.print_int_fmt = private unnamed_addr constant [4 x i8] c\"%d\\0A\\00\"\n\n");
    }

    fn generate_print_int(&mut self) {
        if !self.has_print {
            return;
        }
        self.output.push_str("define void @print_int(i32 %x) {\n");
        self.output.push_str("  %fmt = getelementptr [4 x i8], [4 x i8]* @.print_int_fmt, i64 0, i64 0\n");
        self.output.push_str("  call i32 (i8*, ...) @printf(i8* %fmt, i32 %x)\n");
        self.output.push_str("  ret void\n");
        self.output.push_str("}\n\n");
    }

    fn generate_program(&mut self, program: &Program) -> String {
        self.generate_prologue();

        for func in &program.functions {
            if func.name == "print_int" {
                continue;
            }
            for stmt in &func.body {
                self.collect_print_calls(stmt);
            }
        }

        self.generate_print_int();

        for func in &program.functions {
            if func.name == "print_int" {
                continue;
            }
            self.generate_function(func);
        }

        self.output.clone()
    }

    fn collect_print_calls(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { init, .. } => self.collect_print_expr(init),
            Stmt::Return(Some(e)) => self.collect_print_expr(e),
            Stmt::If { cond, then_body, else_body } => {
                self.collect_print_expr(cond);
                for s in then_body { self.collect_print_calls(s); }
                if let Some(eb) = else_body {
                    for s in eb { self.collect_print_calls(s); }
                }
            }
            Stmt::While { cond, body } => {
                self.collect_print_expr(cond);
                for s in body { self.collect_print_calls(s); }
            }
            Stmt::Assign { value, .. } => self.collect_print_expr(value),
            Stmt::Expr(e) => self.collect_print_expr(e),
            Stmt::Block(stmts) => {
                for s in stmts { self.collect_print_calls(s); }
            }
            Stmt::Return(None) => {}
        }
    }

    fn collect_print_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(name, args) => {
                if name == "print_int" {
                    self.has_print = true;
                }
                for a in args { self.collect_print_expr(a); }
            }
            Expr::Binary(_, l, r) => {
                self.collect_print_expr(l);
                self.collect_print_expr(r);
            }
            Expr::Unary(_, e) => self.collect_print_expr(e),
            _ => {}
        }
    }

    fn generate_function(&mut self, func: &Function) {
        let is_main = func.name == "main";
        let ret_ty = self.llvm_func_ret_type(func);

        self.output.push_str(&format!("define {} @{}(", ret_ty, func.name));

        for (i, (pname, pty)) in func.params.iter().enumerate() {
            if i > 0 { self.output.push_str(", "); }
            self.output.push_str(&format!("{} %{}", self.llvm_type(pty), pname));
        }
        self.output.push_str(") {\n");

        let entry_label = self.next_label("entry");
        self.output.push_str(&format!("{}:\n", entry_label));

        self.vars.clear();
        self.reg_counter = 0;
        self.label_counter = 0;
        self.current_fn = Some(func.name.clone());
        self.fn_return_type = func.ret_type.clone();

        for (pname, pty) in &func.params {
            let ptr = self.next_reg();
            self.output.push_str(&format!(
                "  {} = alloca {}\n", ptr, self.llvm_type(pty)
            ));
            self.output.push_str(&format!(
                "  store {} %{}, {}* {}\n",
                self.llvm_type(pty), pname, self.llvm_type(pty), ptr
            ));
            self.vars.insert(pname.clone(), (ptr.clone(), pty.clone()));
        }

        let has_return = self.generate_stmts(&func.body);

        if !has_return {
            if is_main {
                self.output.push_str("  ret i32 0\n");
            } else if func.ret_type == Type::Unit {
                self.output.push_str("  ret void\n");
            } else {
                panic!(
                    "function '{}' must return a value of type {}",
                    func.name, func.ret_type
                );
            }
        }

        self.output.push_str("}\n\n");
        self.current_fn = None;
    }

    fn generate_stmts(&mut self, stmts: &[Stmt]) -> bool {
        let mut has_return = false;
        for stmt in stmts {
            match stmt {
                Stmt::Block(s) => {
                    has_return = self.generate_stmts(s) || has_return;
                }
                _ => {
                    self.generate_stmt(stmt);
                    if self.stmt_always_returns(stmt) {
                        has_return = true;
                    }
                }
            }
        }
        has_return
    }

    fn generate_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, ty: _, init, .. } => {
                let inferred = self.infer_type(init);
                let ptr = self.next_reg();
                self.output.push_str(&format!(
                    "  {} = alloca {}\n", ptr, self.llvm_type(&inferred)
                ));
                let (val_reg, _) = self.generate_expr(init);
                self.output.push_str(&format!(
                    "  store {} {}, {}* {}\n",
                    self.llvm_type(&inferred), val_reg, self.llvm_type(&inferred), ptr
                ));
                self.vars.insert(name.clone(), (ptr, inferred));
            }
            Stmt::Return(Some(expr)) => {
                let (val_reg, ty) = self.generate_expr(expr);
                if self.current_fn.as_deref() == Some("main") {
                    self.output.push_str(&format!("  ret i32 {}\n", val_reg));
                } else {
                    self.output.push_str(&format!(
                        "  ret {} {}\n", self.llvm_type(&ty), val_reg
                    ));
                }
            }
            Stmt::Return(None) => {
                if self.current_fn.as_deref() == Some("main") {
                    self.output.push_str("  ret i32 0\n");
                } else {
                    self.output.push_str("  ret void\n");
                }
            }
            Stmt::If { cond, then_body, else_body } => {
                self.generate_if(cond, then_body, else_body.as_deref());
            }
            Stmt::While { cond, body } => {
                self.generate_while(cond, body);
            }
            Stmt::Assign { name, value } => {
                let entry = self.vars.get(name).cloned();
                if let Some((ptr, ty)) = entry {
                    let (val_reg, _) = self.generate_expr(value);
                    self.output.push_str(&format!(
                        "  store {} {}, {}* {}\n",
                        self.llvm_type(&ty), val_reg, self.llvm_type(&ty), ptr
                    ));
                } else {
                    panic!("undefined variable '{}'", name);
                }
            }
            Stmt::Expr(expr) => {
                self.generate_expr(expr);
            }
            Stmt::Block(stmts) => {
                self.generate_stmts(stmts);
            }
        }
    }

    fn generate_if(&mut self, cond: &Expr, then_body: &[Stmt], else_body: Option<&[Stmt]>) {
        let (cond_reg, _) = self.generate_expr(cond);
        let then_label = self.next_label("if_then");
        let else_label = self.next_label("if_else");
        let end_label = self.next_label("if_end");

        if let Some(eb) = else_body {
            self.output.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond_reg, then_label, else_label
            ));

            self.output.push_str(&format!("{}:\n", then_label));
            let then_returns = self.generate_stmts(then_body);
            if !then_returns {
                self.output.push_str(&format!("  br label %{}\n", end_label));
            }

            self.output.push_str(&format!("{}:\n", else_label));
            let else_returns = self.generate_stmts(eb);
            if !else_returns {
                self.output.push_str(&format!("  br label %{}\n", end_label));
            }

            if !then_returns || !else_returns {
                self.output.push_str(&format!("{}:\n", end_label));
            }
        } else {
            self.output.push_str(&format!(
                "  br i1 {}, label %{}, label %{}\n",
                cond_reg, then_label, end_label
            ));

            self.output.push_str(&format!("{}:\n", then_label));
            let then_returns = self.generate_stmts(then_body);
            if !then_returns {
                self.output.push_str(&format!("  br label %{}\n", end_label));
            }

            self.output.push_str(&format!("{}:\n", end_label));
        }
    }

    fn generate_while(&mut self, cond: &Expr, body: &[Stmt]) {
        let cond_label = self.next_label("while_cond");
        let body_label = self.next_label("while_body");
        let end_label = self.next_label("while_end");

        self.output.push_str(&format!("  br label %{}\n", cond_label));
        self.output.push_str(&format!("{}:\n", cond_label));

        let (cond_reg, _) = self.generate_expr(cond);
        self.output.push_str(&format!(
            "  br i1 {}, label %{}, label %{}\n",
            cond_reg, body_label, end_label
        ));

        self.output.push_str(&format!("{}:\n", body_label));
        let body_returns = self.generate_stmts(body);
        if !body_returns {
            self.output.push_str(&format!("  br label %{}\n", cond_label));
        }
        self.output.push_str(&format!("{}:\n", end_label));
    }

    fn generate_expr(&mut self, expr: &Expr) -> (String, Type) {
        match expr {
            Expr::Int(val) => {
                let reg = self.next_reg();
                self.output.push_str(&format!(
                    "  {} = add i32 0, {}\n", reg, val
                ));
                (reg, Type::I32)
            }
            Expr::Bool(val) => {
                let reg = self.next_reg();
                let v = if *val { "true" } else { "false" };
                self.output.push_str(&format!(
                    "  {} = add i1 0, {}\n", reg, v
                ));
                (reg, Type::Bool)
            }
            Expr::Ident(name) => {
                let entry = self.vars.get(name).cloned();
                if let Some((ptr, ty)) = entry {
                    let reg = self.next_reg();
                    self.output.push_str(&format!(
                        "  {} = load {}, {}* {}\n",
                        reg, self.llvm_type(&ty), self.llvm_type(&ty), ptr
                    ));
                    (reg, ty)
                } else {
                    panic!("undefined variable '{}'", name);
                }
            }
            Expr::Binary(op, left, right) => {
                self.generate_binary(op, left, right)
            }
            Expr::Unary(op, expr) => {
                self.generate_unary(op, expr)
            }
            Expr::Call(name, args) => {
                self.generate_call(name, args)
            }
        }
    }

    fn generate_binary(&mut self, op: &BinOp, left: &Expr, right: &Expr) -> (String, Type) {
        let (l_reg, l_ty) = self.generate_expr(left);
        let (r_reg, _) = self.generate_expr(right);
        let result = self.next_reg();

        match op {
            BinOp::Add => {
                self.output.push_str(&format!("  {} = add {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, l_ty)
            }
            BinOp::Sub => {
                self.output.push_str(&format!("  {} = sub {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, l_ty)
            }
            BinOp::Mul => {
                self.output.push_str(&format!("  {} = mul {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, l_ty)
            }
            BinOp::Div => {
                self.output.push_str(&format!("  {} = sdiv {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, l_ty)
            }
            BinOp::Mod => {
                self.output.push_str(&format!("  {} = srem {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, l_ty)
            }
            BinOp::Eq => {
                self.output.push_str(&format!("  {} = icmp eq {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Ne => {
                self.output.push_str(&format!("  {} = icmp ne {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Lt => {
                self.output.push_str(&format!("  {} = icmp slt {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Gt => {
                self.output.push_str(&format!("  {} = icmp sgt {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Le => {
                self.output.push_str(&format!("  {} = icmp sle {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Ge => {
                self.output.push_str(&format!("  {} = icmp sge {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::And => {
                self.output.push_str(&format!("  {} = and {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
            BinOp::Or => {
                self.output.push_str(&format!("  {} = or {} {}, {}\n", result, self.llvm_type(&l_ty), l_reg, r_reg));
                (result, Type::Bool)
            }
        }
    }

    fn generate_unary(&mut self, op: &UnaryOp, expr: &Expr) -> (String, Type) {
        let (reg, ty) = self.generate_expr(expr);
        let result = self.next_reg();

        match op {
            UnaryOp::Neg => {
                self.output.push_str(&format!("  {} = sub {} 0, {}\n", result, self.llvm_type(&ty), reg));
                (result, ty)
            }
            UnaryOp::Not => {
                self.output.push_str(&format!("  {} = xor {} {}, true\n", result, self.llvm_type(&ty), reg));
                (result, Type::Bool)
            }
        }
    }

    fn generate_call(&mut self, name: &str, args: &[Expr]) -> (String, Type) {
        if name == "print_int" {
            if args.len() != 1 {
                panic!("print_int takes exactly 1 argument");
            }
            let (arg_reg, _) = self.generate_expr(&args[0]);
            self.output.push_str(&format!("  call void @print_int(i32 {})\n", arg_reg));
            (String::new(), Type::Unit)
        } else {
            let mut arg_parts = Vec::new();
            for arg in args {
                let (reg, ty) = self.generate_expr(arg);
                arg_parts.push(format!("{} {}", self.llvm_type(&ty), reg));
            }
            let result = self.next_reg();
            self.output.push_str(&format!(
                "  {} = call i32 @{}({})\n", result, name, arg_parts.join(", ")
            ));
            (result, Type::I32)
        }
    }
}

pub fn generate_ir(program: &Program) -> String {
    let mut codegen = Codegen::new();
    codegen.generate_program(program)
}

pub fn compile_to_ir(source: &str) -> String {
    let program = crate::parser::parse_source(source);
    generate_ir(&program)
}
