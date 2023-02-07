mod resolve;
mod within;

pub use resolve::{resolve_names, NameInfo};
pub use within::{all_names_within, NamesWithin};

use crate::ast::{DeclarationName, DeclarationNameNode};
use crate::messages::MessageMaker;
use crate::names::{Name, NameNode, NamePart, NamePrefix};
use crate::source::{Source, Span};
use crate::Db;

/// Helper struct for traversing scopes properly.
struct Contextual<'a, Data> {
    db: &'a dyn Db,
    within: (Source, Vec<Name>),
    data: Data,
}

impl<'a, Data> Contextual<'a, Data> {
    pub fn new(db: &'a dyn Db, data: Data, source: Source) -> Self {
        Self {
            db,
            within: (source, Vec::new()),
            data,
        }
    }

    pub fn at(&self, span: Span) -> MessageMaker<'a> {
        MessageMaker::at(self.db, span)
    }

    pub fn prefix(&self) -> NamePrefix {
        if let Some(item) = self.within.1.last() {
            NamePrefix::Item(*item)
        } else {
            NamePrefix::Source(self.within.0)
        }
    }

    pub fn in_scope<T, F>(&mut self, name: Name, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let before = self.within.1.len();
        self.within.1.push(name);

        let result = f(self);

        self.within.1.pop();
        assert_eq!(before, self.within.1.len());

        result
    }

    fn declaration_name(&mut self, name: &DeclarationName) -> Name {
        match name.node {
            DeclarationNameNode::Identifier(name) => self.declare_name(name),
            DeclarationNameNode::Quoted(_) => todo!(),
            DeclarationNameNode::Invalid => {
                self.declare_name(NamePart::new(self.db, NameNode::Invalid))
            }
        }
    }

    fn declare_name(&mut self, name: NamePart) -> Name {
        Name::new(self.db, self.prefix(), name)
    }
}
