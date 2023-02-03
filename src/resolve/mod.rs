use std::collections::{HashMap, HashSet};

use crate::ast::{Declaration, DeclarationName, DeclarationNameNode, DeclarationNode};
use crate::messages::MessageMaker;
use crate::names::{Name, NameNode, NamePart, NamePrefix, NamesWithin};
use crate::parse::parse;
use crate::source::{Source, Span};
use crate::Db;

#[salsa::tracked]
pub fn names_within(db: &dyn Db, source: Source) -> NamesWithin {
    let declarations = parse(db, source);
    let mut declarer = Declarer::new(db, source);

    for item in declarations.declarations(db).iter().copied() {
        let _ = declarer.declare(item);
    }

    let (names, spans) = declarer
        .names
        .into_iter()
        .map(|(name, (span, names))| ((name, names), (name, span)))
        .unzip();

    NamesWithin::new(db, names, spans, declarer.public)
}

struct Declarer<'a> {
    db: &'a dyn Db,
    within: (Source, Vec<Name>),

    names: HashMap<Name, (Span, HashSet<Name>)>,
    public: HashSet<Name>,
}

impl<'a> Declarer<'a> {
    pub fn new(db: &'a dyn Db, source: Source) -> Self {
        Self {
            db,
            within: (source, Vec::new()),

            names: HashMap::new(),
            public: HashSet::new(),
        }
    }

    #[must_use]
    fn declare(&mut self, item: Declaration) -> Option<Name> {
        match item.node(self.db) {
            DeclarationNode::Class {
                name,
                public,
                private,
                ..
            }
            | DeclarationNode::Variant {
                name,
                public,
                private,
                ..
            } => {
                let name = self.declaration_name(name)?;

                self.in_scope(name, |this| {
                    for item in public {
                        let Some(name) = this.declare(*item) else { continue; };
                        this.public.insert(name);
                        this.add_child(name);
                    }

                    for item in private {
                        let Some(name) = this.declare(*item) else { continue; };
                        this.add_child(name);
                    }
                });

                Some(name)
            }

            DeclarationNode::Variable { name, .. } => self.declaration_name(name),

            DeclarationNode::Function { name, .. } => self.declaration_name(name),
        }
    }

    fn in_scope<T, F>(&mut self, name: Name, f: F) -> T
    where
        F: FnOnce(&mut Declarer<'a>) -> T,
    {
        let before = self.within.1.len();
        self.within.1.push(name);

        let result = f(self);

        self.within.1.pop();
        assert_eq!(before, self.within.1.len());

        result
    }

    fn add_child(&mut self, name: Name) {
        if let Some(parent) = self.within.1.last() {
            self.names
                .get_mut(parent)
                .expect("should be set elsewhere to protect against duplicates")
                .1
                .insert(name);
        }
    }

    fn at(&self, span: Span) -> MessageMaker<'a> {
        MessageMaker::at(self.db, span)
    }

    fn declaration_name(&mut self, name: &DeclarationName) -> Option<Name> {
        if name.prefix.is_some() {
            return None;
        }

        let span = name.span;
        let name = match name.node {
            DeclarationNameNode::Identifier(name) => self.declare_name(name),

            DeclarationNameNode::Quoted(_) => todo!(),

            DeclarationNameNode::Invalid => {
                self.declare_name(NamePart::new(self.db, NameNode::Invalid))
            }
        };

        if let Some((other, _)) = self.names.get(&name) {
            self.at(span).resolve_duplicate_definitions(*other);
        }

        self.names.insert(name, (span, HashSet::new()));

        Some(name)
    }

    fn declare_name(&mut self, name: NamePart) -> Name {
        let scope = if let Some(name) = self.within.1.last() {
            NamePrefix::Item(*name)
        } else {
            NamePrefix::Source(self.within.0)
        };

        Name::new(self.db, scope, name)
    }
}
