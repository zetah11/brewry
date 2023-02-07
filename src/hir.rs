use crate::names::Name;
use crate::source::Span;
use crate::types::Type;

#[salsa::tracked]
pub struct Items {
    pub classes: Vec<Class>,
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Class {
    pub name: Name,
    pub items: Items,

    /// Can this class be subclasses by external classes?
    pub open: bool,

    /// Do other classes declared within this one automatically inherit from
    /// this one?
    pub autoinherit: bool,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Value {
    pub name: Name,
    pub node: ValueNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueNode {
    Function {
        /// `None` if the function does not take a `this` argument; `Some(n)` if
        /// it does, where `n` is the number of references it is behind.
        this: Option<usize>,
        args: Vec<Name>,
        body: Option<Block>,
    },

    Variable {
        body: Option<Expression>,
    },
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Block {
    pub declared: Vec<Name>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Statement {
    pub node: StatementNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatementNode {
    Expression(Expression),
    Assignment(Name, Expression),
    Return(Expression),
    Null,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub span: Span,
    pub anno: Type,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExpressionNode {
    Reference(Box<Expression>),

    Call(Box<Expression>, Vec<Expression>),
    Field(Box<Expression>, Name),

    Name(Name),
    Number(String),
    String(String),
    This,
    Unit,

    Invalid,
}
