use messages::Message;
use salsa::DbWithJar;

pub mod ast;
pub mod messages;
pub mod names;
pub mod parse;
pub mod resolve;
pub mod source;
pub mod token;

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::ast::Declaration,
    crate::ast::Declarations,
    crate::names::NamePart,
    crate::names::Name,
    crate::names::NamesWithin,
    crate::parse::parse,
    crate::resolve::names_within,
    crate::source::Source,
    crate::token::lex,
    crate::Messages,
);

pub trait Db: DbWithJar<Jar> {}

impl<D: ?Sized + DbWithJar<Jar>> Db for D {}

#[salsa::accumulator]
pub struct Messages(Message);
