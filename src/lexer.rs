use std::ops::Range;
use crate::obj::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub meta: Meta,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, Default)]
pub struct Meta {
    pub range: Range<usize>,
}

impl Meta {
    pub fn join(l: &Self, r: &Self) -> Self {
        Self {
            range: l.range.start..r.range.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    ParenOpen,
    ParenClose,
    Period,
    Ellipsis,
    SingleQuote,

    DefineSyntax,
    SyntaxRules,
    Load,
    Define,
    Lambda,
    Quote,
    Set,
    Let,
    LetAster,
    LetRec,
    If,
    Cond,
    Else,
    And,
    Or,
    Begin,
    Do,

    Id(String),
    Num(Number),
    Bool(bool),
    Str(String),
}

pub fn get_tokens(src: String) -> Vec<Token> {
    let mut reader = reader::Reader::new(src);

    let mut tokens = vec![];

    let mut idx = 0;

    while let Some(symbol) = read_next_symbol(&mut reader) {
        let mut idx_cur = reader.idx();

        let meta = Meta {
            range: idx..idx_cur,
        };

        idx = idx_cur;

        match symbol.as_str() {
            " " => None,
            "\n" => None,
            "(" => Some(TokenKind::ParenOpen),
            ")" => Some(TokenKind::ParenClose),
            "." => Some(TokenKind::Period),
            "..." => Some(TokenKind::Ellipsis),
            "'" => Some(TokenKind::SingleQuote),
            "define-syntax" => Some(TokenKind::DefineSyntax),
            "syntax-rules" => Some(TokenKind::SyntaxRules),
            "load" => Some(TokenKind::Load),
            "define" => Some(TokenKind::Define),
            "lambda" => Some(TokenKind::Lambda),
            "quote" => Some(TokenKind::Quote),
            "set!" => Some(TokenKind::Set),
            "let" => Some(TokenKind::Let),
            "let*" => Some(TokenKind::LetAster),
            "letrec" => Some(TokenKind::LetRec),
            "if" => Some(TokenKind::If),
            "cond" => Some(TokenKind::Cond),
            "else" => Some(TokenKind::Else),
            "and" => Some(TokenKind::And),
            "or" => Some(TokenKind::Or),
            "begin" => Some(TokenKind::Begin),
            "do" => Some(TokenKind::Do),
            _ => {
                if symbol.starts_with('"') && symbol.ends_with('"') {
                    Some(TokenKind::Str(
                        symbol.chars().skip(1).take(symbol.chars().count() - 2).collect(),
                    ))
                } else if symbol == "#t" {
                    Some(TokenKind::Bool(true))
                } else if symbol == "#f" {
                    Some(TokenKind::Bool(false))
                } else if let Ok(n) = symbol.parse::<i64>() {
                    Some(TokenKind::Num(Number::from(n)))
                } else if let Ok(n) = symbol.parse::<f64>() {
                    Some(TokenKind::Num(Number::from(n)))
                } else {
                    // TODO: id validation
                    Some(TokenKind::Id(symbol.clone()))
                }
            }
        }
        .map(|t| tokens.push(Token { meta, kind: t }));
    }

    tokens
}

fn read_next_symbol(reader: &mut reader::Reader) -> Option<String> {
    while reader.peek() == Some(';') {
        while reader.has_data() && reader.read() != Some('\n') {}
    }

    if !reader.has_data() {
        return None;
    }

    if reader.peek() == Some('"') {
        return read_next_string(reader);
    }

    let mut symbol = "".to_string();

    loop {
        if let Some(c) = reader.read() {
            symbol.push(c);
        }

        if reader.is_symbol_ended() {
            break;
        }
    }

    Some(symbol)
}

fn read_next_string(reader: &mut reader::Reader) -> Option<String> {
    if !reader.has_data() || reader.read() != Some('"') {
        return None;
    }

    let mut str = "\"".to_string();

    loop {
        let c = reader.read()?;

        if c == '\\' {
            let c = reader.read()?;
            str.push(match c {
                'n' => '\n',
                _ => c,
            });
            continue;
        }

        str.push(c);

        if c == '"' {
            break;
        }
    }

    Some(str)
}

mod reader {
    pub struct Reader {
        src: Vec<char>,
        idx: usize,
    }

    impl Reader {
        pub fn new(src: String) -> Self {
            Self {
                src: src.chars().collect(),
                idx: 0,
            }
        }

        pub fn peek(&self) -> Option<char> {
            self.src.get(self.idx).map(|c| *c)
        }

        pub fn read(&mut self) -> Option<char> {
            let c = self.peek();
            self.idx += 1;
            c
        }

        pub fn is_symbol_ended(&self) -> bool {
            let separators = vec![' ', '(', ')', '\n', ';', '\''];

            self.src.get(self.idx - 1).map(|c| separators.contains(c)).unwrap_or(false)
                || self.src.get(self.idx).map(|c| separators.contains(c)).unwrap_or(false)
                || self.idx >= self.src.len()
        }

        pub fn has_data(&self) -> bool {
            self.idx < self.src.len()
        }

        pub fn idx(&self) -> usize {
            self.idx
        }
    }
}
