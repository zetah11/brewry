use std::fs::File;

use brewry::{inheritance, source, types, Messages};
use salsa::{Snapshot, Storage};

fn main() {
    let db = Database::default();
    let source = source::Source::new(&db, include_str!("../test.rry").into(), "main.rry".into());

    let info = types::type_info(&db, source);
    let subtypes = info.subtypes(&db);

    let viz = types::SubtypeVisualizer::new(&db, subtypes);
    let mut output = File::create("subtypes.dot").unwrap();
    dot::render(&viz, &mut output).unwrap();

    println!("\n\nerrors!!!\n");

    for message in inheritance::all_mentions::accumulated::<Messages>(&db, source) {
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
