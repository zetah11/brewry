use brewry::{parse, source, Messages};
use salsa::{Snapshot, Storage};

fn main() {
    let db = Database::default();
    let source = source::Source::new(&db, include_str!("../test.rry").into(), "main.rry".into());

    let tree = parse::parse(&db, source);
    for decl in tree.declarations(&db) {
        println!("{:?}", decl.node(&db));
    }

    println!("\n\nerrors!!!\n");

    for message in parse::parse::accumulated::<Messages>(&db, source) {
        println!("{message:?}");
    }
}

#[derive(Default)]
#[salsa::db(brewry::Jar)]
struct Database {
    storage: Storage<Self>,
}

impl salsa::Database for Database {}
impl salsa::ParallelDatabase for Database {
    fn snapshot(&self) -> Snapshot<Self> {
        Snapshot::new(Database {
            storage: self.storage.snapshot(),
        })
    }
}
