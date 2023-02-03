use std::collections::{HashMap, HashSet};

use crate::source::{Source, Span};

#[salsa::interned]
pub struct NamePart {
    #[return_ref]
    pub node: NameNode,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NameNode {
    Type(String),
    Value(String),
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum NamePrefix {
    Item(Name),
    Source(Source),
}

#[salsa::interned]
pub struct Name {
    pub scope: NamePrefix,
    pub name: NamePart,
}

#[salsa::tracked]
pub struct NamesWithin {
    #[return_ref]
    pub names: HashMap<Name, HashSet<Name>>,

    #[return_ref]
    pub spans: HashMap<Name, Span>,

    #[return_ref]
    pub public: HashSet<Name>,
}
