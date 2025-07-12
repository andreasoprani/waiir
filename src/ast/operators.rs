use crate::Token;

#[derive(PartialEq, Debug)]
pub enum PrefixOperator {
    Not,
    Neg,
}

impl From<&Token> for PrefixOperator {
    fn from(token: &Token) -> Self {
        match token {
            Token::Bang => Self::Not,
            Token::Minus => Self::Neg,
            _ => panic!("Invalid current token as a prefix operator"),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum InfixOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Gt,
    Lt,
}

impl From<&Token> for InfixOperator {
    fn from(token: &Token) -> Self {
        match token {
            Token::Plus => Self::Add,
            Token::Minus => Self::Sub,
            Token::Asterisk => Self::Mul,
            Token::Slash => Self::Div,
            Token::Eq => Self::Eq,
            Token::NotEq => Self::NotEq,
            Token::Gt => Self::Gt,
            Token::Lt => Self::Lt,
            _ => panic!("Invalid current token as a infix operator"),
        }
    }
}
