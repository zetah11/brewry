use std::collections::HashSet;

use itertools::{Either, Itertools};

use super::all_names_within;
use super::{Contextual, NamesWithin};
use crate::ast;
use crate::names::{Name, NameNode, NamePart, NamePrefix};
use crate::parse::parse;
use crate::rst;
use crate::source::{Source, Span};
use crate::Db;

#[salsa::tracked]
pub struct NameInfo {
    pub mutable: HashSet<Name>,
    pub tree: rst::Items,
}

#[salsa::tracked]
pub fn resolve_names(db: &dyn Db, source: Source) -> NameInfo {
    let mut classes = Vec::new();
    let mut values = Vec::new();

    let tree = parse(db, source);
    let names = all_names_within(db, source);

    let mut resolver = Resolver::new(db, source, names);
    for item in tree.declarations(db) {
        match Resolver::resolve_item(&mut resolver, item) {
            ClassOrValue::Class(class) => classes.push(class),
            ClassOrValue::Value(value) => values.push(value),
        }
    }

    let tree = rst::Items::new(db, classes, values);
    NameInfo::new(db, resolver.data.mutable, tree)
}

enum ClassOrValue {
    Class(rst::Class),
    Value(rst::Value),
}

struct Resolver {
    names: NamesWithin,
    mutable: HashSet<Name>,
    locals: Vec<Vec<Name>>,
}

impl Resolver {
    pub fn new(db: &dyn Db, source: Source, names: NamesWithin) -> Contextual<Self> {
        let this = Self {
            names,
            mutable: HashSet::new(),
            locals: Vec::new(),
        };

        Contextual::new(db, this, source)
    }

    pub fn resolve_item(this: &mut Contextual<Self>, item: &ast::Declaration) -> ClassOrValue {
        let name = Self::resolve_decl_name(this, item.name(this.db));
        let span = item.span(this.db);

        match item.node(this.db) {
            ast::DeclarationNode::Class {
                inherits,
                private,
                public,
            } => {
                let class = Self::resolve_class(
                    this,
                    name,
                    span,
                    rst::ClassKind::Class,
                    inherits,
                    private.iter().chain(public),
                );

                ClassOrValue::Class(class)
            }

            ast::DeclarationNode::Variant {
                inherits,
                private,
                public,
            } => {
                let class = Self::resolve_class(
                    this,
                    name,
                    span,
                    rst::ClassKind::Variant,
                    inherits,
                    private.iter().chain(public),
                );

                ClassOrValue::Class(class)
            }

            ast::DeclarationNode::Variable { anno, body } => {
                let value = Self::resolve_variable(this, name, span, anno, body);
                ClassOrValue::Value(value)
            }

            ast::DeclarationNode::Function {
                this: this_arg,
                args,
                return_type,
                body,
            } => {
                let value =
                    Self::resolve_function(this, name, span, this_arg, args, return_type, body);
                ClassOrValue::Value(value)
            }
        }
    }

    fn resolve_class<'a>(
        this: &mut Contextual<Self>,
        name: rst::DeclarationName,
        span: Span,
        kind: rst::ClassKind,
        inherits: &[ast::Type],
        items: impl Iterator<Item = &'a ast::Declaration>,
    ) -> rst::Class {
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

        let items = rst::Items::new(this.db, classes, values);
        rst::Class {
            name,
            kind,
            inherits,
            items,
            span,
        }
    }

    fn resolve_variable(
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

    fn resolve_function(
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

    fn resolve_decl_name(
        this: &mut Contextual<Self>,
        name: &ast::DeclarationName,
    ) -> rst::DeclarationName {
        match this.declaration_name(name) {
            Some(name) => rst::DeclarationName::Name(name),
            None => {
                let scope = name
                    .prefix
                    .expect("`this.declaration_name` should only return `None` on prefixed names");
                let Some(scope) = Self::resolve(this, name.span, scope) else {
                    return rst::DeclarationName::Invalid;
                };

                let name = match name.node {
                    ast::DeclarationNameNode::Identifier(ident) => ident,
                    ast::DeclarationNameNode::Quoted(_) => todo!(),
                    ast::DeclarationNameNode::Invalid => NamePart::new(this.db, NameNode::Invalid),
                };

                rst::DeclarationName::Field(scope, name)
            }
        }
    }

    /// Declare a local variable.
    fn declare(this: &mut Contextual<Self>, name: &NamePart) -> Name {
        let scope = this.prefix();
        let scope = NamePrefix::Local(Box::new(scope), this.data.locals.len());

        let name = Name::new(this.db, scope, *name);
        let locals = this
            .data
            .locals
            .last_mut()
            .expect("cannot declare a local in a non-local");

        if locals.contains(&name) {
            // todo: error!!!
        } else {
            locals.push(name);
        }

        name
    }

    fn resolve(this: &mut Contextual<Self>, span: Span, name: NamePart) -> Option<Name> {
        // Look for locals...
        for scope in this.data.locals.iter().rev() {
            for var in scope.iter().rev() {
                if var.name(this.db) == name {
                    return Some(*var);
                }
            }
        }

        // Then search up through the scopes in the current file...
        let names = this.data.names.names(this.db);

        let (source, mut scopes) = this.within.clone();
        while let Some(scope) = scopes.pop() {
            let Some(names) = names.get(&scope) else { continue; };
            let name = Name::new(this.db, NamePrefix::Item(scope), name);

            if names.contains(&name) {
                return Some(name);
            }
        }

        // Then look if this is a top-level name...
        let name = Name::new(this.db, NamePrefix::Source(source), name);
        if names.contains_key(&name) {
            return Some(name);
        }

        // No name!
        this.at(span).resolve_unresolved_name();
        None
    }

    fn item_scope<T, F>(this: &mut Contextual<Self>, name: rst::DeclarationName, f: F) -> T
    where
        F: FnOnce(&mut Contextual<Self>) -> T,
    {
        let name = Self::make_scope_name(this, name);
        this.in_scope(name, f)
    }

    fn local_scope<T, F>(this: &mut Contextual<Self>, name: rst::DeclarationName, f: F) -> T
    where
        F: FnOnce(&mut Contextual<Self>) -> T,
    {
        let name = Self::make_scope_name(this, name);
        let before = this.data.locals.len();
        this.data.locals.push(Vec::new());

        let result = this.in_scope(name, f);

        this.data.locals.pop();
        assert_eq!(before, this.data.locals.len());

        result
    }

    fn make_scope_name(_this: &mut Contextual<Self>, name: rst::DeclarationName) -> Name {
        match name {
            rst::DeclarationName::Name(name) => name,
            rst::DeclarationName::Field(..) => todo!(),
            rst::DeclarationName::Invalid => todo!(),
        }
    }
}
