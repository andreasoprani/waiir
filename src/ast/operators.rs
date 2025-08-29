use anyhow;

use crate::Token;
use std::fmt;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum PrefixOperator {
    Not,
    Neg,
}

impl TryFrom<&Token> for PrefixOperator {
    type Error = anyhow::Error;

    fn try_from(token: &Token) -> anyhow::Result<Self> {
        Ok(match token {
            Token::Bang => Self::Not,
            Token::Minus => Self::Neg,
            _ => anyhow::bail!("Invalid token {token} as a prefix operator"),
        })
    }
}

impl fmt::Display for PrefixOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrefixOperator::Not => write!(f, "`!`"),
            PrefixOperator::Neg => write!(f, "`-`"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum InfixOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    NotEq,
    Gt,
    Lt,
    Index,
}

impl TryFrom<&Token> for InfixOperator {
    type Error = anyhow::Error;

    fn try_from(token: &Token) -> anyhow::Result<Self> {
        Ok(match token {
            Token::Plus => Self::Add,
            Token::Minus => Self::Sub,
            Token::Asterisk => Self::Mul,
            Token::Slash => Self::Div,
            Token::Eq => Self::Eq,
            Token::NotEq => Self::NotEq,
            Token::Gt => Self::Gt,
            Token::Lt => Self::Lt,
            Token::LBracket => Self::Index,
            _ => anyhow::bail!("Invalid token {token} as a infix operator"),
        })
    }
}

impl fmt::Display for InfixOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InfixOperator::Add => write!(f, "`+`"),
            InfixOperator::Sub => write!(f, "`-`"),
            InfixOperator::Mul => write!(f, "`*`"),
            InfixOperator::Div => write!(f, "`/`"),
            InfixOperator::Eq => write!(f, "`==`"),
            InfixOperator::NotEq => write!(f, "`!=`"),
            InfixOperator::Gt => write!(f, "`>`"),
            InfixOperator::Lt => write!(f, "`<`"),
            InfixOperator::Index => write!(f, "`[...]`"),
        }
    }
}
