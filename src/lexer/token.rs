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

    LParen,   // (
    RParen,   // )
    LBrace,   // {
    RBrace,   // }
    LBracket, // [
    RBracket, // ]

    // Keyboards
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}
