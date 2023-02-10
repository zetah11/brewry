mod check;
mod info;
mod subtyping;

pub use check::annotate;
pub use info::{type_info, TypeInfo};
use itertools::Itertools;
pub use subtyping::{SubtypeVisualizer, Subtypes};

use crate::names::{Name, NameNode};
use crate::Db;

#[cfg(test)]
mod tests;

#[salsa::interned]
pub struct Type {
    pub node: TypeNode,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Bottom,

    Unit,
    Int,
    Nat,
    Boolean,
    Name(Name),

    Function(Vec<Type>, Type),
    Reference(Type),
}

pub fn pretty_type(db: &dyn Db, ty: &Type) -> String {
    match ty.node(db) {
        TypeNode::Bottom => "!".to_string(),
        TypeNode::Unit => "Unit".to_string(),
        TypeNode::Int => "Int".to_string(),
        TypeNode::Nat => "Nat".to_string(),
        TypeNode::Boolean => "Boolean".to_string(),
        TypeNode::Name(name) => match name.name(db).node(db) {
            NameNode::Invalid => "<error>".to_string(),
            NameNode::Type(ty) => ty.clone(),
            NameNode::Value(ty) => ty.clone(),
        },

        TypeNode::Function(from, to) => {
            format!(
                "({}) {}",
                from.iter().map(|ty| pretty_type(db, ty)).join(", "),
                pretty_type(db, &to)
            )
        }

        TypeNode::Reference(of) => {
            format!("{}&", pretty_type(db, &of))
        }
    }
}
