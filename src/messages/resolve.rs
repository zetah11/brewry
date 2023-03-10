use crate::source::Span;

use super::{Label, Message, MessageMaker};

const DUPLICATE_DEFINITIONS: &str = "ER00";
const UNRESOLVED_NAME: &str = "ER01";

impl MessageMaker<'_> {
    pub fn resolve_duplicate_definitions(&self, other: Span) {
        let labels = vec![
            Label::primary(self.span).with_message("duplicate definition here"),
            Label::note(other).with_message("first defined here"),
        ];

        self.add(
            Message::error()
                .with_code(DUPLICATE_DEFINITIONS)
                .with_message("duplicate definitions")
                .with_labels(labels),
        );
    }

    pub fn resolve_unresolved_name(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(UNRESOLVED_NAME)
                .with_message("unresolved name")
                .with_labels(labels),
        )
    }
}
