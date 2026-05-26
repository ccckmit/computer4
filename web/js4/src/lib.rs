use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Let, Function, Return, If, Else, While, Try, Catch, Throw, Break, Continue,
    True, False, Null, Undefined,
    Identifier(String), Number(f64), StringLiteral(String),
    Plus, Minus, Star, Slash, Percent, Assign, Eq, Lt, Gt, LtEq, GtEq, NotEq, StrictEq, StrictNotEq, AmperAmper, PipePipe, Bang,
    Comma, Dot, Semicolon, Colon, Question, LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Eof,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() { i += 1; continue; }
        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            while i < chars.len() && chars[i] != '\n' { i += 1; }
            continue;
        }
        if c == '"' || c == '\'' {
            let quote = c; i += 1; let mut s = String::new();
            while i < chars.len() && chars[i] != quote { s.push(chars[i]); i += 1; }
            i += 1; tokens.push(Token::StringLiteral(s)); continue;
        }
        if c.is_ascii_digit() {
            let mut num_s = String::new();
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                num_s.push(chars[i]); i += 1;
            }
            tokens.push(Token::Number(num_s.parse().unwrap_or(0.0))); continue;
        }
        if c.is_alphabetic() || c == '_' || c == '$' {
            let mut ident = String::new();
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                ident.push(chars[i]); i += 1;
            }
            let tok = match ident.as_str() {
                "let" => Token::Let, "function" => Token::Function, "return" => Token::Return,
                "if" => Token::If, "else" => Token::Else, "while" => Token::While,
                "try" => Token::Try, "catch" => Token::Catch, "throw" => Token::Throw,
                "break" => Token::Break, "continue" => Token::Continue,
                "true" => Token::True, "false" => Token::False, "null" => Token::Null, "undefined" => Token::Undefined,
                _ => Token::Identifier(ident),
            };
            tokens.push(tok); continue;
        }
        if c == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            if i + 2 < chars.len() && chars[i + 2] == '=' { tokens.push(Token::StrictEq); i += 3; continue; }
            tokens.push(Token::Eq); i += 2; continue;
        }
        if c == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
            if i + 2 < chars.len() && chars[i + 2] == '=' { tokens.push(Token::StrictNotEq); i += 3; continue; }
            tokens.push(Token::NotEq); i += 2; continue;
        }
        if c == '<' && i + 1 < chars.len() && chars[i + 1] == '=' { tokens.push(Token::LtEq); i += 2; continue; }
        if c == '>' && i + 1 < chars.len() && chars[i + 1] == '=' { tokens.push(Token::GtEq); i += 2; continue; }
        if c == '&' && i + 1 < chars.len() && chars[i + 1] == '&' { tokens.push(Token::AmperAmper); i += 2; continue; }
        if c == '|' && i + 1 < chars.len() && chars[i + 1] == '|' { tokens.push(Token::PipePipe); i += 2; continue; }
        let tok = match c {
            '+' => Token::Plus, '-' => Token::Minus, '*' => Token::Star, '/' => Token::Slash, '%' => Token::Percent,
            '=' => Token::Assign, '<' => Token::Lt, '>' => Token::Gt, '!' => Token::Bang, ',' => Token::Comma,
            '.' => Token::Dot, ';' => Token::Semicolon, ':' => Token::Colon, '?' => Token::Question, '(' => Token::LParen, ')' => Token::RParen,
            '{' => Token::LBrace, '}' => Token::RBrace, '[' => Token::LBracket, ']' => Token::RBracket,
            _ => { i += 1; continue; }
        };
        tokens.push(tok); i += 1;
    }
    tokens.push(Token::Eof); tokens
}

#[derive(Clone)]
pub enum Value {
    Undefined, Null, Number(f64), String(String), Boolean(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Object(Rc<RefCell<HashMap<String, Value>>>),
    Function { params: Vec<String>, body: Vec<Stmt>, env: Rc<RefCell<Environment>> },
    Builtin(Rc<dyn Fn(Vec<Value>) -> Value>),
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Undefined | Value::Null => false,
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"), Value::Null => write!(f, "null"),
            Value::Number(n) => write!(f, "{}", n), Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                let items: Vec<String> = arr.borrow().iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj.borrow().iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", items.join(", "))
            }
            Value::Function { .. } | Value::Builtin(_) => write!(f, "[Function]"),
        }
    }
}

