mod traverse;

use std::collections::{HashMap, HashSet};

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
    let mut values = Vec::new();

    let tree = parse(db, source);
    let names = all_names_within(db, source);

    let mut resolver = Resolver::new(db, source, names);
    for item in tree.declarations(db) {
        if let ClassOrValue::Value(value) = Resolver::resolve_item(&mut resolver, item) {
            values.push(value);
        }
    }

    let tree = rst::Items::new(db, resolver.data.classes, values);
    NameInfo::new(db, resolver.data.mutable, tree)
}

enum ClassOrValue {
    Class(Name),
    Value(rst::Value),
}

struct Resolver {
    names: NamesWithin,
    mutable: HashSet<Name>,
    locals: Vec<Vec<Name>>,

    classes: HashMap<Name, rst::Class>,
}

impl Resolver {
    pub fn new(db: &dyn Db, source: Source, names: NamesWithin) -> Contextual<Self> {
        let this = Self {
            names,
            mutable: HashSet::new(),
            locals: Vec::new(),

            classes: HashMap::new(),
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

    /// Create a [`Name`] from the given [`rst::DeclarationName`]. This is used
    /// to unambiguously refer to scopes, even in items whose names are invalid
    /// or fields of other types (e.g. when overriding functions).
    fn make_scope_name(_this: &mut Contextual<Self>, name: rst::DeclarationName) -> Name {
        match name {
            rst::DeclarationName::Name(name) => name,
            rst::DeclarationName::Field(..) => todo!(),
            rst::DeclarationName::Invalid => todo!(),
        }
    }
}
