use std::collections::{HashMap, HashSet};

use crate::names::Name;
use crate::resolution::resolve_names;
use crate::rst::{Class, ClassKind, Type, TypeNode};
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

    let mut mentioner = MentionLocator::new();
    for (name, item) in items.classes(db) {
        mentioner.class_mentions(name, item);
    }

    Mentions::new(db, mentioner.inherits)
}

#[derive(Debug, Default)]
struct MentionLocator {
    inherits: HashMap<Name, HashSet<Name>>,
}

impl MentionLocator {
    pub fn new() -> Self {
        Self {
            inherits: HashMap::new(),
        }
    }

    pub fn class_mentions(&mut self, name: &Name, item: &Class) {
        let mut set = HashSet::new();
        for ty in &item.inherits {
            Self::type_mentions(&mut set, ty);
        }

        // Nested items also inherit from outer variants
        if let ClassKind::Variant = item.kind {
            for nested in item.fields.classes.iter() {
                self.inherits.entry(*nested).or_default().insert(*name);
            }
        }

        if let Some(inherits) = self.inherits.get_mut(name) {
            inherits.extend(set);
        } else {
            self.inherits.insert(*name, set);
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
}
