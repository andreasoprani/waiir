use std::fmt;

#[derive(PartialEq, Debug, Clone)]
pub enum Token {
    Illegal,
    Eof,

    // Identifiers and Literals
    Ident(String),
    Int(i64),
    String(String),

    // Operators
    Assign,   // =
    Plus,     // +
    Minus,    // -
    Bang,     // !
    Asterisk, // *
    Slash,    // /

    Lt, // <
    Gt, // >

    Eq,    // ==
    NotEq, // !=

    // Delimiters
    Comma,     // ,
    Semicolon, // ;
    Colon,     // :

    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Keyboards
    True,
    False,
    Function,
    Let,
    If,
    Else,
    Return,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Illegal => write!(f, "ILLEGAL TOKEN"),
            Token::Eof => write!(f, "EOF"),
            Token::Ident(value) => write!(f, "<identifier={value}>"),
            Token::Int(value) => write!(f, "<int={value}>"),
            Token::String(value) => write!(f, "<str={value}>"),
            Token::Assign => write!(f, "`=`"),
            Token::Plus => write!(f, "`+`"),
            Token::Minus => write!(f, "`-`"),
            Token::Bang => write!(f, "`!`"),
            Token::Asterisk => write!(f, "`*`"),
            Token::Slash => write!(f, "`/`"),
            Token::Lt => write!(f, "`<`"),
            Token::Gt => write!(f, "`>`"),
            Token::Eq => write!(f, "`==`"),
            Token::NotEq => write!(f, "`!=`"),
            Token::Comma => write!(f, "`,`"),
            Token::Semicolon => write!(f, "`;`"),
            Token::Colon => write!(f, "`:`"),
            Token::LParen => write!(f, "`(`"),
            Token::RParen => write!(f, "`)`"),
            Token::LBrace => write!(f, "`{{`"),
            Token::RBrace => write!(f, "`}}`"),
            Token::LBracket => write!(f, "`[`"),
            Token::RBracket => write!(f, "`]`"),
            Token::True => write!(f, "<bool=true>"),
            Token::False => write!(f, "<bool=false>"),
            Token::Function => write!(f, "`fn`"),
            Token::Let => write!(f, "`let`"),
            Token::If => write!(f, "`if`"),
            Token::Else => write!(f, "`else`"),
            Token::Return => write!(f, "`return`"),
        }
    }
}
