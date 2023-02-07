mod check;
mod info;
mod subtyping;

pub use check::annotate;
pub use info::TypeInfo;
pub use subtyping::Subtypes;

use crate::names::Name;

#[cfg(test)]
mod tests;

#[salsa::interned]
pub struct Type {
    pub node: TypeNode,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Bottom,

    Unit,
    Int,
    Nat,
    Boolean,
    Name(Name),

    Function(Type),
    Reference(Type),
}
