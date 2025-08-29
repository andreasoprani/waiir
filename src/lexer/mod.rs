use std::iter::Peekable;
use std::str::Chars;

mod token;

pub use token::Token;

pub struct Lexer<'a> {
    chars_iter: Peekable<Chars<'a>>,
    ch: Option<char>,
}

impl<'a> Lexer<'a> {
    pub fn init(input: &'a str) -> Self {
        let mut lexer = Self {
            chars_iter: input.chars().peekable(),
            ch: None,
        };
        lexer.advance_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            Some('=') => {
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            Some('!') => {
                if self.peek_char() == Some('=') {
                    self.advance_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            Some('+') => Token::Plus,
            Some('-') => Token::Minus,
            Some('*') => Token::Asterisk,
            Some('/') => Token::Slash,
            Some('<') => Token::Lt,
            Some('>') => Token::Gt,
            Some(',') => Token::Comma,
            Some(';') => Token::Semicolon,
            Some(':') => Token::Colon,
            Some('(') => Token::LParen,
            Some(')') => Token::RParen,
            Some('{') => Token::LBrace,
            Some('}') => Token::RBrace,
            Some('[') => Token::LBracket,
            Some(']') => Token::RBracket,
            Some('a'..='z') => self.parse_identifier(),
            Some('0'..='9') => self.parse_number(),
            Some('"') => self.parse_string(),
            None => Token::Eof,
            _ => Token::Illegal,
        };

        self.advance_char();

        token
    }

    pub fn get_all_tokens(&mut self) -> Vec<Token> {
        let mut output: Vec<Token> = vec![];
        loop {
            output.push(self.next_token());
            if output.last().unwrap() == &Token::Eof {
                break;
            }
        }
        output
    }

    fn advance_char(&mut self) {
        self.ch = self.chars_iter.next()
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars_iter.peek().copied()
    }

    fn skip_whitespace(&mut self) {
        while self.ch == Some(' ')
            || self.ch == Some('\t')
            || self.ch == Some('\n')
            || self.ch == Some('\r')
        {
            self.advance_char();
        }
    }

    fn parse_identifier(&mut self) -> Token {
        let mut output = String::new();
        while let Some(ch) = self.ch {
            output.push(ch);
            let peek = self.peek_char();
            if peek.is_some() && peek.unwrap().is_alphabetic() {
                self.advance_char();
            } else {
                break;
            }
        }
        match output.as_str() {
            "fn" => Token::Function,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            _ => Token::Ident(output),
        }
    }

    fn parse_number(&mut self) -> Token {
        let mut output = 0;
        while let Some(ch) = self.ch {
            output = output * 10 + ch.to_digit(10).unwrap();
            let peek = self.peek_char();
            if peek.is_some() && peek.unwrap().is_numeric() {
                self.advance_char();
            } else {
                break;
            }
        }
        Token::Int(output as i64)
    }

    fn parse_string(&mut self) -> Token {
        self.advance_char();
        let mut string = String::new();
        while let Some(ch) = self.ch
            && ch != '"'
        {
            string.push(ch);
            self.advance_char();
        }
        Token::String(string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_input() {
        let mut lexer = Lexer::init("=+(){},;");
        assert_eq!(
            lexer.get_all_tokens(),
            vec![
                Token::Assign,
                Token::Plus,
                Token::LParen,
                Token::RParen,
                Token::LBrace,
                Token::RBrace,
                Token::Comma,
                Token::Semicolon,
                Token::Eof,
            ]
        )
    }

    #[test]
    fn extended_test() {
        let mut lexer = Lexer::init(
            "let five = 5; \n\
            let ten = 10; \n\
            \n\
            let add = fn(x, y) { \n\
              x + y; \n\
            }; \n\
            \n\
            let result = add(five, ten); \n\
            !-/*5; \n\
            5 < 10 > 5; \n\
            \n\
            if (5 < 10) { \n\
            	return true; \n\
            } else { \n\
            	return false; \n\
            } \n\
            \n\
            10 == 10; \n\
            10 != 9; \n\
            \"foobar\" \n\
            \"foo bar\" \n\
            [1, 2]; \n\
            {\"foo\": \"bar\"}",
        );
        assert_eq!(
            lexer.get_all_tokens(),
            vec![
                Token::Let,
                Token::Ident(String::from("five")),
                Token::Assign,
                Token::Int(5),
                Token::Semicolon,
                Token::Let,
                Token::Ident(String::from("ten")),
                Token::Assign,
                Token::Int(10),
                Token::Semicolon,
                Token::Let,
                Token::Ident(String::from("add")),
                Token::Assign,
                Token::Function,
                Token::LParen,
                Token::Ident(String::from("x")),
                Token::Comma,
                Token::Ident(String::from("y")),
                Token::RParen,
                Token::LBrace,
                Token::Ident(String::from("x")),
                Token::Plus,
                Token::Ident(String::from("y")),
                Token::Semicolon,
                Token::RBrace,
                Token::Semicolon,
                Token::Let,
                Token::Ident(String::from("result")),
                Token::Assign,
                Token::Ident(String::from("add")),
                Token::LParen,
                Token::Ident(String::from("five")),
                Token::Comma,
                Token::Ident(String::from("ten")),
                Token::RParen,
                Token::Semicolon,
                Token::Bang,
                Token::Minus,
                Token::Slash,
                Token::Asterisk,
                Token::Int(5),
                Token::Semicolon,
                Token::Int(5),
                Token::Lt,
                Token::Int(10),
                Token::Gt,
                Token::Int(5),
                Token::Semicolon,
                Token::If,
                Token::LParen,
                Token::Int(5),
                Token::Lt,
                Token::Int(10),
                Token::RParen,
                Token::LBrace,
                Token::Return,
                Token::True,
                Token::Semicolon,
                Token::RBrace,
                Token::Else,
                Token::LBrace,
                Token::Return,
                Token::False,
                Token::Semicolon,
                Token::RBrace,
                Token::Int(10),
                Token::Eq,
                Token::Int(10),
                Token::Semicolon,
                Token::Int(10),
                Token::NotEq,
                Token::Int(9),
                Token::Semicolon,
                Token::String(String::from("foobar")),
                Token::String(String::from("foo bar")),
                Token::LBracket,
                Token::Int(1),
                Token::Comma,
                Token::Int(2),
                Token::RBracket,
                Token::Semicolon,
                Token::LBrace,
                Token::String(String::from("foo")),
                Token::Colon,
                Token::String(String::from("bar")),
                Token::RBrace,
                Token::Eof
            ]
        )
    }
}
