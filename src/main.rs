use brewry::names::NameNode;
use brewry::{inheritance, source, Messages};
use salsa::{Snapshot, Storage};

fn main() {
    let db = Database::default();
    let source = source::Source::new(&db, include_str!("../test.rry").into(), "main.rry".into());

    let mentions = inheritance::inherit_components(&db, source);
    println!("{mentions:?}");

    for component in mentions {
        for name in component {
            let name = name.name(&db);
            let name = name.node(&db);

            print!(
                "{} ",
                match name {
                    NameNode::Invalid => "<invalid>",
                    NameNode::Type(name) => name.as_str(),
                    NameNode::Value(name) => name.as_str(),
                }
            );
        }

        println!();
    }

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
