use std::collections::{HashMap, HashSet};

use crate::inheritance::inherit_components;
use crate::messages::MessageMaker;
use crate::names::{Name, NamePart};
use crate::resolution::resolve_names;
use crate::rst::{self, Class, ClassKind, DeclarationName};
use crate::source::{Source, Span};
use crate::Db;

use super::subtyping::Subtypes;
use super::{Type, TypeNode};

#[salsa::tracked]
pub struct TypeInfo {
    #[return_ref]
    pub subtypes: Subtypes,

    #[return_ref]
    pub open: HashSet<Type>,

    #[return_ref]
    pub nested: HashMap<Name, HashMap<NamePart, Name>>,
}

#[salsa::tracked]
pub fn type_info(db: &dyn Db, source: Source) -> TypeInfo {
    let classes = resolve_names(db, source).tree(db).classes(db);
    let components = inherit_components(db, source);

    let mut collector = InfoCollector::new(db, classes);
    for component in components {
        collector.collect_component(component);
    }

    TypeInfo::new(db, collector.subtypes, collector.open, collector.nested)
}

struct InfoCollector<'a> {
    db: &'a dyn Db,
    classes: &'a HashMap<Name, Class>,
    subtypes: Subtypes,
    open: HashSet<Type>,
    nested: HashMap<Name, HashMap<NamePart, Name>>,
}

impl<'a> InfoCollector<'a> {
    pub fn new(db: &'a dyn Db, classes: &'a HashMap<Name, Class>) -> Self {
        Self {
            db,
            classes,
            subtypes: Subtypes::new(),
            open: HashSet::new(),
            nested: HashMap::new(),
        }
    }

    pub fn collect_component(&mut self, names: &HashSet<Name>) {
        // Declare all the nested types
        for name in names {
            self.declare_nested(*name);
        }

        // Do the subtyping
        for name in names {
            self.declare_subtyping(*name);
        }
    }

    fn declare_nested(&mut self, name: Name) {
        let class = self.classes.get(&name).expect("not a class name!");

        if let ClassKind::Class = class.kind {
            let class = Type::new(self.db, TypeNode::Name(name));
            self.open.insert(class);
        }

        let mut nested = HashMap::new();
        for global in class.fields.classes.iter() {
            let class = self
                .classes
                .get(global)
                .expect("mention of non-class in nested classes");

            let name = match class.name {
                DeclarationName::Name(name) => name.name(self.db),

                // hmmm
                DeclarationName::Field(..) => continue,
                DeclarationName::Invalid => continue,
            };

            assert!(nested.insert(name, *global).is_none());
        }

        assert!(self.nested.insert(name, nested).is_none());
    }

    fn declare_subtyping(&mut self, name: Name) {
        let class = self.classes.get(&name).expect("not a class name!");

        if let ClassKind::Variant = class.kind {
            // implicit subtyping galore
            todo!()
        }

        let this_type = Type::new(self.db, TypeNode::Name(name));
        for inherit in class.inherits.iter() {
            let inherit = self.to_type(inherit);

            if let Some(path) = self.subtypes.supertype_path(&this_type, &inherit) {
                self.at(class.span).types_subtype_cycle(Some(path));
            } else {
                self.subtypes.add_subtype(inherit, this_type);
            }
        }
    }

    fn to_type(&self, ty: &rst::Type) -> Type {
        let node = match &ty.node {
            rst::TypeNode::Name(name) => TypeNode::Name(*name),
            rst::TypeNode::Field(of, field) => {
                let of = self.to_type(of);
                if let TypeNode::Name(name) = of.node(self.db) {
                    self.nested
                        .get(&name)
                        .expect("todo: ordering issues")
                        .get(field)
                        .map(|name| TypeNode::Name(*name))
                        .unwrap_or_else(|| {
                            self.at(ty.span).resolve_unresolved_name();
                            TypeNode::Bottom
                        })
                } else {
                    todo!("hmmmm");
                }
            }

            rst::TypeNode::Applied(..) => todo!(),

            rst::TypeNode::Function(from, to) => {
                let from = from.iter().map(|ty| self.to_type(ty)).collect();
                let to = self.to_type(to);
                TypeNode::Function(from, to)
            }

            rst::TypeNode::Reference(of) => {
                let of = self.to_type(of);
                TypeNode::Reference(of)
            }

            rst::TypeNode::Int => TypeNode::Int,
            rst::TypeNode::Nat => TypeNode::Nat,
            rst::TypeNode::Boolean => TypeNode::Boolean,
            rst::TypeNode::Unit => TypeNode::Unit,
            rst::TypeNode::Invalid => TypeNode::Bottom,
        };

        Type::new(self.db, node)
    }

    fn at(&self, span: Span) -> MessageMaker<'a> {
        MessageMaker::at(self.db, span)
    }
}
