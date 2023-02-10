use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use crate::Db;

use super::{pretty_type, Type};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Subtypes {
    supers: HashMap<Type, HashSet<Type>>,
    subs: HashMap<Type, HashSet<Type>>,
}

impl Subtypes {
    pub fn new() -> Self {
        Self {
            supers: HashMap::new(),
            subs: HashMap::new(),
        }
    }

    /// Add a subtyping relation between `parent` and `sub`.
    pub fn add_subtype(&mut self, parent: Type, sub: Type) {
        self.supers.entry(sub).or_default().insert(parent);
        self.subs.entry(parent).or_default().insert(sub);

        #[cfg(debug_assertions)]
        self.assert_integrity();
    }

    /// Returns true if `this` is a subtype of `of`, or if `this == of`. Linear
    /// time complexity over the number of supertypes.
    pub fn is_subtype(&self, this: &Type, of: &Type) -> bool {
        if this == of {
            return true;
        }

        let Some(supers) = self.supers.get(this) else { return false; };

        // Do a depth first search to see if this is a direct subtype of if any
        // supertype is a parent.
        supers.contains(of) || supers.iter().any(|parent| self.is_subtype(parent, of))
    }

    pub fn supertype_path(&self, this: &Type, of: &Type) -> Option<Vec<Type>> {
        if this == of {
            return Some(vec![*this]);
        }

        let supers = self.supers.get(this)?;
        supers
            .iter()
            .find_map(|sup| self.supertype_path(sup, of))
            .map(|mut path| {
                path.push(*this);
                path
            })
    }

    /// Returns an iterator over every supertype of the given type. The first
    /// item returned is always the type itself.
    pub fn supertypes<'a>(&'a self, of: &Type) -> Box<dyn Iterator<Item = Type> + 'a> {
        let supers = self
            .supers
            .get(of)
            .into_iter()
            .flatten()
            .flat_map(|ty| self.supertypes(ty));

        Box::new(std::iter::once(*of).chain(supers))
    }

    /// Assert that there are no cycles (e.g. `A < B` and `B < A`). This is
    /// expensive (cubic in the number of types), and so is disabled in release
    /// builds.
    pub fn assert_integrity(&self) {
        for (a, b) in self.supers.keys().tuple_combinations() {
            assert!(!(self.is_subtype(a, b) && self.is_subtype(b, a)));
        }
    }
}

pub struct SubtypeVisualizer<'a> {
    db: &'a dyn Db,
    subtypes: &'a Subtypes,
}

impl<'a> SubtypeVisualizer<'a> {
    pub fn new(db: &'a dyn Db, subtypes: &'a Subtypes) -> Self {
        Self { db, subtypes }
    }
}

impl<'a> dot::Labeller<'a, Type, (Type, Type)> for SubtypeVisualizer<'a> {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("subtypes").unwrap()
    }

    fn node_id(&'a self, n: &Type) -> dot::Id<'a> {
        dot::Id::new(format!("t{}", n.0.as_u32())).unwrap()
    }

    fn node_label(&'a self, n: &Type) -> dot::LabelText<'a> {
        dot::LabelText::label(pretty_type(self.db, n))
    }
}

impl<'a> dot::GraphWalk<'a, Type, (Type, Type)> for SubtypeVisualizer<'a> {
    fn nodes(&'a self) -> dot::Nodes<'a, Type> {
        let mut types = HashSet::new();
        for (k, vs) in self.subtypes.supers.iter() {
            types.insert(*k);

            for v in vs.iter() {
                types.insert(*v);
            }
        }

        Cow::Owned(types.into_iter().collect())
    }

    fn edges(&'a self) -> dot::Edges<'a, (Type, Type)> {
        let mut edges = HashSet::new();
        for (k, vs) in self.subtypes.supers.iter() {
            for v in vs.iter() {
                // Supertypes point towards their subtypes
                edges.insert((*v, *k));
            }
        }

        Cow::Owned(edges.into_iter().collect())
    }

    fn source(&'a self, edge: &(Type, Type)) -> Type {
        edge.0
    }

    fn target(&'a self, edge: &(Type, Type)) -> Type {
        edge.1
    }
}
