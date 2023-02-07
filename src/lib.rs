use messages::Message;
use salsa::DbWithJar;

pub mod ast;
pub mod hir;
pub mod messages;
pub mod names;
pub mod parse;
pub mod resolution;
pub mod rst;
pub mod source;
pub mod token;
pub mod types;

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::ast::Declaration,
    crate::ast::Declarations,
    crate::hir::Items,
    crate::names::NamePart,
    crate::names::Name,
    crate::parse::parse,
    crate::resolution::NamesWithin,
    crate::resolution::all_names_within,
    crate::resolution::NameInfo,
    crate::resolution::resolve_names,
    crate::rst::Items,
    crate::source::Source,
    crate::token::lex,
    crate::types::annotate,
    crate::types::Type,
    crate::types::TypeInfo,
    crate::Messages,
);

pub trait Db: DbWithJar<Jar> {}

impl<D: ?Sized + DbWithJar<Jar>> Db for D {}

#[salsa::accumulator]
pub struct Messages(Message);
