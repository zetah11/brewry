use std::collections::{HashMap, HashSet};

use super::resolve_names;
use crate::names::Name;
use crate::rst::{Class, DeclarationName, Type, TypeNode};
use crate::source::Source;
use crate::Db;

#[salsa::tracked]
pub struct Mentions {
    /// Any (non-field) names mentioned in the inheritance clause of a class or
    /// variant.
    #[return_ref]
    pub inherits: HashMap<Name, HashSet<Name>>,
}

#[salsa::tracked]
pub fn all_mentions(db: &dyn Db, source: Source) -> Mentions {
    let tree = resolve_names(db, source);
    let items = tree.tree(db);

    let mut mentioner = MentionLocator::new(db);
    for item in items.classes(db) {
        mentioner.class_mentions(item);
    }

    Mentions::new(db, mentioner.inherits)
}

struct MentionLocator<'a> {
    db: &'a dyn Db,
    inherits: HashMap<Name, HashSet<Name>>,
}

impl<'a> MentionLocator<'a> {
    pub fn new(db: &'a dyn Db) -> Self {
        Self {
            db,
            inherits: HashMap::new(),
        }
    }

    pub fn class_mentions(&mut self, item: &Class) {
        if let Some(name) = self.declaration_name(&item.name) {
            let mut set = HashSet::new();
            for ty in &item.inherits {
                Self::type_mentions(&mut set, ty);
            }

            assert!(self.inherits.insert(name, set).is_none());
        }

        for item in item.items.classes(self.db) {
            self.class_mentions(item);
        }
    }

    fn type_mentions(into: &mut HashSet<Name>, ty: &Type) {
        match &ty.node {
            TypeNode::Name(name) => {
                into.insert(*name);
            }

            TypeNode::Field(of, _) => {
                Self::type_mentions(into, of);
            }

            TypeNode::Applied(to, args) => {
                Self::type_mentions(into, to);
                args.iter().for_each(|ty| Self::type_mentions(into, ty));
            }

            TypeNode::Function(args, to) => {
                args.iter().for_each(|ty| Self::type_mentions(into, ty));
                Self::type_mentions(into, to);
            }

            TypeNode::Reference(ty) => {
                Self::type_mentions(into, ty);
            }

            TypeNode::Int
            | TypeNode::Nat
            | TypeNode::Boolean
            | TypeNode::Unit
            | TypeNode::Invalid => {}
        }
    }

    fn declaration_name(&self, name: &DeclarationName) -> Option<Name> {
        match name {
            DeclarationName::Name(name) => Some(*name),
            DeclarationName::Field(..) => todo!(),
            DeclarationName::Invalid => None,
        }
    }
}
