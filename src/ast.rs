#[derive(PartialEq, Debug)]
pub enum Expression {
    Bool(bool),
    Int(i64),
    String(String),
    Identifiter(String),
}

#[derive(PartialEq, Debug)]
pub enum Statement {
    Let { name: String, value: Expression },
    Expr(Expression),
}

#[derive(PartialEq, Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
