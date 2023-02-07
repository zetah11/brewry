use crate::names::{Name, NameNode, NamePart, NamePrefix};
use crate::source::Source;
use crate::Db;

use super::subtyping::Subtypes;
use super::{Type, TypeNode};

#[derive(Default)]
#[salsa::db(crate::Jar)]
struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}

fn make_name(db: &dyn Db, name: impl Into<String>) -> Type {
    let source = Source::new(db, String::new(), String::new());
    let scope = NamePrefix::Source(source);

    let name = Name::new(db, scope, NamePart::new(db, NameNode::Type(name.into())));
    Type::new(db, TypeNode::Name(name))
}

fn test_subtypes(
    subtypes: &Subtypes,
    holds: impl IntoIterator<Item = (Type, Type)>,
    doesnt: impl IntoIterator<Item = (Type, Type)>,
) {
    for (a, b) in holds {
        assert!(subtypes.is_subtype(&a, &b));
    }

    for (a, b) in doesnt {
        assert!(!subtypes.is_subtype(&a, &b));
    }

    subtypes.assert_integrity();
}

#[test]
fn subtype_reflexivity() {
    let db = Database::default();
    let subtypes = Subtypes::new();

    let a = make_name(&db, "A");

    let holds = [(a, a)];
    let doesnt = [];

    test_subtypes(&subtypes, holds, doesnt);
}

#[test]
fn subtype_transitivity() {
    let db = Database::default();
    let mut subtypes = Subtypes::new();

    let a = make_name(&db, "A");
    let b = make_name(&db, "B");
    let c = make_name(&db, "C");
    let d = make_name(&db, "D");

    subtypes.add_subtype(b, a);
    subtypes.add_subtype(c, b);
    subtypes.add_subtype(d, c);

    let holds = [(a, c), (a, d), (b, d)];
    let doesnt = [(d, a), (d, b), (d, c), (c, a), (c, b), (b, a)];

    test_subtypes(&subtypes, holds, doesnt);
}

#[test]
fn subtype_direct_subtypes() {
    let db = Database::default();
    let mut subtypes = Subtypes::new();

    let a = make_name(&db, "A");
    let b = make_name(&db, "B");
    let c = make_name(&db, "C");
    let d = make_name(&db, "D");

    subtypes.add_subtype(b, a);
    subtypes.add_subtype(d, c);

    let holds = [(a, b), (c, d)];
    let doesnt = [
        (a, d),
        (a, c),
        (b, a),
        (b, c),
        (b, d),
        (c, a),
        (c, b),
        (d, a),
        (d, b),
        (d, c),
    ];

    test_subtypes(&subtypes, holds, doesnt);
}

#[test]
fn subtype_lattice() {
    //         T
    //        / \
    //       A   B
    //      / \ / \
    //     C   D   E
    //      \  |  /
    //         F

    let db = Database::default();
    let mut subtypes = Subtypes::new();

    let a = make_name(&db, "A");
    let b = make_name(&db, "B");
    let c = make_name(&db, "C");
    let d = make_name(&db, "D");
    let e = make_name(&db, "E");
    let t = make_name(&db, "T");
    let f = make_name(&db, "F");

    subtypes.add_subtype(t, a);
    subtypes.add_subtype(t, b);

    subtypes.add_subtype(a, c);
    subtypes.add_subtype(a, d);
    subtypes.add_subtype(b, d);
    subtypes.add_subtype(b, e);

    subtypes.add_subtype(c, f);
    subtypes.add_subtype(d, f);
    subtypes.add_subtype(e, f);

    let holds = [a, b, c, d, e, t].into_iter().map(|of| (f, of));
    test_subtypes(&subtypes, holds, []);

    let holds = [a, b, c, d, e, f].into_iter().map(|this| (this, t));
    test_subtypes(&subtypes, holds, []);

    let some_unrelated = [
        (a, b),
        (b, a),
        (c, d),
        (d, c),
        (c, e),
        (e, c),
        (b, c),
        (c, b),
        (a, e),
        (e, a),
    ];

    test_subtypes(&subtypes, [], some_unrelated);
}

#[test]
#[should_panic]
fn subtype_cycle() {
    let db = Database::default();
    let mut subtypes = Subtypes::new();

    let a = make_name(&db, "A");
    let b = make_name(&db, "b");
    let c = make_name(&db, "C");

    subtypes.add_subtype(a, b);
    subtypes.add_subtype(b, c);
    subtypes.add_subtype(c, a);
}
