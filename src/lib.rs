use messages::Message;
use salsa::DbWithJar;

pub mod ast;
pub mod messages;
pub mod parse;
pub mod source;
pub mod token;

#[salsa::jar(db = Db)]
pub struct Jar(
    crate::ast::Declaration,
    crate::ast::Declarations,
    crate::ast::Name,
    crate::parse::parse,
    crate::source::Source,
    crate::token::lex,
    crate::Messages,
);

pub trait Db: DbWithJar<Jar> {}

impl<D: ?Sized + DbWithJar<Jar>> Db for D {}

#[salsa::accumulator]
pub struct Messages(Message);
