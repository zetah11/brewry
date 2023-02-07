#![allow(unused)]

use std::collections::HashMap;

use super::Type;
use crate::hir::{Class, Items, Value};
use crate::names::Name;
use crate::parse::parse;
use crate::source::Source;
use crate::{ast, Db};

#[salsa::tracked]
pub fn annotate(db: &dyn Db, source: Source) -> Items {
    let mut classes = Vec::new();
    let mut values = Vec::new();

    let tree = parse(db, source);

    let mut checker = Checker::new(db);
    for item in tree.declarations(db) {
        match checker.check(item) {
            ClassOrValue::Class(class) => classes.push(class),
            ClassOrValue::Value(value) => values.push(value),
        }
    }

    Items::new(db, classes, values)
}

#[derive(Debug)]
enum ClassOrValue {
    Class(Class),
    Value(Value),
}

struct Checker<'a> {
    db: &'a dyn Db,
    context: HashMap<Name, Type>,
}

impl<'a> Checker<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            db,
            context: HashMap::new(),
        }
    }

    pub fn check(&mut self, item: &ast::Declaration) -> ClassOrValue {
        todo!()
    }
}

enum ClassType {
    Class,
    Variant,
}
