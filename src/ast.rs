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
    pub name: DeclarationName,
    #[return_ref]
    pub node: DeclarationNode,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum DeclarationNode {
    Class {
        public: Vec<Declaration>,
        private: Vec<Declaration>,
        inherits: Vec<Type>,
    },

    Variant {
        public: Vec<Declaration>,
        private: Vec<Declaration>,
        inherits: Vec<Type>,
    },

    Function {
        /// `None` if the function does not take a `this` argument; `Some(n)` if
        /// it does, where `n` is the number of references it is behind.
        this: Option<usize>,
        args: Vec<(NamePart, Type)>,
        return_type: Type,
        body: Option<Block>,
    },

    Variable {
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
    Name(NamePart),
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

    Assignment(Expression, Expression),

    Return(Expression),

    Null,
    Invalid,
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

    Name(NamePart),
    Number(String),
    String(String),
    This,
    Unit,

    Invalid,
}
