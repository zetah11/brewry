use super::Parser;
use crate::ast::{Expression, ExpressionNode};
use crate::names::{NameNode, NamePart};
use crate::token::Token;

impl Parser<'_> {
    pub const EXPR_STARTS: &[Token] = &[
        Token::ValueName(String::new()),
        Token::Number(String::new()),
        Token::String(String::new()),
        Token::OpenParen,
    ];

    /// ```abnf
    /// expr = prefix-expr
    /// ```
    pub fn parse_expression(&mut self) -> Expression {
        self.prefix_expr()
    }

    /// ```abnf
    /// prefix-expr = "&" prefix-expr / long-expr
    /// ```
    fn prefix_expr(&mut self) -> Expression {
        if let Some(opener) = self.consume(Token::Ampersand) {
            let expr = Box::new(self.prefix_expr());
            let span = opener + expr.span;

            Expression {
                node: ExpressionNode::Reference(expr),
                span,
            }
        } else {
            self.long_expr()
        }
    }

    /// ```abnf
    /// long-expr  = call-expr / field-expr / simple-expr
    /// call-expr  = long-expr "(" expr-list ")"
    /// field-expr = long-expr "." (VALUE_NAME / TYPE_NAME)
    /// ```
    fn long_expr(&mut self) -> Expression {
        let mut expr = self.simple_expr();

        loop {
            if let Some(opener) = self.consume(Token::OpenParen) {
                let args = self.expr_list();
                let closer = self.consume(Token::CloseParen).unwrap_or_else(|| {
                    self.at(opener).parse_missing_paren();
                    args.last().map(|ty| ty.span).unwrap_or(opener)
                });

                let span = expr.span + closer;

                expr = Expression {
                    node: ExpressionNode::Call(Box::new(expr), args),
                    span,
                };
            } else if self.consume(Token::Dot).is_some() {
                let (node, closer) = self.parse_name(|_, name, _| ExpressionNode::NamePart(name));

                let span = expr.span + closer;

                expr = Expression { node, span };
            } else {
                break;
            }
        }

        expr
    }

    /// ```abnf
    /// simple-expr  = VALUE_NAME / TYPE_NAME / NUMBER / STRING
    /// simple-expr =/ "(" expr ")"
    /// ```
    fn simple_expr(&mut self) -> Expression {
        let (node, span) = match self.this_one() {
            Some((Token::ValueName(name), span)) => {
                let _ = self.next();
                let name = NamePart::new(self.db, NameNode::Value(name.clone()));
                (ExpressionNode::NamePart(name), *span)
            }

            Some((Token::TypeName(name), span)) => {
                let _ = self.next();
                let name = NamePart::new(self.db, NameNode::Type(name.clone()));
                (ExpressionNode::NamePart(name), *span)
            }

            Some((Token::Number(number), span)) => {
                let _ = self.next();
                (ExpressionNode::Number(number.clone()), *span)
            }

            Some((Token::String(string), span)) => {
                let _ = self.next();
                (ExpressionNode::String(string.clone()), *span)
            }

            Some((Token::OpenParen, opener)) => {
                let _ = self.next();

                let expr = self.parse_expression();
                let _closer = self.consume(Token::CloseParen).unwrap_or({
                    self.at(*opener).parse_missing_paren();
                    expr.span
                });

                return expr;
            }

            Some((_, span)) => {
                self.at(*span).parse_expected_expression();
                (ExpressionNode::Invalid, *span)
            }

            None => {
                let span = self.closest_span();
                self.at(span).parse_expected_expression();
                (ExpressionNode::Invalid, span)
            }
        };

        Expression { node, span }
    }

    /// ```abnf
    /// expr-list = [expr *("," expr) [","]]
    /// ```
    fn expr_list(&mut self) -> Vec<Expression> {
        let mut args = Vec::new();

        while self.matches(Self::EXPR_STARTS).is_some() {
            args.push(self.parse_expression());

            let _ = self.consume(Token::Comma);
        }

        args
    }
}
