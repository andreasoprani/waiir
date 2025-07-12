use crate::ast::expression::Expression;

#[derive(PartialEq, Debug)]
pub enum Statement {
    Let { name: String, value: Expression },
    Return { value: Expression },
    Expr(Expression),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
