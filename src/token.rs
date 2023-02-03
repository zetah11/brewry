use logos::{Lexer, Logos};

use crate::source::{Source, Span};
use crate::Db;

#[salsa::tracked(return_ref)]
pub fn lex(db: &dyn Db, source: Source) -> Vec<(Token, Span)> {
    let text = source.text(db);
    Lexer::new(text.as_ref())
        .spanned()
        .map(|(token, range)| (token, Span::new(source, range.start, range.end)))
        .collect()
}

#[derive(Logos, Clone, Debug, Eq, PartialEq)]
pub enum Token {
    #[token("case")]
    Case,
    #[token("class")]
    Class,
    #[token("end")]
    End,
    #[token("function")]
    Function,
    #[token("is")]
    Is,
    #[token("let")]
    Let,
    #[token("null")]
    Null,
    #[token("private")]
    Private,
    #[token("return")]
    Return,
    #[token("this")]
    This,
    #[token("var")]
    Var,
    #[token("variant")]
    Variant,

    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("[")]
    OpenBracket,
    #[token("]")]
    CloseBracket,

    #[token("&")]
    Ampersand,
    #[token(":=")]
    ColonEqual,

    #[token(",")]
    Comma,
    #[token(".")]
    Dot,

    #[token("<")]
    Less,
    #[token("=")]
    Equal,
    #[token(">")]
    Greater,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,

    #[regex(r"\d[0-9_]*(\.\d[0-9_])?", |lex| lex.slice().to_string())]
    Number(String),
    #[regex("\"[^\"]+\"", string_content)]
    String(String),
    #[regex(r"[A-Z][a-zA-Z_'?!]*", |lex| lex.slice().to_string())]
    TypeName(String),
    #[regex(r"[a-z][a-zA-Z_'?!]*", |lex| lex.slice().to_string())]
    ValueName(String),

    #[regex(r"[ \r\n\t]+", logos::skip)]
    #[error]
    Invalid,
}

impl Token {
    pub fn type_name(&self) -> Option<&String> {
        match self {
            Self::TypeName(name) => Some(name),
            _ => None,
        }
    }

    pub fn value_name(&self) -> Option<&String> {
        match self {
            Self::ValueName(name) => Some(name),
            _ => None,
        }
    }
}

fn string_content(lexer: &Lexer<Token>) -> String {
    let slice = lexer.slice();
    slice[1..slice.len() - 1].to_string()
}
