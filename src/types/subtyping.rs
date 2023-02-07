use std::collections::{HashMap, HashSet};

use itertools::Itertools;

use super::Type;

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
