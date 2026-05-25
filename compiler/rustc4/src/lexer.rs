use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    IntLit(i64),
    True,
    False,
    Let,
    Mut,
    Fn,
    If,
    Else,
    While,
    Return,
    I32,
    I64,
    Bool,
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    AndAnd,
    OrOr,
    Not,
    Eq,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Colon,
    Arrow,
    Comma,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => {
                    self.advance();
                }
                Some('/') if self.peek_next() == Some('/') => {
                    while self.peek().map_or(false, |c| c != '\n') {
                        self.advance();
                    }
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        s.push(first);
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let val: i64 = s.parse().unwrap_or(0);
        Token {
            kind: TokenKind::IntLit(val),
            line: start_line,
            col: start_col,
        }
    }

    fn read_ident_or_keyword(&mut self, first: char) -> Token {
        let start_line = self.line;
        let start_col = self.col;
        let mut s = String::new();
        s.push(first);
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match s.as_str() {
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "fn" => TokenKind::Fn,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "i32" => TokenKind::I32,
            "i64" => TokenKind::I64,
            "bool" => TokenKind::Bool,
            _ => TokenKind::Ident(s),
        };
        Token { kind, line: start_line, col: start_col }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        let line = self.line;
        let col = self.col;

        match self.peek() {
            None => Token { kind: TokenKind::EOF, line, col },
            Some(ch) => {
                self.advance();
                match ch {
                    '+' => Token { kind: TokenKind::Plus, line, col },
                    '-' => {
                        if self.peek() == Some('>') {
                            self.advance();
                            Token { kind: TokenKind::Arrow, line, col }
                        } else {
                            Token { kind: TokenKind::Minus, line, col }
                        }
                    }
                    '*' => Token { kind: TokenKind::Star, line, col },
                    '/' => Token { kind: TokenKind::Slash, line, col },
                    '%' => Token { kind: TokenKind::Percent, line, col },
                    '=' => {
                        if self.peek() == Some('=') {
                            self.advance();
                            Token { kind: TokenKind::EqEq, line, col }
                        } else {
                            Token { kind: TokenKind::Eq, line, col }
                        }
                    }
                    '!' => {
                        if self.peek() == Some('=') {
                            self.advance();
                            Token { kind: TokenKind::Ne, line, col }
                        } else {
                            Token { kind: TokenKind::Not, line, col }
                        }
                    }
                    '<' => {
                        if self.peek() == Some('=') {
                            self.advance();
                            Token { kind: TokenKind::Le, line, col }
                        } else {
                            Token { kind: TokenKind::Lt, line, col }
                        }
                    }
                    '>' => {
                        if self.peek() == Some('=') {
                            self.advance();
                            Token { kind: TokenKind::Ge, line, col }
                        } else {
                            Token { kind: TokenKind::Gt, line, col }
                        }
                    }
                    '&' => {
                        if self.peek() == Some('&') {
                            self.advance();
                            Token { kind: TokenKind::AndAnd, line, col }
                        } else {
                            panic!("lexer error: unexpected '&' at {}:{}", line, col);
                        }
                    }
                    '|' => {
                        if self.peek() == Some('|') {
                            self.advance();
                            Token { kind: TokenKind::OrOr, line, col }
                        } else {
                            panic!("lexer error: unexpected '|' at {}:{}", line, col);
                        }
                    }
                    '(' => Token { kind: TokenKind::LParen, line, col },
                    ')' => Token { kind: TokenKind::RParen, line, col },
                    '{' => Token { kind: TokenKind::LBrace, line, col },
                    '}' => Token { kind: TokenKind::RBrace, line, col },
                    ';' => Token { kind: TokenKind::Semicolon, line, col },
                    ':' => Token { kind: TokenKind::Colon, line, col },
                    ',' => Token { kind: TokenKind::Comma, line, col },
                    ch if ch.is_ascii_digit() => self.read_number(ch),
                    ch if ch.is_alphabetic() || ch == '_' => self.read_ident_or_keyword(ch),
                    _ => panic!("lexer error: unexpected character '{}' at {}:{}", ch, line, col),
                }
            }
        }
    }
}

pub fn lex_source(source: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let tok = lexer.next_token();
        let is_eof = matches!(tok.kind, TokenKind::EOF);
        tokens.push(tok);
        if is_eof {
            return tokens;
        }
    }
}
