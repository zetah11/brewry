//! Like the AST, but with most names (except fields) resolved.

use crate::names::{Name, NamePart};
use crate::source::Span;

#[salsa::tracked]
pub struct Items {
    #[return_ref]
    pub classes: Vec<Class>,

    #[return_ref]
    pub values: Vec<Value>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Class {
    pub name: DeclarationName,
    pub kind: ClassKind,
    pub items: Items,
    pub inherits: Vec<Type>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Value {
    pub name: DeclarationName,
    pub node: ValueNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ValueNode {
    Function {
        this: Option<usize>,
        args: Vec<(Name, Type)>,
        return_type: Type,
        body: Option<Block>,
    },

    Variable {
        anno: Type,
        body: Option<Expression>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ClassKind {
    Class,
    Variant,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationName {
    Name(Name),
    Field(Name, NamePart),
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    Name(Name),
    Field(Box<Type>, NamePart),

    Applied(Box<Type>, Vec<Type>),

    Function(Vec<Type>, Box<Type>),

    Reference(Box<Type>),

    Int,
    Nat,
    Boolean,
    Unit,

    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Block {
    pub declarations: Vec<(Name, Type)>,
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
    Assignment(Expression, Expression),
    Return(Expression),

    Null,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Expression {
    pub node: ExpressionNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ExpressionNode {
    Reference(Box<Expression>),

    Call(Box<Expression>, Vec<Expression>),
    Field(Box<Expression>, NamePart),

    Name(Name),
    Number(String),
    String(String),
    This,
    Unit,

    Invalid,
}
