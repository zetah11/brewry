use itertools::Itertools;

use super::{Label, Message, MessageMaker};
use crate::types::{pretty_type, Type};

const SUBTYPE_CYCLE: &str = "ET00";

impl MessageMaker<'_> {
    pub fn types_subtype_cycle(&self, involves: Option<Vec<Type>>) {
        let involves = involves.map(|involves| {
            involves
                .iter()
                .map(|ty| pretty_type(self.db, ty))
                .join(", ")
        });

        let labels = if let Some(involves) = involves {
            vec![Label::primary(self.span).with_message(format!(
                "this type ends up being its own subtype through {}",
                involves
            ))]
        } else {
            vec![Label::primary(self.span).with_message("this type ends up being its own subtype")]
        };

        self.add(
            Message::error()
                .with_code(SUBTYPE_CYCLE)
                .with_message("subtyping cycle")
                .with_labels(labels),
        );
    }
}
