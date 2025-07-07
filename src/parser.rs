use crate::ast::{Expression, Program, Statement};
use crate::{lexer::Lexer, token::Token};

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    curr_token: Token,
    peek_token: Token,
}

impl<'a> Parser<'a> {
    pub fn init(input: &'a str) -> Self {
        let lexer = Lexer::init(input);

        let mut p = Parser {
            lexer,
            curr_token: Token::Illegal,
            peek_token: Token::Illegal,
        };

        p.advance_token();
        p.advance_token();

        p
    }

    fn advance_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn parse_let_statement(&mut self) -> Statement {
        self.advance_token();

        let name = if let Token::Ident(_name) = &self.curr_token {
            _name.clone()
        } else {
            panic!("Invalid statement");
        };
        self.advance_token();

        if self.curr_token != Token::Assign {
            panic!("Invalid statement");
        }
        self.advance_token();

        let value = self.parse_expression();

        if self.curr_token != Token::Semicolon {
            panic!("Invalid statement");
        }

        Statement::Let { name, value }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.advance_token();

        let value = self.parse_expression();

        if self.curr_token != Token::Semicolon {
            panic!("Invalid statement");
        }

        Statement::Return { value }
    }

    fn parse_expression(&mut self) -> Expression {
        let value;

        // TODO: move to parse expression
        if let Token::Int(_value) = self.curr_token {
            value = _value;
        } else {
            panic!("Invalid statement");
        }
        self.advance_token();

        Expression::Int(value)
    }

    fn parse_statement(&mut self) -> Statement {
        match self.curr_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            _ => todo!("Not implemented"),
        }
    }

    pub fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Statement> = vec![];

        while self.curr_token != Token::Eof {
            let stmt = self.parse_statement();
            statements.push(stmt);
            self.advance_token();
        }

        Program { statements }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_parser() {
        let parser = Parser::init("=+(){},;");

        assert_eq!(parser.curr_token, Token::Assign);
        assert_eq!(parser.peek_token, Token::Plus);
    }

    #[test]
    fn let_stmts() {
        let mut parser = Parser::init(
            "let five = 5; \n\
            let ten = 10; \n\
            let foobar = 838383;",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Let {
                        name: String::from("five"),
                        value: Expression::Int(5)
                    },
                    Statement::Let {
                        name: String::from("ten"),
                        value: Expression::Int(10)
                    },
                    Statement::Let {
                        name: String::from("foobar"),
                        value: Expression::Int(838383)
                    },
                ]
            }
        );
    }

    #[test]
    fn return_stmts() {
        let mut parser = Parser::init(
            "return 5; \n\
            return 10; \n\
            return 993322;",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Return {
                        value: Expression::Int(5)
                    },
                    Statement::Return {
                        value: Expression::Int(10)
                    },
                    Statement::Return {
                        value: Expression::Int(993322)
                    },
                ]
            }
        );
    }
}
