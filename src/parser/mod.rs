use crate::{Expression, InfixOperator, Lexer, PrefixOperator, Program, Statement, Token};

mod macros;
use macros::assert_token;

mod precedence;
use anyhow::{Result, bail};
use precedence::Precedence;

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

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut statements: Vec<Statement> = vec![];

        while self.curr_token != Token::Eof {
            statements.push(self.parse_statement()?);
            self.advance_token();
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.curr_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::RBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement> {
        self.advance_token();

        let name = if let Token::Ident(_name) = &self.curr_token {
            _name.clone()
        } else {
            bail!(
                "Invalid Token for let statement, expected an identifier, found {}",
                &self.curr_token
            );
        };
        self.advance_token();

        assert_token!(self.curr_token, Token::Assign);
        self.advance_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        assert_token!(self.peek_token, Token::Semicolon | Token::Eof);
        self.advance_token();

        Ok(Statement::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<Statement> {
        self.advance_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        assert_token!(self.peek_token, Token::Semicolon);
        self.advance_token();

        Ok(Statement::Return { value })
    }

    fn parse_block_statement(&mut self) -> Result<Statement> {
        let mut statements: Vec<Statement> = vec![];

        while self.curr_token != Token::RBrace {
            statements.push(self.parse_statement()?);
            self.advance_token();
        }

        Ok(Statement::Block(statements))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let stmt = Statement::Expr(self.parse_expression(Precedence::Lowest)?);

        if self.peek_token == Token::Semicolon {
            self.advance_token();
        }

        Ok(stmt)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression> {
        let mut left = self.parse_prefix()?;

        while self.peek_token != Token::Semicolon && precedence < self.peek_precedence() {
            self.advance_token();
            left = match self.curr_token {
                Token::LParen => self.parse_call_expression(left)?,
                _ => self.parse_infix_expression(left)?,
            }
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expression> {
        match &self.curr_token {
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::Ident(value) => Ok(Expression::Ident(value.to_owned())),
            Token::Int(value) => Ok(Expression::Int(value.to_owned())),
            Token::String(string) => Ok(Expression::String(string.to_owned())),
            Token::True => Ok(Expression::from(true)),
            Token::False => Ok(Expression::from(false)),
            Token::LParen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_fn_expression(),
            Token::LBracket => self.parse_array_expression(),
            Token::LBrace => self.parse_hash_expression(),
            _ => bail!("{} is an invalid token as a prefix.", self.curr_token),
        }
    }

    fn parse_prefix_expression(&mut self) -> Result<Expression> {
        let operator = PrefixOperator::try_from(&self.curr_token)?;
        self.advance_token();
        Ok(Expression::Prefix {
            operator,
            right: Box::new(self.parse_expression(Precedence::Prefix)?),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Result<Expression> {
        let operator = InfixOperator::try_from(&self.curr_token)?;
        let precedence = match operator {
            InfixOperator::Index => Precedence::Lowest,
            _ => self.curr_precedence(),
        };
        self.advance_token();

        let right = self.parse_expression(precedence)?;

        if operator == InfixOperator::Index {
            assert_token!(self.peek_token, Token::RBracket);
            self.advance_token();
        }

        Ok(Expression::Infix {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Result<Expression> {
        self.advance_token();

        let exp = self.parse_expression(Precedence::Lowest);

        assert_token!(self.peek_token, Token::RParen);
        self.advance_token();

        exp
    }

    fn parse_array_expression(&mut self) -> Result<Expression> {
        self.advance_token();

        let mut content: Vec<Expression> = vec![];

        while self.curr_token != Token::RBracket {
            content.push(self.parse_expression(Precedence::Lowest)?);

            self.advance_token();

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RBracket => break,
                _ => bail!(
                    "Invalid token found while parsing array arguments, expected , as separator or ] to close, found {}",
                    &self.curr_token
                ),
            }
        }

        Ok(Expression::Array(content))
    }

    fn parse_hash_expression(&mut self) -> Result<Expression> {
        self.advance_token();

        let mut content: Vec<(Expression, Expression)> = vec![];

        while self.curr_token != Token::RBrace {
            let left = self.parse_expression(Precedence::Lowest)?;
            self.advance_token();

            assert_token!(self.curr_token, Token::Colon);
            self.advance_token();

            let right = self.parse_expression(Precedence::Lowest)?;
            self.advance_token();

            content.push((left, right));

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RBrace => break,
                _ => bail!(
                    "Invalid token found while parsing hashmap arguments, expected , as separator or }} to close, found {}",
                    &self.curr_token
                ),
            }
        }

        Ok(Expression::Hash(content))
    }

    fn parse_if_expression(&mut self) -> Result<Expression> {
        self.advance_token();

        assert_token!(self.curr_token, Token::LParen);
        self.advance_token();

        let cond = self.parse_expression(Precedence::Lowest)?;
        self.advance_token();

        assert_token!(self.curr_token, Token::RParen);
        self.advance_token();

        assert_token!(self.curr_token, Token::LBrace);
        self.advance_token();

        let then_ = match self.parse_block_statement()? {
            Statement::Block(statements) => statements,
            _ => bail!("The `then` part of an if statement must be a block."),
        };

        let else_ = if self.peek_token == Token::Else {
            self.advance_token();
            self.advance_token();

            assert_token!(self.curr_token, Token::LBrace);
            self.advance_token();

            Some(match self.parse_block_statement()? {
                Statement::Block(statements) => statements,
                _ => bail!("The `else` part of an if statement must be a block."),
            })
        } else {
            None
        };

        Ok(Expression::Cond {
            cond: Box::new(cond),
            then_,
            else_,
        })
    }

    fn parse_fn_expression(&mut self) -> Result<Expression> {
        self.advance_token();

        assert_token!(self.curr_token, Token::LParen);
        self.advance_token();

        let mut args: Vec<String> = vec![];

        while self.curr_token != Token::RParen {
            match &self.curr_token {
                Token::Ident(arg) => args.push(arg.to_string()),
                _ => bail!("A function name must be an identifier."),
            }

            self.advance_token();

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RParen => break,
                _ => bail!(
                    "Invalid token found while parsing function arguments, expected , as separator or ) to close, found {}",
                    &self.curr_token
                ),
            }
        }

        self.advance_token();

        assert_token!(self.curr_token, Token::LBrace);
        self.advance_token();

        let body = match self.parse_block_statement()? {
            Statement::Block(statements) => statements,
            _ => bail!("A function body must be enclosed in a block."),
        };

        Ok(Expression::Func { args, body })
    }

    fn parse_call_expression(&mut self, func: Expression) -> Result<Expression> {
        self.advance_token();

        let mut args: Vec<Expression> = vec![];

        while self.curr_token != Token::RParen {
            args.push(self.parse_expression(Precedence::Lowest)?);

            self.advance_token();

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RParen => break,
                _ => bail!(
                    "Invalid token found while parsing function arguments, expected , as separator or ) to close, found {}",
                    &self.curr_token
                ),
            }
        }

        Ok(Expression::Call {
            func: Box::new(func),
            args,
        })
    }

    fn peek_precedence(&mut self) -> Precedence {
        Precedence::get_from_token(&self.peek_token)
    }

    fn curr_precedence(&mut self) -> Precedence {
        Precedence::get_from_token(&self.curr_token)
    }

    fn advance_token(&mut self) {
        self.curr_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_program(input: &str, statements: Vec<Statement>) {
        let mut parser = Parser::init(input);
        let program = parser.parse_program().unwrap();

        assert_eq!(program, Program { statements })
    }

    #[test]
    fn init_parser() {
        let parser = Parser::init("=+(){},;");

        assert_eq!(parser.curr_token, Token::Assign);
        assert_eq!(parser.peek_token, Token::Plus);
    }

    #[test]
    fn let_stmts() {
        assert_program(
            "let five = 5; \n\
            let ten = 10; \n\
            let foobar = 838383;",
            vec![
                Statement::Let {
                    name: String::from("five"),
                    value: Expression::from(5),
                },
                Statement::Let {
                    name: String::from("ten"),
                    value: Expression::from(10),
                },
                Statement::Let {
                    name: String::from("foobar"),
                    value: Expression::from(838383),
                },
            ],
        );
    }

    #[test]
    fn return_stmts() {
        assert_program(
            "return 5; \n\
            return 10; \n\
            return 993322;",
            vec![
                Statement::Return {
                    value: Expression::from(5),
                },
                Statement::Return {
                    value: Expression::from(10),
                },
                Statement::Return {
                    value: Expression::from(993322),
                },
            ],
        );
    }

    #[test]
    fn base_expression() {
        assert_program(
            "foobar; \n\
            5 \n\
            true;",
            vec![
                Statement::Expr(Expression::from("foobar")),
                Statement::Expr(Expression::from(5)),
                Statement::Expr(Expression::from(true)),
            ],
        );
    }

    #[test]
    fn prefix_expressions() {
        assert_program(
            "-5; \n\
            !15",
            vec![
                Statement::Expr(Expression::Prefix {
                    operator: PrefixOperator::Neg,
                    right: Box::new(Expression::Int(5)),
                }),
                Statement::Expr(Expression::Prefix {
                    operator: PrefixOperator::Not,
                    right: Box::new(Expression::Int(15)),
                }),
            ],
        );
    }

    #[test]
    fn infix_expressions() {
        assert_program(
            "1 + 2; \n\
            3 - 4; \n\
            5 * 6; \n\
            7 / 8; \n\
            9 > 10; \n\
            11 < 12; \n\
            13 == 14; \n\
            15 != 16;",
            vec![
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Int(1)),
                    right: Box::new(Expression::Int(2)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Sub,
                    left: Box::new(Expression::Int(3)),
                    right: Box::new(Expression::Int(4)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Int(5)),
                    right: Box::new(Expression::Int(6)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Div,
                    left: Box::new(Expression::Int(7)),
                    right: Box::new(Expression::Int(8)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Gt,
                    left: Box::new(Expression::Int(9)),
                    right: Box::new(Expression::Int(10)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Lt,
                    left: Box::new(Expression::Int(11)),
                    right: Box::new(Expression::Int(12)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Eq,
                    left: Box::new(Expression::Int(13)),
                    right: Box::new(Expression::Int(14)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::NotEq,
                    left: Box::new(Expression::Int(15)),
                    right: Box::new(Expression::Int(16)),
                }),
            ],
        );
    }

    #[test]
    fn op_precedence_expressions() {
        assert_program(
            "-a * b; \n\
            !-a;                                \n\
            a + b + c;                          \n\
            a + b - c;                          \n\
            a * b * c;                          \n\
            a * b / c;                          \n\
            a + b / c;                          \n\
            a + b * c + d / e - f;              \n\
            3 + 4; -5 * 5;                      \n\
            5 > 4 == 3 < 4;                     \n\
            5 < 4 != 3 > 4;                     \n\
            3 + 4 * 5 == 3 * 1 + 4 * 5;         \n\
            true;                               \n\
            false;                              \n\
            3 > 5 == false;                     \n\
            3 < 5 == true;                      \n\
            a * [1, 2, 3, 4][b * c] * d         \n\
            add(a * b[2], b[1], 2 * [1, 2][1])  \n\
            ",
            vec![
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Prefix {
                        operator: PrefixOperator::Neg,
                        right: Box::new(Expression::from("a")),
                    }),
                    right: Box::new(Expression::from("b")),
                }),
                Statement::Expr(Expression::Prefix {
                    operator: PrefixOperator::Not,
                    right: Box::new(Expression::Prefix {
                        operator: PrefixOperator::Neg,
                        right: Box::new(Expression::from("a")),
                    }),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::from("b")),
                    }),
                    right: Box::new(Expression::from("c")),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Sub,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::from("b")),
                    }),
                    right: Box::new(Expression::from("c")),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::from("b")),
                    }),
                    right: Box::new(Expression::from("c")),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Div,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::from("b")),
                    }),
                    right: Box::new(Expression::from("c")),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::from("a")),
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Div,
                        left: Box::new(Expression::from("b")),
                        right: Box::new(Expression::from("c")),
                    }),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Sub,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from("a")),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::from("b")),
                                right: Box::new(Expression::from("c")),
                            }),
                        }),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Div,
                            left: Box::new(Expression::from("d")),
                            right: Box::new(Expression::from("e")),
                        }),
                    }),
                    right: Box::new(Expression::from("f")),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::from(3)),
                    right: Box::new(Expression::from(4)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Prefix {
                        operator: PrefixOperator::Neg,
                        right: Box::new(Expression::from(5)),
                    }),
                    right: Box::new(Expression::from(5)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Eq,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Gt,
                        left: Box::new(Expression::from(5)),
                        right: Box::new(Expression::from(4)),
                    }),
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::from(3)),
                        right: Box::new(Expression::from(4)),
                    }),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::NotEq,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::from(5)),
                        right: Box::new(Expression::from(4)),
                    }),
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Gt,
                        left: Box::new(Expression::from(3)),
                        right: Box::new(Expression::from(4)),
                    }),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Eq,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(3)),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from(4)),
                            right: Box::new(Expression::from(5)),
                        }),
                    }),
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::from(1)),
                        }),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from(4)),
                            right: Box::new(Expression::from(5)),
                        }),
                    }),
                }),
                Statement::Expr(Expression::from(true)),
                Statement::Expr(Expression::from(false)),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Eq,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Gt,
                        left: Box::new(Expression::from(3)),
                        right: Box::new(Expression::from(5)),
                    }),
                    right: Box::new(Expression::from(false)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Eq,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::from(3)),
                        right: Box::new(Expression::from(5)),
                    }),
                    right: Box::new(Expression::from(true)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::Ident("a".into())),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Index,
                            left: Box::new(Expression::Array(vec![
                                Expression::from(1),
                                Expression::from(2),
                                Expression::from(3),
                                Expression::from(4),
                            ])),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::Ident("b".into())),
                                right: Box::new(Expression::Ident("c".into())),
                            }),
                        }),
                    }),
                    right: Box::new(Expression::Ident("d".into())),
                }),
                Statement::Expr(Expression::Call {
                    func: Box::new(Expression::Ident("add".into())),
                    args: vec![
                        Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::Ident("a".into())),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Index,
                                left: Box::new(Expression::Ident("b".into())),
                                right: Box::new(Expression::from(2)),
                            }),
                        },
                        Expression::Infix {
                            operator: InfixOperator::Index,
                            left: Box::new(Expression::Ident("b".into())),
                            right: Box::new(Expression::from(1)),
                        },
                        Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from(2)),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Index,
                                left: Box::new(Expression::Array(vec![
                                    Expression::from(1),
                                    Expression::from(2),
                                ])),
                                right: Box::new(Expression::from(1)),
                            }),
                        },
                    ],
                }),
            ],
        );
    }

    #[test]
    fn grouped_expressions_precedence() {
        assert_program(
            "1 + (2 + 3) + 4;   \n\
            (5 + 5) * 2;                \n\
            2 / (5 + 5);                \n\
            -(5 + 5);                   \n\
            !(true == true);",
            vec![
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(1)),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(2)),
                            right: Box::new(Expression::from(3)),
                        }),
                    }),
                    right: Box::new(Expression::from(4)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(5)),
                        right: Box::new(Expression::from(5)),
                    }),
                    right: Box::new(Expression::from(2)),
                }),
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Div,
                    left: Box::new(Expression::from(2)),
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(5)),
                        right: Box::new(Expression::from(5)),
                    }),
                }),
                Statement::Expr(Expression::Prefix {
                    operator: PrefixOperator::Neg,
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(5)),
                        right: Box::new(Expression::from(5)),
                    }),
                }),
                Statement::Expr(Expression::Prefix {
                    operator: PrefixOperator::Not,
                    right: Box::new(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::from(true)),
                        right: Box::new(Expression::from(true)),
                    }),
                }),
            ],
        );
    }

    #[test]
    fn if_expressions() {
        assert_program(
            "
        if (x < y) { x }; \n\
        if (x < y) { x } else { y };
        ",
            vec![
                Statement::Expr(Expression::Cond {
                    cond: Box::new(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::from("x")),
                        right: Box::new(Expression::from("y")),
                    }),
                    then_: vec![Statement::Expr(Expression::from("x"))],
                    else_: None,
                }),
                Statement::Expr(Expression::Cond {
                    cond: Box::new(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::from("x")),
                        right: Box::new(Expression::from("y")),
                    }),
                    then_: vec![Statement::Expr(Expression::from("x"))],
                    else_: Some(vec![Statement::Expr(Expression::from("y"))]),
                }),
            ],
        );
    }

    #[test]
    fn fn_expressions() {
        assert_program(
            "
            fn() {}; \n\
            fn(x) {}; \n\
            fn(x, y, z) {}; \n\
            fn(x, y) { x + y; };
            ",
            vec![
                Statement::Expr(Expression::Func {
                    args: vec![],
                    body: vec![],
                }),
                Statement::Expr(Expression::Func {
                    args: vec![String::from("x")],
                    body: vec![],
                }),
                Statement::Expr(Expression::Func {
                    args: vec![String::from("x"), String::from("y"), String::from("z")],
                    body: vec![],
                }),
                Statement::Expr(Expression::Func {
                    args: vec![String::from("x"), String::from("y")],
                    body: vec![Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from("x")),
                        right: Box::new(Expression::from("y")),
                    })],
                }),
            ],
        );
    }

    #[test]
    fn call_expressions() {
        assert_program(
            "
            add(1, 2 * 3, 4 + 5);
            ",
            vec![Statement::Expr(Expression::Call {
                func: Box::new(Expression::from("add")),
                args: vec![
                    Expression::from(1),
                    Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::from(2)),
                        right: Box::new(Expression::from(3)),
                    },
                    Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from(4)),
                        right: Box::new(Expression::from(5)),
                    },
                ],
            })],
        );
    }

    #[test]
    fn call_precedence() {
        assert_program(
            "
            a + add(b * c) + d; \n\
            add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8)); \n\
            add(a + b + c * d / f + g);
            ",
            vec![
                Statement::Expr(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::Call {
                            func: Box::new(Expression::from("add")),
                            args: vec![Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::from("b")),
                                right: Box::new(Expression::from("c")),
                            }],
                        }),
                    }),
                    right: Box::new(Expression::from("d")),
                }),
                Statement::Expr(Expression::Call {
                    func: Box::new(Expression::from("add")),
                    args: vec![
                        Expression::from("a"),
                        Expression::from("b"),
                        Expression::from(1),
                        Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from(2)),
                            right: Box::new(Expression::from(3)),
                        },
                        Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(4)),
                            right: Box::new(Expression::from(5)),
                        },
                        Expression::Call {
                            func: Box::new(Expression::from("add")),
                            args: vec![
                                Expression::from(6),
                                Expression::Infix {
                                    operator: InfixOperator::Mul,
                                    left: Box::new(Expression::from(7)),
                                    right: Box::new(Expression::from(8)),
                                },
                            ],
                        },
                    ],
                }),
                Statement::Expr(Expression::Call {
                    func: Box::new(Expression::from("add")),
                    args: vec![Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::Infix {
                                operator: InfixOperator::Add,
                                left: Box::new(Expression::from("a")),
                                right: Box::new(Expression::from("b")),
                            }),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Div,
                                left: Box::new(Expression::Infix {
                                    operator: InfixOperator::Mul,
                                    left: Box::new(Expression::from("c")),
                                    right: Box::new(Expression::from("d")),
                                }),
                                right: Box::new(Expression::from("f")),
                            }),
                        }),
                        right: Box::new(Expression::from("g")),
                    }],
                }),
            ],
        );
    }

    #[test]
    fn string_literal_expression() {
        assert_program(
            "\"hello world\"",
            vec![Statement::Expr(Expression::String(String::from(
                "hello world",
            )))],
        );
    }

    #[test]
    fn array_literal_expression() {
        assert_program(
            "[1, 2 * 2, 3 + 3]",
            vec![Statement::Expr(Expression::Array(vec![
                Expression::Int(1),
                Expression::Infix {
                    operator: InfixOperator::Mul,
                    left: Box::new(Expression::Int(2)),
                    right: Box::new(Expression::Int(2)),
                },
                Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Int(3)),
                    right: Box::new(Expression::Int(3)),
                },
            ]))],
        );
    }

    #[test]
    fn array_indexing() {
        assert_program(
            "myArray[1 + 1]",
            vec![Statement::Expr(Expression::Infix {
                operator: InfixOperator::Index,
                left: Box::new(Expression::Ident("myArray".into())),
                right: Box::new(Expression::Infix {
                    operator: InfixOperator::Add,
                    left: Box::new(Expression::Int(1)),
                    right: Box::new(Expression::Int(1)),
                }),
            })],
        );
    }

    #[test]
    fn hash_literal_expression() {
        assert_program(
            "{\"one\": 1, \"two\": 2, \"three\": 3}; \n\
            {true: 1, false: 0}; \n\
            {1: 2, 2: 4}; \n\
            {}; \n\
            {\"one\": 0 + 1, two: 10 - 8, \"th\" + \"ree\": 15 / 5};",
            vec![
                Statement::Expr(Expression::Hash(vec![
                    (Expression::String(String::from("one")), Expression::from(1)),
                    (Expression::String(String::from("two")), Expression::from(2)),
                    (
                        Expression::String(String::from("three")),
                        Expression::from(3),
                    ),
                ])),
                Statement::Expr(Expression::Hash(vec![
                    (Expression::from(true), Expression::from(1)),
                    (Expression::from(false), Expression::from(0)),
                ])),
                Statement::Expr(Expression::Hash(vec![
                    (Expression::from(1), Expression::from(2)),
                    (Expression::from(2), Expression::from(4)),
                ])),
                Statement::Expr(Expression::Hash(vec![])),
                Statement::Expr(Expression::Hash(vec![
                    (
                        Expression::String(String::from("one")),
                        Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(0)),
                            right: Box::new(Expression::from(1)),
                        },
                    ),
                    (
                        Expression::Ident(String::from("two")),
                        Expression::Infix {
                            operator: InfixOperator::Sub,
                            left: Box::new(Expression::from(10)),
                            right: Box::new(Expression::from(8)),
                        },
                    ),
                    (
                        Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::String(String::from("th"))),
                            right: Box::new(Expression::String(String::from("ree"))),
                        },
                        Expression::Infix {
                            operator: InfixOperator::Div,
                            left: Box::new(Expression::from(15)),
                            right: Box::new(Expression::from(5)),
                        },
                    ),
                ])),
            ],
        );
    }
}