pub struct Environment {
    variables: HashMap<String, Value>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self { Environment { variables: HashMap::new(), outer: None } }
    pub fn new_with_outer(outer: Rc<RefCell<Environment>>) -> Self {
        Environment { variables: HashMap::new(), outer: Some(outer) }
    }
    pub fn define(&mut self, name: String, value: Value) { self.variables.insert(name, value); }
    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), Value> {
        if self.variables.contains_key(name) { self.variables.insert(name.to_string(), value); return Ok(()); }
        if let Some(outer) = &self.outer { return outer.borrow_mut().assign(name, value); }
        Err(Value::String(format!("ReferenceError: {} is not defined", name)))
    }
    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.get(name) { return Some(val.clone()); }
        if let Some(outer) = &self.outer { return outer.borrow().get(name); } None
    }
}

#[derive(Clone)]
pub enum Expr {
    Literal(Value), Variable(String), Array(Vec<Expr>), Object(Vec<(String, Expr)>),
    Assign(Box<Expr>, Box<Expr>), Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    Logical(Box<Expr>, String, Box<Expr>), Binary(Box<Expr>, String, Box<Expr>), Unary(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>), Member(Box<Expr>, String), Index(Box<Expr>, Box<Expr>),
}

#[derive(Clone)]
pub enum Stmt {
    Block(Vec<Stmt>), Let(String, Expr), Expression(Expr),
    Function(String, Vec<String>, Vec<Stmt>), Return(Option<Expr>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>), While(Expr, Box<Stmt>),
    Break, Continue, Throw(Expr), TryCatch(Box<Stmt>, String, Box<Stmt>),
}

