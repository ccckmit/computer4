use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek().kind == *kind
    }

    fn check_int_lit(&self) -> bool {
        matches!(self.peek().kind, TokenKind::IntLit(_))
    }

    fn check_ident(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Ident(_))
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            return true;
        }
        false
    }

    fn expect(&mut self, kind: &TokenKind) {
        if !self.eat(kind) {
            panic!(
                "expected {:?}, got {:?} at {}:{}",
                kind,
                self.peek().kind,
                self.peek().line,
                self.peek().col
            );
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut functions = Vec::new();
        while !self.check(&TokenKind::EOF) {
            functions.push(self.parse_function());
        }
        Program { functions }
    }

    fn parse_function(&mut self) -> Function {
        self.expect(&TokenKind::Fn);
        let name = self.expect_ident();
        self.expect(&TokenKind::LParen);
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            params.push(self.parse_param());
            while self.eat(&TokenKind::Comma) {
                params.push(self.parse_param());
            }
        }
        self.expect(&TokenKind::RParen);
        let ret_type = if self.eat(&TokenKind::Arrow) {
            self.parse_type()
        } else {
            Type::Unit
        };
        let body = self.parse_block();
        Function { name, params, ret_type, body }
    }

    fn expect_ident(&mut self) -> String {
        if let TokenKind::Ident(s) = &self.peek().kind {
            let s = s.clone();
            self.advance();
            s
        } else {
            panic!(
                "expected identifier, got {:?} at {}:{}",
                self.peek().kind,
                self.peek().line,
                self.peek().col
            );
        }
    }

    fn expect_int(&mut self) -> i64 {
        if let TokenKind::IntLit(n) = &self.peek().kind {
            let n = *n;
            self.advance();
            n
        } else {
            panic!(
                "expected integer, got {:?} at {}:{}",
                self.peek().kind,
                self.peek().line,
                self.peek().col
            );
        }
    }

    fn parse_param(&mut self) -> (String, Type) {
        let name = self.expect_ident();
        self.expect(&TokenKind::Colon);
        let ty = self.parse_type();
        (name, ty)
    }

    fn parse_type(&mut self) -> Type {
        match &self.peek().kind {
            TokenKind::I32 => { self.advance(); Type::I32 }
            TokenKind::I64 => { self.advance(); Type::I64 }
            TokenKind::Bool => { self.advance(); Type::Bool }
            _ => panic!(
                "expected type, got {:?} at {}:{}",
                self.peek().kind,
                self.peek().line,
                self.peek().col
            ),
        }
    }

    fn parse_block(&mut self) -> Vec<Stmt> {
        self.expect(&TokenKind::LBrace);
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            stmts.push(self.parse_stmt());
        }
        self.expect(&TokenKind::RBrace);
        stmts
    }

    fn parse_stmt(&mut self) -> Stmt {
        match &self.peek().kind {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::LBrace => Stmt::Block(self.parse_block()),
            _ => {
                if self.check_int_lit()
                    || self.check(&TokenKind::True)
                    || self.check(&TokenKind::False)
                    || self.check_ident()
                    || self.check(&TokenKind::LParen)
                    || self.check(&TokenKind::Minus)
                    || self.check(&TokenKind::Not)
                {
                    let expr = self.parse_expr();
                    if self.check(&TokenKind::Eq) && matches!(&expr, Expr::Ident(_)) {
                        self.advance();
                        let value = self.parse_expr();
                        self.expect(&TokenKind::Semicolon);
                        let name = if let Expr::Ident(s) = &expr {
                            s.clone()
                        } else {
                            unreachable!()
                        };
                        return Stmt::Assign { name, value };
                    }
                    self.expect(&TokenKind::Semicolon);
                    return Stmt::Expr(expr);
                }
                panic!(
                    "unexpected token {:?} at {}:{}",
                    self.peek().kind,
                    self.peek().line,
                    self.peek().col
                );
            }
        }
    }

    fn parse_let(&mut self) -> Stmt {
        self.expect(&TokenKind::Let);
        let mutable = self.eat(&TokenKind::Mut);
        let name = self.expect_ident();
        let ty = if self.eat(&TokenKind::Colon) {
            Some(self.parse_type())
        } else {
            None
        };
        self.expect(&TokenKind::Eq);
        let init = self.parse_expr();
        self.expect(&TokenKind::Semicolon);
        Stmt::Let { name, mutable, ty, init }
    }

    fn parse_return(&mut self) -> Stmt {
        self.expect(&TokenKind::Return);
        if self.check(&TokenKind::Semicolon) {
            self.advance();
            return Stmt::Return(None);
        }
        let expr = self.parse_expr();
        self.expect(&TokenKind::Semicolon);
        Stmt::Return(Some(expr))
    }

    fn parse_if(&mut self) -> Stmt {
        self.expect(&TokenKind::If);
        let cond = self.parse_expr();
        let then_body = self.parse_block();
        let else_body = if self.eat(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                Some(vec![self.parse_if()])
            } else {
                Some(self.parse_block())
            }
        } else {
            None
        };
        Stmt::If { cond, then_body, else_body }
    }

    fn parse_while(&mut self) -> Stmt {
        self.expect(&TokenKind::While);
        let cond = self.parse_expr();
        let body = self.parse_block();
        Stmt::While { cond, body }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Expr {
        let mut left = self.parse_and();
        while self.eat(&TokenKind::OrOr) {
            let right = self.parse_and();
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_and(&mut self) -> Expr {
        let mut left = self.parse_cmp();
        while self.eat(&TokenKind::AndAnd) {
            let right = self.parse_cmp();
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_cmp(&mut self) -> Expr {
        let left = self.parse_add();
        if self.check(&TokenKind::EqEq) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Eq, Box::new(left), Box::new(right));
        }
        if self.check(&TokenKind::Ne) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Ne, Box::new(left), Box::new(right));
        }
        if self.check(&TokenKind::Lt) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Lt, Box::new(left), Box::new(right));
        }
        if self.check(&TokenKind::Gt) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Gt, Box::new(left), Box::new(right));
        }
        if self.check(&TokenKind::Le) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Le, Box::new(left), Box::new(right));
        }
        if self.check(&TokenKind::Ge) {
            self.advance();
            let right = self.parse_add();
            return Expr::Binary(BinOp::Ge, Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_add(&mut self) -> Expr {
        let mut left = self.parse_mul();
        loop {
            if self.check(&TokenKind::Plus) {
                self.advance();
                let right = self.parse_mul();
                left = Expr::Binary(BinOp::Add, Box::new(left), Box::new(right));
            } else if self.check(&TokenKind::Minus) {
                self.advance();
                let right = self.parse_mul();
                left = Expr::Binary(BinOp::Sub, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_mul(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            if self.check(&TokenKind::Star) {
                self.advance();
                let right = self.parse_unary();
                left = Expr::Binary(BinOp::Mul, Box::new(left), Box::new(right));
            } else if self.check(&TokenKind::Slash) {
                self.advance();
                let right = self.parse_unary();
                left = Expr::Binary(BinOp::Div, Box::new(left), Box::new(right));
            } else if self.check(&TokenKind::Percent) {
                self.advance();
                let right = self.parse_unary();
                left = Expr::Binary(BinOp::Mod, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        if self.check(&TokenKind::Minus) {
            self.advance();
            let expr = self.parse_unary();
            return Expr::Unary(UnaryOp::Neg, Box::new(expr));
        }
        if self.check(&TokenKind::Not) {
            self.advance();
            let expr = self.parse_unary();
            return Expr::Unary(UnaryOp::Not, Box::new(expr));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Expr {
        let mut expr = self.parse_primary();
        loop {
            if self.check(&TokenKind::LParen) {
                self.advance();
                let mut args = Vec::new();
                if !self.check(&TokenKind::RParen) {
                    args.push(self.parse_expr());
                    while self.eat(&TokenKind::Comma) {
                        args.push(self.parse_expr());
                    }
                }
                self.expect(&TokenKind::RParen);
                if let Expr::Ident(name) = expr {
                    expr = Expr::Call(name, args);
                } else {
                    panic!("cannot call non-identifier expression");
                }
            } else {
                break;
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        if self.check_int_lit() {
            let val = self.expect_int();
            return Expr::Int(val);
        }
        if self.check(&TokenKind::True) {
            self.advance();
            return Expr::Bool(true);
        }
        if self.check(&TokenKind::False) {
            self.advance();
            return Expr::Bool(false);
        }
        if self.check_ident() {
            let name = self.expect_ident();
            return Expr::Ident(name);
        }
        if self.check(&TokenKind::LParen) {
            self.advance();
            let expr = self.parse_expr();
            self.expect(&TokenKind::RParen);
            return expr;
        }
        panic!(
            "unexpected token {:?} at {}:{}",
            self.peek().kind,
            self.peek().line,
            self.peek().col
        );
    }
}

pub fn parse_source(source: &str) -> Program {
    let tokens = crate::lexer::lex_source(source);
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}
