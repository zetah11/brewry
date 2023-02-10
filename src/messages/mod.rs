mod parse;
mod resolve;
mod types;

use crate::source::Span;
use crate::{Db, Messages};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    pub level: MessageLevel,

    pub code: Option<String>,
    pub message: Option<String>,
    pub labels: Vec<Label>,
}

impl Message {
    pub fn error() -> Self {
        Self {
            level: MessageLevel::Error,
            code: None,
            message: None,
            labels: Vec::new(),
        }
    }

    pub fn warning() -> Self {
        Self {
            level: MessageLevel::Warning,
            code: None,
            message: None,
            labels: Vec::new(),
        }
    }

    pub fn with_code(self, code: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            ..self
        }
    }

    pub fn with_message(self, message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            ..self
        }
    }

    pub fn with_labels(self, labels: impl IntoIterator<Item = Label>) -> Self {
        Self {
            labels: labels.into_iter().collect(),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MessageLevel {
    Error,
    Warning,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Label {
    pub kind: LabelKind,
    pub at: Span,
    pub message: Option<String>,
}

impl Label {
    pub fn primary(at: Span) -> Self {
        Self {
            kind: LabelKind::Primary,
            at,
            message: None,
        }
    }

    pub fn note(at: Span) -> Self {
        Self {
            kind: LabelKind::Note,
            at,
            message: None,
        }
    }

    pub fn help(at: Span) -> Self {
        Self {
            kind: LabelKind::Help,
            at,
            message: None,
        }
    }

    pub fn with_message(self, message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum LabelKind {
    Primary,
    Note,
    Help,
}

#[must_use]
pub struct MessageMaker<'a> {
    db: &'a dyn Db,
    span: Span,
}

impl<'a> MessageMaker<'a> {
    pub fn at(db: &'a dyn Db, span: Span) -> Self {
        Self { db, span }
    }

    fn add(&self, message: Message) {
        Messages::push(self.db, message);
    }
}
