mod declarations;
mod expressions;
mod matcher;
mod statements;
mod types;

use self::matcher::Matcher;
use crate::ast::Declarations;
use crate::messages::MessageMaker;
use crate::names::{NameNode, NamePart};
use crate::source::{Source, Span};
use crate::token::{lex, Token};
use crate::Db;

#[salsa::tracked]
pub fn parse(db: &dyn Db, source: Source) -> Declarations {
    let tokens = lex(db, source);
    let mut parser = Parser {
        db,

        tokens: tokens.as_slice(),
        last_span: tokens.first().map(|(_, span)| *span),
        source,
    };

    parser.parse_top_level()
}

pub struct Parser<'a> {
    db: &'a dyn Db,

    tokens: &'a [(Token, Span)],
    last_span: Option<Span>,
    source: Source,
}

impl<'a> Parser<'a> {
    fn closest_span(&self) -> Span {
        self.tokens
            .first()
            .map(|(_, span)| *span)
            .or(self.last_span)
            .unwrap_or_else(|| Span::new(self.source, 0, 0))
    }

    #[must_use]
    fn this_one(&self) -> Option<&'a (Token, Span)> {
        self.tokens.first()
    }

    #[must_use]
    fn next(&mut self) -> Option<&'a (Token, Span)> {
        if self.tokens.is_empty() {
            None
        } else {
            self.tokens = &self.tokens[1..];

            let first = self.tokens.first();
            self.last_span = first.map(|(_, span)| *span).or(self.last_span);

            first
        }
    }

    fn is_done(&self) -> bool {
        self.tokens.is_empty()
    }

    #[must_use]
    fn matches(&self, m: impl Matcher) -> Option<Span> {
        if let Some((token, span)) = self.this_one() {
            m.matches(token).then_some(*span)
        } else {
            None
        }
    }

    #[must_use]
    fn consume(&mut self, m: impl Matcher) -> Option<Span> {
        if let Some(span) = self.matches(m) {
            let _ = self.next();
            Some(span)
        } else {
            None
        }
    }

    /// Parse a name token (type or value). May produce a `NameNode::Invalid`.
    fn parse_name<T, F>(&mut self, f: F) -> (T, Span)
    where
        F: FnOnce(&mut Parser<'a>, NamePart, Span) -> T,
    {
        match self.this_one() {
            Some((Token::ValueName(name), span)) => {
                let _ = self.next();
                let name = NamePart::new(self.db, NameNode::Value(name.clone()));
                (f(self, name, *span), *span)
            }

            Some((Token::TypeName(name), span)) => {
                let _ = self.next();
                let name = NamePart::new(self.db, NameNode::Type(name.clone()));
                (f(self, name, *span), *span)
            }

            Some((_, span)) => {
                let name = NamePart::new(self.db, NameNode::Invalid);
                (f(self, name, *span), *span)
            }

            None => {
                let span = self.closest_span();
                let name = NamePart::new(self.db, NameNode::Invalid);
                (f(self, name, span), span)
            }
        }
    }

    fn at(&self, span: Span) -> MessageMaker<'a> {
        MessageMaker::at(self.db, span)
    }
}
