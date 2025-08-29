use crate::ast::{
    operators::{InfixOperator, PrefixOperator},
    statement::Statement,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expression {
    Bool(bool),
    Int(i64),
    Ident(String),
    String(String),
    Infix {
        operator: InfixOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Prefix {
        operator: PrefixOperator,
        right: Box<Expression>,
    },
    Func {
        args: Vec<String>,
        body: Vec<Statement>,
    },
    Call {
        func: Box<Expression>,
        args: Vec<Expression>,
    },
    Cond {
        cond: Box<Expression>,
        then_: Vec<Statement>,
        else_: Option<Vec<Statement>>,
    },
    Array(Vec<Expression>),
    Hash(Vec<(Expression, Expression)>),
}

impl From<i64> for Expression {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<bool> for Expression {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Self {
        Self::Ident(value.to_owned())
    }
}

impl From<String> for Expression {
    fn from(value: String) -> Self {
        Self::Ident(value.to_owned())
    }
}