pub struct Parser { tokens: Vec<Token>, pos: usize }
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0 } }
    pub fn peek(&self) -> &Token { self.tokens.get(self.pos).unwrap_or(&Token::Eof) }
    fn advance(&mut self) -> Token { let t = self.peek().clone(); if t != Token::Eof { self.pos += 1; } t }
    fn expect(&mut self, expected: Token) { if *self.peek() == expected { self.advance(); } else { panic!("Expected {:?}", expected); } }

    pub fn parse_statement(&mut self) -> Stmt {
        let stmt = match self.peek() {
            Token::Let => {
                self.advance();
                let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected ident") };
                self.expect(Token::Assign); let expr = self.parse_expression(); Stmt::Let(name, expr)
            }
            Token::Function => {
                self.advance();
                let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected ident") };
                self.expect(Token::LParen); let mut params = Vec::new();
                if *self.peek() != Token::RParen {
                    params.push(match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected param") });
                    while *self.peek() == Token::Comma { self.advance(); params.push(match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected param") }); }
                }
                self.expect(Token::RParen); let body = self.parse_block(); Stmt::Function(name, params, match body { Stmt::Block(b) => b, _ => unreachable!() })
            }
            Token::Return => {
                self.advance();
                let expr = if *self.peek() != Token::Semicolon && *self.peek() != Token::RBrace && *self.peek() != Token::Eof { Some(self.parse_expression()) } else { None };
                Stmt::Return(expr)
            }
            Token::If => {
                self.advance(); self.expect(Token::LParen); let cond = self.parse_expression(); self.expect(Token::RParen);
                let then_b = self.parse_statement();
                let else_b = if *self.peek() == Token::Else { self.advance(); Some(Box::new(self.parse_statement())) } else { None };
                Stmt::If(cond, Box::new(then_b), else_b)
            }
            Token::While => { self.advance(); self.expect(Token::LParen); let cond = self.parse_expression(); self.expect(Token::RParen); Stmt::While(cond, Box::new(self.parse_statement())) }
            Token::Break => { self.advance(); Stmt::Break }
            Token::Continue => { self.advance(); Stmt::Continue }
            Token::Throw => { self.advance(); Stmt::Throw(self.parse_expression()) }
            Token::Try => {
                self.advance(); let try_b = self.parse_block(); self.expect(Token::Catch); self.expect(Token::LParen);
                let param = match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected catch variable") };
                self.expect(Token::RParen); let catch_b = self.parse_block();
                Stmt::TryCatch(Box::new(try_b), param, Box::new(catch_b))
            }
            Token::LBrace => self.parse_block(),
            _ => Stmt::Expression(self.parse_expression()),
        };
        if *self.peek() == Token::Semicolon { self.advance(); } stmt
    }

    fn parse_block(&mut self) -> Stmt {
        self.expect(Token::LBrace); let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace && *self.peek() != Token::Eof { stmts.push(self.parse_statement()); }
        self.expect(Token::RBrace); Stmt::Block(stmts)
    }

    pub fn parse_expression(&mut self) -> Expr { self.parse_assignment() }
    fn parse_assignment(&mut self) -> Expr {
        let left = self.parse_ternary();
        if *self.peek() == Token::Assign { self.advance(); Expr::Assign(Box::new(left), Box::new(self.parse_assignment())) } else { left }
    }
    fn parse_ternary(&mut self) -> Expr {
        let left = self.parse_logical_or();
        if *self.peek() == Token::Question {
            self.advance();
            let then_branch = self.parse_expression();
            self.expect(Token::Colon);
            let else_branch = self.parse_ternary();
            Expr::Ternary(Box::new(left), Box::new(then_branch), Box::new(else_branch))
        } else { left }
    }
    fn parse_logical_or(&mut self) -> Expr {
        let mut left = self.parse_logical_and();
        while *self.peek() == Token::PipePipe { self.advance(); left = Expr::Logical(Box::new(left), "||".to_string(), Box::new(self.parse_logical_and())); } left
    }
    fn parse_logical_and(&mut self) -> Expr {
        let mut left = self.parse_equality();
        while *self.peek() == Token::AmperAmper { self.advance(); left = Expr::Logical(Box::new(left), "&&".to_string(), Box::new(self.parse_equality())); } left
    }
    fn parse_equality(&mut self) -> Expr {
        let mut left = self.parse_relational();
        while matches!(*self.peek(), Token::Eq | Token::NotEq | Token::StrictEq | Token::StrictNotEq) {
            let op = match self.advance() {
                Token::Eq => "==", Token::NotEq => "!=", Token::StrictEq => "===", Token::StrictNotEq => "!==", _ => unreachable!()
            };
            left = Expr::Binary(Box::new(left), op.to_string(), Box::new(self.parse_relational()));
        } left
    }
    fn parse_relational(&mut self) -> Expr {
        let mut left = self.parse_additive();
        while matches!(*self.peek(), Token::Lt | Token::Gt | Token::LtEq | Token::GtEq) {
            let op = match self.advance() {
                Token::Lt => "<", Token::Gt => ">", Token::LtEq => "<=", Token::GtEq => ">=", _ => unreachable!()
            };
            left = Expr::Binary(Box::new(left), op.to_string(), Box::new(self.parse_additive()));
        } left
    }
    fn parse_additive(&mut self) -> Expr {
        let mut left = self.parse_multiplicative();
        while *self.peek() == Token::Plus || *self.peek() == Token::Minus {
            let op = match self.advance() { Token::Plus => "+", Token::Minus => "-", _ => unreachable!() };
            left = Expr::Binary(Box::new(left), op.to_string(), Box::new(self.parse_multiplicative()));
        } left
    }
    fn parse_multiplicative(&mut self) -> Expr {
        let mut left = self.parse_unary();
        while *self.peek() == Token::Star || *self.peek() == Token::Slash || *self.peek() == Token::Percent {
            let op = match self.advance() { Token::Star => "*", Token::Slash => "/", Token::Percent => "%", _ => unreachable!() };
            left = Expr::Binary(Box::new(left), op.to_string(), Box::new(self.parse_unary()));
        } left
    }
    fn parse_unary(&mut self) -> Expr {
        if *self.peek() == Token::Bang || *self.peek() == Token::Minus {
            let op = match self.advance() { Token::Bang => "!", Token::Minus => "-", _ => unreachable!() };
            Expr::Unary(op.to_string(), Box::new(self.parse_unary()))
        } else { self.parse_call_member_index() }
    }
    fn parse_call_member_index(&mut self) -> Expr {
        let mut left = self.parse_primary();
        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance(); let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expression());
                        while *self.peek() == Token::Comma { self.advance(); args.push(self.parse_expression()); }
                    }
                    self.expect(Token::RParen); left = Expr::Call(Box::new(left), args);
                }
                Token::Dot => { self.advance(); let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Expected property") }; left = Expr::Member(Box::new(left), name); }
                Token::LBracket => { self.advance(); let index = self.parse_expression(); self.expect(Token::RBracket); left = Expr::Index(Box::new(left), Box::new(index)); }
                _ => break,
            }
        } left
    }
    fn parse_primary(&mut self) -> Expr {
        match self.advance() {
            Token::True => Expr::Literal(Value::Boolean(true)), Token::False => Expr::Literal(Value::Boolean(false)),
            Token::Null => Expr::Literal(Value::Null), Token::Undefined => Expr::Literal(Value::Undefined),
            Token::Number(n) => Expr::Literal(Value::Number(n)), Token::StringLiteral(s) => Expr::Literal(Value::String(s)),
            Token::Identifier(s) => Expr::Variable(s),
            Token::LBracket => {
                let mut elements = Vec::new();
                if *self.peek() != Token::RBracket {
                    elements.push(self.parse_expression());
                    while *self.peek() == Token::Comma { self.advance(); elements.push(self.parse_expression()); }
                }
                self.expect(Token::RBracket); Expr::Array(elements)
            }
            Token::LBrace => {
                let mut pairs = Vec::new();
                if *self.peek() != Token::RBrace {
                    let key = match self.advance() { Token::Identifier(s) | Token::StringLiteral(s) => s, _ => panic!("Key needed") };
                    self.expect(Token::Colon); pairs.push((key, self.parse_expression()));
                    while *self.peek() == Token::Comma {
                        self.advance(); if *self.peek() == Token::RBrace { break; }
                        let key = match self.advance() { Token::Identifier(s) | Token::StringLiteral(s) => s, _ => panic!("Key needed") };
                        self.expect(Token::Colon); pairs.push((key, self.parse_expression()));
                    }
                }
                self.expect(Token::RBrace); Expr::Object(pairs)
            }
            Token::LParen => { let expr = self.parse_expression(); self.expect(Token::RParen); expr }
            t => panic!("Unexpected token: {:?}", t),
        }
    }
}

