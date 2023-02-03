use super::{Label, Message, MessageMaker};

const EXPECTED_DECLARATION: &str = "EP00";
const EXPECTED_EXPRESSION: &str = "EP01";
const EXPECTED_TYPE: &str = "EP02";
const EXPECTED_TYPE_NAME: &str = "EP10";
const EXPECTED_VALUE_NAME: &str = "EP11";
const EXPECTED_ASSIGNMENT: &str = "EP12";
const MISSING_END: &str = "EP20";
const MISSING_PAREN: &str = "EP21";

impl MessageMaker<'_> {
    pub fn parse_expected_declaration(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(EXPECTED_DECLARATION)
                .with_message("expected a declaration")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_expression(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(EXPECTED_EXPRESSION)
                .with_message("expected an expression")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_type(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(EXPECTED_TYPE)
                .with_message("expected a type")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_value_name(&self, type_name: Option<&str>) {
        let mut labels = vec![Label::primary(self.span)];

        if let Some(type_name) = type_name {
            labels.push(Label::help(self.span).with_message(format!(
                "value names must begin with a lowercase letter: '{}'",
                make_value_case(type_name)
            )));
        }

        self.add(
            Message::error()
                .with_code(EXPECTED_VALUE_NAME)
                .with_message("expected a value name")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_type_name(&self, value_name: Option<&str>) {
        let mut labels = vec![Label::primary(self.span)];

        if let Some(value_name) = value_name {
            labels.push(Label::help(self.span).with_message(format!(
                "type names must begin with an uppercase letter: '{}'",
                make_type_case(value_name)
            )));
        }

        self.add(
            Message::error()
                .with_code(EXPECTED_TYPE_NAME)
                .with_message("expected a value name")
                .with_labels(labels),
        );
    }

    pub fn parse_expected_assignment(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(EXPECTED_ASSIGNMENT)
                .with_message("expected a value assignment")
                .with_labels(labels),
        );
    }

    pub fn parse_missing_end(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(MISSING_END)
                .with_message("missing an 'end' keyword")
                .with_labels(labels),
        );
    }

    pub fn parse_missing_paren(&self) {
        let labels = vec![Label::primary(self.span)];

        self.add(
            Message::error()
                .with_code(MISSING_PAREN)
                .with_message("unclosed opening parenthesis")
                .with_labels(labels),
        );
    }
}

fn make_type_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());

    let mut chars = name.chars();
    if let Some(initial) = chars.next() {
        result.push(initial.to_ascii_uppercase());
    }

    result.extend(chars);
    result
}

fn make_value_case(name: &str) -> String {
    let mut result = String::with_capacity(name.len());

    let mut chars = name.chars();
    if let Some(initial) = chars.next() {
        result.push(initial.to_ascii_lowercase());
    }

    result.extend(chars);
    result
}
