use super::Parser;
use crate::ast::{Type, TypeNode};
use crate::names::{NameNode, NamePart};
use crate::token::Token;

impl Parser<'_> {
    pub const TYPE_STARTS: &[Token] = &[
        Token::TypeName(String::new()),
        Token::Ampersand,
        Token::OpenParen,
    ];

    /// ```abnf
    /// type = prefix-type
    /// ```
    pub fn parse_type(&mut self) -> Type {
        self.prefix_type()
    }

    /// ```abnf
    /// prefix-type  = "&" prefix-type
    /// prefix-type =/ "(" type-list ")" prefix-type
    /// prefix-type =/ long.type
    /// ```
    fn prefix_type(&mut self) -> Type {
        if let Some(opener) = self.consume(Token::Ampersand) {
            let ty = Box::new(self.prefix_type());
            let span = opener + ty.span;

            Type {
                node: TypeNode::Reference(ty),
                span,
            }
        } else if let Some(opener) = self.consume(Token::OpenParen) {
            let args = self.type_list();
            let closer = self.consume(Token::CloseParen).unwrap_or_else(|| {
                self.at(opener).parse_missing_paren();
                self.closest_span()
            });

            let ret = Box::new(self.prefix_type());
            let span = opener + closer + ret.span;

            Type {
                node: TypeNode::Function(args, ret),
                span,
            }
        } else {
            self.long_type()
        }
    }

    /// ```abnf
    /// long-type    = applied-type / field-type / simple-type
    /// applied-type = long-type "(" type-list ")"
    /// field-type   = long-type "." NAME
    /// ```
    fn long_type(&mut self) -> Type {
        let mut ty = self.simple_type();

        loop {
            if let Some(opener) = self.consume(Token::OpenParen) {
                let args = self.type_list();
                let closer = self.consume(Token::CloseParen).unwrap_or_else(|| {
                    self.at(opener).parse_missing_paren();
                    self.closest_span()
                });

                let span = ty.span + closer;

                ty = Type {
                    node: TypeNode::Applied(Box::new(ty), args),
                    span,
                };
            } else if self.consume(Token::Dot).is_some() {
                let span = ty.span;
                let (node, closer) = self.parse_name(|this, name, span| {
                    match name.node(this.db) {
                        NameNode::Type(..) => {}
                        NameNode::Value(name) => {
                            this.at(span).parse_expected_type_name(Some(name.as_str()))
                        }
                        NameNode::Invalid => this.at(span).parse_expected_type_name(None),
                    }

                    TypeNode::Field(Box::new(ty), name)
                });

                let span = span + closer;

                ty = Type { node, span };
            } else {
                break;
            }
        }

        ty
    }

    /// ```abnf
    /// simple-type = NAME / "(" type ")"
    /// ```
    fn simple_type(&mut self) -> Type {
        let (node, span) = match self.this_one() {
            Some((Token::TypeName(name), span)) => {
                let _ = self.next();
                let name = NamePart::new(self.db, NameNode::Type(name.clone()));
                (TypeNode::Name(name), *span)
            }

            Some((Token::OpenParen, opener)) => {
                let _ = self.next();

                let ty = self.parse_type();
                let _closer = self.consume(Token::CloseParen).unwrap_or_else(|| {
                    self.at(*opener).parse_missing_paren();
                    self.closest_span()
                });

                return ty;
            }

            Some((tok, span)) => {
                if let Token::ValueName(name) = tok {
                    self.at(*span).parse_expected_type_name(Some(name.as_str()));
                } else {
                    self.at(*span).parse_expected_type();
                }

                (TypeNode::Invalid, *span)
            }

            None => {
                let span = self.closest_span();
                self.at(span).parse_expected_type();
                (TypeNode::Invalid, span)
            }
        };

        Type { node, span }
    }

    /// ```abnf
    /// type-list = [type *("," type) [","]]
    /// ```
    fn type_list(&mut self) -> Vec<Type> {
        let mut args = Vec::new();

        while self.matches(Self::TYPE_STARTS).is_some() {
            args.push(self.parse_type());

            let _ = self.consume(Token::Comma);
        }

        args
    }
}
