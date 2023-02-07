use std::collections::HashSet;

use super::all_mentions;
use crate::names::Name;
use crate::source::Source;
use crate::{components, Db};

/// Find every strongly connected component in the graph of which types directly
/// refer to each other in their inheritance clause
#[salsa::tracked(return_ref)]
pub fn inherit_components(db: &dyn Db, source: Source) -> Vec<HashSet<Name>> {
    let mentions = all_mentions(db, source).inherits(db);
    components::find(mentions)
}
