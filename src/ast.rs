use crate::names::NamePart;
use crate::source::Span;

#[salsa::tracked]
pub struct Declarations {
    #[return_ref]
    pub declarations: Vec<Declaration>,
}

#[salsa::tracked]
pub struct Declaration {
    #[return_ref]
    pub node: DeclarationNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationNode {
    Class {
        name: DeclarationName,
        public: Vec<Declaration>,
        private: Vec<Declaration>,
        inherits: Vec<Type>,
    },

    Variant {
        name: DeclarationName,
        public: Vec<Declaration>,
        private: Vec<Declaration>,
        inherits: Vec<Type>,
    },

    Function {
        name: DeclarationName,
        args: Vec<(NamePart, Type)>,
        return_type: Type,
        body: Option<Block>,
    },

    Variable {
        name: DeclarationName,
        anno: Type,
        body: Option<Expression>,
    },
}

/// A declaration name is possibly a prefix (the name of the inherited class)
/// plus a function name. The function name may be quoted (in which case it
/// refers to a builtin, like an operator).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct DeclarationName {
    pub node: DeclarationNameNode,
    pub prefix: Option<NamePart>,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationNameNode {
    Identifier(NamePart),
    Quoted(String),
    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Type {
    pub node: TypeNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TypeNode {
    NamePart(NamePart),
    Field(Box<Type>, NamePart),

    Applied(Box<Type>, Vec<Type>),

    /// A function type `() T` or `(T, U) V`.
    Function(Vec<Type>, Box<Type>),

    /// A reference.
    Reference(Box<Type>),

    Int,
    Nat,
    Boolean,
    Unit,

    Invalid,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Block(pub Vec<Statement>);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Statement {
    pub node: StatementNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum StatementNode {
    Expression(Expression),

    Variable(NamePart, Type, Expression),
    Constant(NamePart, Type, Expression),

    Assignment(NamePart, Expression),

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
    Field(Box<Expression>, String),

    NamePart(NamePart),
    Number(String),
    String(String),
    Unit,

    Invalid,
}
