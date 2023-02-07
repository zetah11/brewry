use crate::source::Source;
use crate::types::Type;

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
    Local(Box<NamePrefix>, usize),
    Item(Name),
    Type(Type),
    Source(Source),
}

#[salsa::interned]
pub struct Name {
    pub scope: NamePrefix,
    pub name: NamePart,
}