pub enum Signal { None, Return(Value), Break, Continue }

pub struct Interpreter;
impl Interpreter {
    pub fn eval_expr(expr: &Expr, env: &Rc<RefCell<Environment>>) -> Result<Value, Value> {
        match expr {
            Expr::Literal(val) => Ok(val.clone()),
            Expr::Variable(name) => env.borrow().get(name).ok_or_else(|| Value::String(format!("ReferenceError: {} is not defined", name))),
            Expr::Array(elems) => {
                let mut arr = Vec::new();
                for el in elems { arr.push(Self::eval_expr(el, env)?); }
                Ok(Value::Array(Rc::new(RefCell::new(arr))))
            }
            Expr::Object(pairs) => {
                let mut map = HashMap::new();
                for (k, v_expr) in pairs { map.insert(k.clone(), Self::eval_expr(v_expr, env)?); }
                Ok(Value::Object(Rc::new(RefCell::new(map))))
            }
            Expr::Assign(left, right) => {
                let val = Self::eval_expr(right, env)?;
                match &**left {
                    Expr::Variable(name) => { env.borrow_mut().assign(name, val.clone())?; Ok(val) }
                    Expr::Member(obj_e, prop) => {
                        if let Value::Object(map) = Self::eval_expr(obj_e, env)? { map.borrow_mut().insert(prop.clone(), val.clone()); Ok(val) }
                        else { Err(Value::String("TypeError: Cannot set property of non-object".into())) }
                    }
                    Expr::Index(arr_e, idx_e) => {
                        let arr = Self::eval_expr(arr_e, env)?; let idx = Self::eval_expr(idx_e, env)?;
                        if let (Value::Array(vec), Value::Number(i)) = (arr, idx) {
                            let mut borrow = vec.borrow_mut();
                            if (i as usize) < borrow.len() { borrow[i as usize] = val.clone(); Ok(val) }
                            else { Err(Value::String("RangeError: Array index out of bounds".into())) }
                        } else { Err(Value::String("TypeError: Invalid indexing".into())) }
                    }
                    _ => Err(Value::String("ReferenceError: Invalid left-hand side assignment".into())),
                }
            }
            Expr::Ternary(cond, then_branch, else_branch) => {
                if Self::eval_expr(cond, env)?.is_truthy() { Self::eval_expr(then_branch, env) } else { Self::eval_expr(else_branch, env) }
            }
            Expr::Logical(left, op, right) => {
                let l_val = Self::eval_expr(left, env)?;
                if op == "||" { if l_val.is_truthy() { Ok(l_val) } else { Self::eval_expr(right, env) } }
                else { if !l_val.is_truthy() { Ok(l_val) } else { Self::eval_expr(right, env) } }
            }
            Expr::Binary(left, op, right) => {
                let l = Self::eval_expr(left, env)?; let r = Self::eval_expr(right, env)?;
                match op.as_str() {
                    "+" => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                        (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                        (Value::String(a), v) => Ok(Value::String(format!("{}{}", a, v))),
                        (v, Value::String(b)) => Ok(Value::String(format!("{}{}", v, b))),
                        _ => Err(Value::String("TypeError: Invalid + operands".into())),
                    },
                    "-" | "*" | "/" | "%" => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => match op.as_str() {
                            "-" => Ok(Value::Number(a - b)), "*" => Ok(Value::Number(a * b)),
                            "/" => Ok(Value::Number(a / b)), "%" => Ok(Value::Number(a % b)), _ => unreachable!(),
                        },
                        _ => Err(Value::String("TypeError: Numeric operands required".into())),
                    },
                    "==" | "!=" => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(if op == "==" { a == b } else { a != b })),
                        (Value::String(a), Value::String(b)) => Ok(Value::Boolean(if op == "==" { a == b } else { a != b })),
                        (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(if op == "==" { a == b } else { a != b })),
                        (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => Ok(Value::Boolean(op == "==")),
                        _ => Ok(Value::Boolean(op != "==")),
                    },
                    "===" | "!==" => {
                        let strict_eq = op == "===";
                        let same_type = std::mem::discriminant(&l) == std::mem::discriminant(&r);
                        if !same_type { return Ok(Value::Boolean(!strict_eq)); }
                        let result = match (&l, &r) {
                            (Value::Number(a), Value::Number(b)) => a == b,
                            (Value::String(a), Value::String(b)) => a == b,
                            (Value::Boolean(a), Value::Boolean(b)) => a == b,
                            (Value::Null, Value::Null) | (Value::Undefined, Value::Undefined) => true,
                            _ => false,
                        };
                        Ok(Value::Boolean(if strict_eq { result } else { !result }))
                    }
                    "<" | ">" | "<=" | ">=" => match (l, r) {
                        (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(match op.as_str() {
                            "<" => a < b, ">" => a > b, "<=" => a <= b, ">=" => a >= b, _ => unreachable!()
                        })),
                        _ => Err(Value::String("TypeError: Comparable operands required".into())),
                    },
                    _ => unreachable!(),
                }
            }
            Expr::Unary(op, expr) => {
                let v = Self::eval_expr(expr, env)?;
                if op == "!" { Ok(Value::Boolean(!v.is_truthy())) }
                else if op == "-" { if let Value::Number(n) = v { Ok(Value::Number(-n)) } else { Err(Value::String("TypeError".into())) } }
                else { unreachable!() }
            }
            Expr::Member(obj_expr, prop) => {
                if let Value::Object(map) = Self::eval_expr(obj_expr, env)? { Ok(map.borrow().get(prop).cloned().unwrap_or(Value::Undefined)) }
                else { Ok(Value::Undefined) }
            }
            Expr::Index(arr_expr, idx_expr) => {
                let arr = Self::eval_expr(arr_expr, env)?; let idx = Self::eval_expr(idx_expr, env)?;
                if let (Value::Array(vec), Value::Number(i)) = (arr, idx) { Ok(vec.borrow().get(i as usize).cloned().unwrap_or(Value::Undefined)) }
                else { Ok(Value::Undefined) }
            }
            Expr::Call(callee_expr, args_expr) => {
                if let Expr::Member(obj_e, prop) = &**callee_expr {
                    if prop == "push" {
                        if let Value::Array(arr) = Self::eval_expr(obj_e, env)? {
                            let mut vals = Vec::new();
                            for arg in args_expr { vals.push(Self::eval_expr(arg, env)?); }
                            let mut borrow = arr.borrow_mut();
                            for v in vals { borrow.push(v); }
                            return Ok(Value::Number(borrow.len() as f64));
                        }
                    }
                }
                let callee = Self::eval_expr(callee_expr, env)?;
                let mut args = Vec::new();
                for arg in args_expr { args.push(Self::eval_expr(arg, env)?); }
                match callee {
                    Value::Function { params, body, env: closure_env } => {
                        let mut call_env = Environment::new_with_outer(closure_env);
                        for (i, p) in params.iter().enumerate() { call_env.define(p.clone(), args.get(i).cloned().unwrap_or(Value::Undefined)); }
                        match Self::eval_stmt(&Stmt::Block(body), &Rc::new(RefCell::new(call_env)))? {
                            Signal::Return(v) => Ok(v), _ => Ok(Value::Undefined),
                        }
                    }
                    Value::Builtin(func) => Ok(func(args)),
                    _ => Err(Value::String("TypeError: callee is not a function".into())),
                }
            }
        }
    }

    pub fn eval_stmt(stmt: &Stmt, env: &Rc<RefCell<Environment>>) -> Result<Signal, Value> {
        match stmt {
            Stmt::Expression(e) => { Self::eval_expr(e, env)?; Ok(Signal::None) }
            Stmt::Let(name, expr) => { let val = Self::eval_expr(expr, env)?; env.borrow_mut().define(name.clone(), val); Ok(Signal::None) }
            Stmt::Function(name, params, body) => {
                let func = Value::Function { params: params.clone(), body: body.clone(), env: Rc::clone(env) };
                env.borrow_mut().define(name.clone(), func); Ok(Signal::None)
            }
            Stmt::Return(opt_expr) => {
                let val = match opt_expr { Some(e) => Self::eval_expr(e, env)?, None => Value::Undefined };
                Ok(Signal::Return(val))
            }
            Stmt::Block(stmts) => {
                let block_env = Rc::new(RefCell::new(Environment::new_with_outer(Rc::clone(env))));
                for s in stmts {
                    let sig = Self::eval_stmt(s, &block_env)?;
                    if !matches!(sig, Signal::None) { return Ok(sig); }
                } Ok(Signal::None)
            }
            Stmt::If(cond_e, then_s, else_s) => {
                if Self::eval_expr(cond_e, env)?.is_truthy() { Self::eval_stmt(then_s, env) }
                else if let Some(e) = else_s { Self::eval_stmt(e, env) } else { Ok(Signal::None) }
            }
            Stmt::While(cond_e, body_s) => {
                while Self::eval_expr(cond_e, env)?.is_truthy() {
                    match Self::eval_stmt(body_s, env)? {
                        Signal::Break => break, Signal::Continue => continue,
                        Signal::Return(v) => return Ok(Signal::Return(v)), _ => {}
                    }
                } Ok(Signal::None)
            }
            Stmt::Break => Ok(Signal::Break), Stmt::Continue => Ok(Signal::Continue),
            Stmt::Throw(expr) => Err(Self::eval_expr(expr, env)?),
            Stmt::TryCatch(try_s, param_name, catch_s) => {
                match Self::eval_stmt(try_s, env) {
                    Ok(sig) => Ok(sig),
                    Err(thrown_value) => {
                        let catch_env = Rc::new(RefCell::new(Environment::new_with_outer(Rc::clone(env))));
                        catch_env.borrow_mut().define(param_name.clone(), thrown_value);
                        Self::eval_stmt(catch_s, &catch_env)
                    }
                }
            }
        }
    }
}

pub fn create_global_env() -> Rc<RefCell<Environment>> {
    let global_env = Rc::new(RefCell::new(Environment::new()));
    let console_obj = Rc::new(RefCell::new(HashMap::new()));
    console_obj.borrow_mut().insert("log".to_string(), Value::Builtin(Rc::new(|args: Vec<Value>| {
        let output: Vec<String> = args.iter().map(|v| format!("{}", v)).collect();
        println!("{}", output.join(" ")); Value::Undefined
    })));
    global_env.borrow_mut().define("console".to_string(), Value::Object(console_obj));
    global_env
}

pub fn run(code: &str) -> Result<(), Value> {
    let global_env = create_global_env();
    let tokens = tokenize(code);
    let mut parser = Parser::new(tokens);
    let mut statements = Vec::new();
    while *parser.peek() != Token::Eof { statements.push(parser.parse_statement()); }
    Interpreter::eval_stmt(&Stmt::Block(statements), &global_env).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenizer() {
        let tokens = tokenize("let x = 42;");
        assert_eq!(tokens.len(), 6);
    }

    #[test]
    fn test_simple_expression() {
        let result = run("let x = 1 + 2;");
        assert!(result.is_ok());
    }
}