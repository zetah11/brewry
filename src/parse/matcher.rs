use crate::token::Token;

pub trait Matcher {
    fn matches(&self, token: &Token) -> bool;
}

impl Matcher for Token {
    fn matches(&self, token: &Token) -> bool {
        match (self, token) {
            (Self::ValueName(_), Self::ValueName(_)) => true,
            (Self::TypeName(_), Self::TypeName(_)) => true,
            (Self::String(_), Self::String(_)) => true,
            (Self::Number(_), Self::Number(_)) => true,

            _ => self == token,
        }
    }
}

impl Matcher for &'_ [Token] {
    fn matches(&self, token: &Token) -> bool {
        self.iter().any(|this| this.matches(token))
    }
}
