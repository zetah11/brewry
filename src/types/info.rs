use std::collections::HashSet;

use super::subtyping::Subtypes;
use super::Type;

#[salsa::tracked]
pub struct TypeInfo {
    #[return_ref]
    pub subtypes: Subtypes,

    #[return_ref]
    pub open: HashSet<Type>,
}
