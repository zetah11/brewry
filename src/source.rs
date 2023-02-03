use std::ops::{Add, AddAssign};

#[salsa::input]
pub struct Source {
    #[return_ref]
    pub text: String,

    #[return_ref]
    pub name: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    pub source: Source,
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(source: Source, start: usize, end: usize) -> Self {
        Self { source, start, end }
    }
}

impl Add for Span {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        assert_eq!(self.source, rhs.source);
        Self {
            source: self.source,
            start: self.start.min(rhs.start),
            end: self.end.max(rhs.end),
        }
    }
}

impl AddAssign for Span {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
