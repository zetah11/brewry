use super::Parser;
use crate::ast::{
    Declaration, DeclarationName, DeclarationNameNode, DeclarationNode, Declarations, Name,
    NameNode, Type, TypeNode,
};
use crate::source::Span;
use crate::token::Token;

impl Parser<'_> {
    pub const DECLARATION_START: &[Token] =
        &[Token::Class, Token::Function, Token::Var, Token::Variant];

    pub fn parse_top_level(&mut self) -> Declarations {
        let mut declarations = Vec::new();

        // Parse the entire thing
        while !self.is_done() {
            declarations.extend(self.declaration());
        }

        Declarations::new(self.db, declarations)
    }

    pub fn parse_declarations(&mut self) -> (Vec<Declaration>, Vec<Declaration>) {
        let mut public = Vec::new();
        let mut private = Vec::new();

        while self.matches(Self::DECLARATION_START).is_some() {
            public.extend(self.declaration());
        }

        if self.consume(Token::Private).is_some() {
            while self.matches(Self::DECLARATION_START).is_some() {
                private.extend(self.declaration());
            }
        }

        (public, private)
    }

    fn declaration(&mut self) -> Option<Declaration> {
        let (node, span) = match self.this_one() {
            Some((Token::Class, opener)) => {
                let _ = self.next();
                let name = self.declaration_name();

                let inherits = self
                    .consume(Token::Is)
                    .map(|_| self.inherits())
                    .unwrap_or_default();

                let (public, private) = self.parse_declarations();

                let end = self.consume(Token::End).unwrap_or_else(|| {
                    self.at(*opener).parse_missing_end();
                    self.closest_span()
                });

                let span = *opener + end;
                let node = DeclarationNode::Class {
                    name,
                    public,
                    private,
                    inherits,
                };

                (node, span)
            }

            Some((Token::Variant, opener)) => {
                let _ = self.next();
                let name = self.declaration_name();

                let inherits = self
                    .consume(Token::Is)
                    .map(|_| self.inherits())
                    .unwrap_or_default();

                let (public, private) = self.parse_declarations();

                let end = self.consume(Token::End).unwrap_or_else(|| {
                    self.at(*opener).parse_missing_end();
                    self.closest_span()
                });

                let span = *opener + end;
                let node = DeclarationNode::Variant {
                    name,
                    public,
                    private,
                    inherits,
                };

                (node, span)
            }

            Some((Token::Function, opener)) => {
                let _ = self.next();
                let name = self.declaration_name();
                let args = self
                    .consume(Token::OpenParen)
                    .map(|opener| {
                        let parameters = self.parameters();

                        let _ = self.consume(Token::CloseParen).unwrap_or_else(|| {
                            self.at(opener).parse_missing_paren();
                            self.closest_span()
                        });

                        parameters
                    })
                    .unwrap_or_default();

                let return_type = self
                    .matches(Self::TYPE_STARTS)
                    .is_some()
                    .then(|| self.parse_type())
                    .unwrap_or(Type {
                        node: TypeNode::Unit,
                        span: self.closest_span(),
                    });

                let body = self
                    .matches(Self::STATEMENT_START)
                    .is_some()
                    .then(|| self.parse_block());

                let end = if body.is_some() {
                    self.consume(Token::End).unwrap_or_else(|| {
                        self.at(*opener).parse_missing_end();
                        self.closest_span()
                    })
                } else {
                    self.closest_span()
                };

                let span = *opener + end;
                let node = DeclarationNode::Function {
                    name,
                    args,
                    return_type,
                    body,
                };

                (node, span)
            }

            Some((Token::Var, opener)) => {
                let _ = self.next();
                let name = self.declaration_name();
                let anno = self.parse_type();

                let body = self
                    .consume(Token::ColonEqual)
                    .map(|_| self.parse_expression());

                let span = *opener + self.closest_span();
                let node = DeclarationNode::Variable { name, anno, body };

                (node, span)
            }

            Some((_, span)) => {
                let _ = self.next();

                self.at(*span).parse_expected_declaration();
                return None;
            }

            None => {
                return None;
            }
        };

        Some(Declaration::new(self.db, node, span))
    }

    fn declaration_name(&mut self) -> DeclarationName {
        let (node, span) = self.simple_name();

        match node {
            DeclarationNameNode::Identifier(prefix) if self.consume(Token::Dot).is_some() => {
                let (node, end_span) = self.simple_name();
                let span = span + end_span;
                DeclarationName {
                    node,
                    prefix: Some(prefix),
                    span,
                }
            }

            _ => DeclarationName {
                node,
                prefix: None,
                span,
            },
        }
    }

    fn simple_name(&mut self) -> (DeclarationNameNode, Span) {
        match self.this_one() {
            Some((Token::TypeName(name), span)) => {
                let _ = self.next();
                let name = Name::new(self.db, NameNode::Type(name.clone()));
                (DeclarationNameNode::Identifier(name), *span)
            }

            Some((Token::ValueName(name), span)) => {
                let _ = self.next();
                let name = Name::new(self.db, NameNode::Value(name.clone()));
                (DeclarationNameNode::Identifier(name), *span)
            }

            Some((Token::String(name), span)) => {
                let _ = self.next();
                (DeclarationNameNode::Quoted(name.clone()), *span)
            }

            Some((tok, span)) => {
                self.at(*span)
                    .parse_expected_type_name(tok.type_name().map(|s| s.as_str()));
                (DeclarationNameNode::Invalid, *span)
            }

            None => {
                let span = self.closest_span();
                self.at(span).parse_expected_type_name(None);
                (DeclarationNameNode::Invalid, span)
            }
        }
    }

    fn inherits(&mut self) -> Vec<Type> {
        let mut types = Vec::new();

        while self.matches(Self::TYPE_STARTS).is_some() {
            types.push(self.parse_type());

            let _ = self.consume(Token::Comma);
        }

        types
    }

    fn parameters(&mut self) -> Vec<(Name, Type)> {
        let mut names = Vec::new();
        let mut types = Vec::new();

        while let Some((Token::ValueName(name), _span)) = self.this_one() {
            let _ = self.next();
            let name = Name::new(self.db, NameNode::Value(name.clone()));
            names.push(name);

            if self.matches(Self::TYPE_STARTS).is_some() {
                let ty = self.parse_type();
                while types.len() < names.len() {
                    types.push(ty.clone());
                }
            }

            let _ = self.consume(Token::Comma);
        }

        names.into_iter().zip(types).collect()
    }
}
