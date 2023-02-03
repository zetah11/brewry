use super::Parser;
use crate::ast::{Block, Expression, ExpressionNode, NameNode, Statement, StatementNode};
use crate::token::Token;

impl Parser<'_> {
    pub const STATEMENT_START: &[Token] = &[
        Token::Return,
        Token::Let,
        Token::Var,
        Token::ValueName(String::new()),
        Token::Number(String::new()),
        Token::String(String::new()),
        Token::OpenParen,
    ];

    pub fn parse_block(&mut self) -> Block {
        let mut statements = Vec::new();

        while self.matches(Self::STATEMENT_START).is_some() {
            statements.push(self.statement());
        }

        Block(statements)
    }

    fn statement(&mut self) -> Statement {
        match self.this_one() {
            Some((Token::Null, span)) => {
                let _ = self.next();
                Statement {
                    node: StatementNode::Null,
                    span: *span,
                }
            }

            Some((Token::Return, opener)) => {
                let _ = self.next();
                let expr = self
                    .matches(Self::EXPR_STARTS)
                    .is_some()
                    .then(|| self.parse_expression())
                    .unwrap_or(Expression {
                        node: ExpressionNode::Unit,
                        span: *opener,
                    });

                let span = *opener + expr.span;

                Statement {
                    node: StatementNode::Return(expr),
                    span,
                }
            }

            Some((Token::Let, opener)) => {
                let _ = self.next();
                let (name, _) = self.parse_name(|this, name, span| {
                    match name.node(this.db) {
                        NameNode::Value(..) => {}
                        NameNode::Type(name) => {
                            this.at(span).parse_expected_value_name(Some(name.as_str()))
                        }
                        NameNode::Invalid => this.at(span).parse_expected_value_name(None),
                    }

                    name
                });

                let ty = self.parse_type();

                let _ = self.consume(Token::ColonEqual).unwrap_or({
                    let span = self.closest_span();
                    self.at(span).parse_expected_assignment();
                    span
                });

                let body = self.parse_expression();

                let span = *opener + body.span;

                Statement {
                    node: StatementNode::Constant(name, ty, body),
                    span,
                }
            }

            Some((Token::Var, opener)) => {
                let _ = self.next();
                let (name, _) = self.parse_name(|this, name, span| {
                    match name.node(this.db) {
                        NameNode::Value(..) => {}
                        NameNode::Type(name) => {
                            this.at(span).parse_expected_value_name(Some(name.as_str()))
                        }
                        NameNode::Invalid => this.at(span).parse_expected_value_name(None),
                    }

                    name
                });

                let ty = self.parse_type();

                let _ = self.consume(Token::ColonEqual).unwrap_or({
                    let span = self.closest_span();
                    self.at(span).parse_expected_assignment();
                    span
                });

                let body = self.parse_expression();

                let span = *opener + body.span;

                Statement {
                    node: StatementNode::Variable(name, ty, body),
                    span,
                }
            }

            Some(_) => {
                let expr = self.parse_expression();
                self.expression_or_assignment(expr)
            }

            None => unreachable!(),
        }
    }

    fn expression_or_assignment(&mut self, expr: Expression) -> Statement {
        let mut span = expr.span;
        let node = match expr.node {
            ExpressionNode::Name(name) if self.consume(Token::ColonEqual).is_some() => {
                let body = self.parse_expression();
                span += body.span;
                StatementNode::Assignment(name, body)
            }

            node => StatementNode::Expression(Expression {
                node,
                span: expr.span,
            }),
        };

        Statement { node, span }
    }
}
