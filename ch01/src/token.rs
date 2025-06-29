#[derive(PartialEq, Debug)]
pub enum Token {
    Illegal,
    Eof,

    // Identifiers and Literals
    Ident(String),
    Int(i64),

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

    LParen, // (
    RParen, // )
    LBrace, // {
    RBrace, // }

    // Keyboards
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}
