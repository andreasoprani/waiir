use crate::token::Token;

pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: char,
}

impl Lexer {
    pub fn init(input: &str) -> Self {
        let mut lexer = Self {
            input: String::from(input),
            position: 0,
            read_position: 0,
            ch: char::MIN,
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            '=' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            '!' => {
                if self.peek_char() == '=' {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Asterisk,
            '/' => Token::Slash,
            '<' => Token::Lt,
            '>' => Token::Gt,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            char::MIN => Token::Eof,
            'a'..='z' => self.parse_identifier(),
            '0'..='9' => self.parse_number(),
            _ => Token::Illegal,
        };

        self.read_char();

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

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = char::MIN;
        } else {
            self.ch = self.input.chars().nth(self.read_position).unwrap();
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn peek_char(&mut self) -> char {
        if self.read_position >= self.input.len() {
            char::MIN
        } else {
            self.input.chars().nth(self.read_position).unwrap()
        }
    }

    fn skip_whitespace(&mut self) {
        while self.ch == ' ' || self.ch == '\t' || self.ch == '\n' || self.ch == '\r' {
            self.read_char();
        }
    }

    fn parse_identifier(&mut self) -> Token {
        let mut output = String::new();
        loop {
            output.push(self.ch);
            if self.peek_char().is_alphabetic() {
                self.read_char();
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
        loop {
            output = output * 10 + self.ch.to_digit(10).unwrap();
            if self.peek_char().is_numeric() {
                self.read_char();
            } else {
                break;
            }
        }
        Token::Int(output as i64)
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
            10 != 9;",
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
                Token::Eof
            ]
        )
    }

    #[test]
    fn perf_test() {
        let input = "let five = 5; \n\
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
            10 != 9;"
            .repeat(100);
        let mut lexer = Lexer::init(input.as_str());

        let tokens = lexer.get_all_tokens();
        assert_eq!(tokens.len(), 73 * 100 + 1);
    }
}
