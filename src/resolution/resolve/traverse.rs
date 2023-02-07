use itertools::{Either, Itertools};

use super::{ClassOrValue, Contextual, Resolver};
use crate::names::{Name, NamePart};
use crate::source::Span;
use crate::{ast, rst};

impl Resolver {
    pub(super) fn resolve_class<'a>(
        this: &mut Contextual<Self>,
        name: rst::DeclarationName,
        span: Span,
        kind: rst::ClassKind,
        inherits: &[ast::Type],
        items: impl Iterator<Item = &'a ast::Declaration>,
    ) -> Name {
        let inherits = inherits
            .iter()
            .map(|ty| Self::resolve_type(this, ty))
            .collect();

        let (classes, values) = Self::item_scope(this, name, |this| {
            items
                .map(|item| Self::resolve_item(this, item))
                .partition_map(|item| match item {
                    ClassOrValue::Class(class) => Either::Left(class),
                    ClassOrValue::Value(value) => Either::Right(value),
                })
        });

        let fields = rst::Fields { classes, values };
        let class = rst::Class {
            name,
            kind,
            inherits,
            fields,
            span,
        };

        let name = Self::make_scope_name(this, name);
        this.data.classes.insert(name, class);

        name
    }

    pub(super) fn resolve_variable(
        this: &mut Contextual<Self>,
        name: rst::DeclarationName,
        span: Span,
        anno: &ast::Type,
        body: &Option<ast::Expression>,
    ) -> rst::Value {
        let anno = Self::resolve_type(this, anno);
        let body = body
            .as_ref()
            .map(|expr| Self::resolve_expression(this, expr));

        rst::Value {
            name,
            node: rst::ValueNode::Variable { anno, body },
            span,
        }
    }

    pub(super) fn resolve_function(
        this: &mut Contextual<Self>,
        name: rst::DeclarationName,
        span: Span,
        this_arg: &Option<usize>,
        args: &[(NamePart, ast::Type)],
        return_type: &ast::Type,
        body: &Option<ast::Block>,
    ) -> rst::Value {
        Self::local_scope(this, name, |this| {
            let args = args
                .iter()
                .map(|(name, ty)| {
                    let name = Self::declare(this, name);
                    let ty = Self::resolve_type(this, ty);
                    (name, ty)
                })
                .collect();

            let return_type = Self::resolve_type(this, return_type);

            let body = body.as_ref().map(|body| Self::resolve_block(this, body));

            rst::Value {
                name,
                node: rst::ValueNode::Function {
                    this: *this_arg,
                    args,
                    return_type,
                    body,
                },
                span,
            }
        })
    }

    fn resolve_type(this: &mut Contextual<Self>, ty: &ast::Type) -> rst::Type {
        let node = match &ty.node {
            ast::TypeNode::Name(name) => match Self::resolve(this, ty.span, *name) {
                Some(name) => rst::TypeNode::Name(name),
                None => rst::TypeNode::Invalid,
            },

            ast::TypeNode::Field(of, field) => {
                let of = Box::new(Self::resolve_type(this, of));
                rst::TypeNode::Field(of, *field)
            }

            ast::TypeNode::Applied(to, args) => {
                let to = Box::new(Self::resolve_type(this, to));
                let args = args.iter().map(|ty| Self::resolve_type(this, ty)).collect();
                rst::TypeNode::Applied(to, args)
            }

            ast::TypeNode::Function(args, to) => {
                let args = args.iter().map(|ty| Self::resolve_type(this, ty)).collect();
                let to = Box::new(Self::resolve_type(this, to));
                rst::TypeNode::Function(args, to)
            }

            ast::TypeNode::Reference(of) => {
                let of = Box::new(Self::resolve_type(this, of));
                rst::TypeNode::Reference(of)
            }

            ast::TypeNode::Int => rst::TypeNode::Int,
            ast::TypeNode::Nat => rst::TypeNode::Nat,
            ast::TypeNode::Boolean => rst::TypeNode::Boolean,
            ast::TypeNode::Unit => rst::TypeNode::Unit,
            ast::TypeNode::Invalid => rst::TypeNode::Invalid,
        };

        rst::Type {
            node,
            span: ty.span,
        }
    }

    fn resolve_block(this: &mut Contextual<Self>, block: &ast::Block) -> rst::Block {
        let mut declarations = Vec::new();
        let mut statements = Vec::new();

        for statement in block.0.iter() {
            statements.push(Self::resolve_statement(this, &mut declarations, statement));
        }

        rst::Block {
            declarations,
            statements,
        }
    }

    fn resolve_statement(
        this: &mut Contextual<Self>,
        declarations: &mut Vec<(Name, rst::Type)>,
        statement: &ast::Statement,
    ) -> rst::Statement {
        let span = statement.span;

        let node = match &statement.node {
            ast::StatementNode::Expression(expr) => {
                let expr = Self::resolve_expression(this, expr);
                rst::StatementNode::Expression(expr)
            }

            ast::StatementNode::Variable(name, anno, body) => {
                let name = Self::declare(this, name);
                let anno = Self::resolve_type(this, anno);
                let body = Self::resolve_expression(this, body);

                assert!(this.data.mutable.insert(name));
                declarations.push((name, anno));

                let target = rst::Expression {
                    node: rst::ExpressionNode::Name(name),
                    span,
                };

                rst::StatementNode::Assignment(target, body)
            }

            ast::StatementNode::Constant(name, anno, body) => {
                let name = Self::declare(this, name);
                let anno = Self::resolve_type(this, anno);
                let body = Self::resolve_expression(this, body);

                declarations.push((name, anno));

                let target = rst::Expression {
                    node: rst::ExpressionNode::Name(name),
                    span,
                };

                rst::StatementNode::Assignment(target, body)
            }

            ast::StatementNode::Assignment(target, body) => {
                let target = Self::resolve_expression(this, target);
                let body = Self::resolve_expression(this, body);

                rst::StatementNode::Assignment(target, body)
            }

            ast::StatementNode::Return(expr) => {
                let expr = Self::resolve_expression(this, expr);

                rst::StatementNode::Return(expr)
            }

            ast::StatementNode::Null => rst::StatementNode::Null,
            ast::StatementNode::Invalid => rst::StatementNode::Expression(rst::Expression {
                node: rst::ExpressionNode::Invalid,
                span,
            }),
        };

        rst::Statement { node, span }
    }

    fn resolve_expression(this: &mut Contextual<Self>, expr: &ast::Expression) -> rst::Expression {
        let node = match &expr.node {
            ast::ExpressionNode::Reference(expr) => {
                let expr = Box::new(Self::resolve_expression(this, expr));
                rst::ExpressionNode::Reference(expr)
            }

            ast::ExpressionNode::Call(fun, args) => {
                let fun = Box::new(Self::resolve_expression(this, fun));
                let args = args
                    .iter()
                    .map(|expr| Self::resolve_expression(this, expr))
                    .collect();
                rst::ExpressionNode::Call(fun, args)
            }

            ast::ExpressionNode::Field(of, field) => {
                let of = Box::new(Self::resolve_expression(this, of));
                rst::ExpressionNode::Field(of, *field)
            }

            ast::ExpressionNode::Name(name) => match Self::resolve(this, expr.span, *name) {
                Some(name) => rst::ExpressionNode::Name(name),
                None => rst::ExpressionNode::Invalid,
            },

            ast::ExpressionNode::Number(num) => rst::ExpressionNode::Number(num.clone()),
            ast::ExpressionNode::String(num) => rst::ExpressionNode::String(num.clone()),
            ast::ExpressionNode::This => rst::ExpressionNode::This,
            ast::ExpressionNode::Unit => rst::ExpressionNode::Unit,

            ast::ExpressionNode::Invalid => rst::ExpressionNode::Invalid,
        };

        rst::Expression {
            node,
            span: expr.span,
        }
    }
}
