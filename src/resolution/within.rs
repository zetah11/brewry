//! This module maps out what names are inside which items.

use std::collections::{HashMap, HashSet};

use crate::ast::{Declaration, DeclarationNode};
use crate::names::Name;
use crate::parse::parse;
use crate::source::{Source, Span};
use crate::Db;

use super::Contextual;

#[salsa::tracked]
pub struct NamesWithin {
    #[return_ref]
    pub names: HashMap<Name, HashSet<Name>>,

    #[return_ref]
    pub spans: HashMap<Name, Span>,

    #[return_ref]
    pub public: HashSet<Name>,
}

#[salsa::tracked]
pub fn all_names_within(db: &dyn Db, source: Source) -> NamesWithin {
    let declarations = parse(db, source);
    let mut declarer = Declarer::new(db, source);

    for item in declarations.declarations(db).iter().copied() {
        let _ = Declarer::declare(&mut declarer, item);
    }

    let (names, spans) = declarer
        .data
        .names
        .into_iter()
        .map(|(name, (span, names))| ((name, names), (name, span)))
        .unzip();

    NamesWithin::new(db, names, spans, declarer.data.public)
}

struct Declarer {
    names: HashMap<Name, (Span, HashSet<Name>)>,
    public: HashSet<Name>,
}

impl Declarer {
    pub fn new(db: &dyn Db, source: Source) -> Contextual<Self> {
        let this = Self {
            names: HashMap::new(),
            public: HashSet::new(),
        };

        Contextual::new(db, this, source)
    }

    #[must_use]
    fn declare(this: &mut Contextual<Self>, item: Declaration) -> Name {
        let name = item.name(this.db);
        let span = item.span(this.db);
        let name = this.declaration_name(name);

        if let Some((other, _)) = this.data.names.get(&name) {
            this.at(span).resolve_duplicate_definitions(*other);
        }

        this.data.names.insert(name, (span, HashSet::new()));

        match item.node(this.db) {
            DeclarationNode::Class {
                public, private, ..
            }
            | DeclarationNode::Variant {
                public, private, ..
            } => {
                this.in_scope(name, |this| {
                    for item in public {
                        let child = Self::declare(this, *item);
                        this.data.public.insert(child);
                        Self::add_child(this, name, child);
                    }

                    for item in private {
                        let child = Self::declare(this, *item);
                        Self::add_child(this, name, child);
                    }
                });
            }

            DeclarationNode::Variable { .. } | DeclarationNode::Function { .. } => {}
        }

        name
    }

    fn add_child(this: &mut Contextual<Self>, parent: Name, child: Name) {
        this.data
            .names
            .get_mut(&parent)
            .expect("should be set elsewhere to protect against duplicates")
            .1
            .insert(child);
    }
}
