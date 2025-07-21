use crate::{Expression, InfixOperator, Lexer, PrefixOperator, Program, Statement, Token};

mod macros;
use macros::assert_token;

mod precedence;
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

    pub fn parse_program(&mut self) -> Program {
        let mut statements: Vec<Statement> = vec![];

        while self.curr_token != Token::Eof {
            let stmt = self.parse_statement();
            statements.push(stmt);
            self.advance_token();
        }

        Program { statements }
    }

    fn parse_statement(&mut self) -> Statement {
        match self.curr_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::RBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Statement {
        self.advance_token();

        let name = if let Token::Ident(_name) = &self.curr_token {
            _name.clone()
        } else {
            panic!("Invalid statement");
        };
        self.advance_token();

        assert_token!(self.curr_token, Token::Assign);
        self.advance_token();

        let value = self.parse_expression(Precedence::Lowest);

        assert_token!(self.peek_token, Token::Semicolon);
        self.advance_token();

        Statement::Let { name, value }
    }

    fn parse_return_statement(&mut self) -> Statement {
        self.advance_token();

        let value = self.parse_expression(Precedence::Lowest);

        assert_token!(self.peek_token, Token::Semicolon);
        self.advance_token();

        Statement::Return { value }
    }

    fn parse_block_statement(&mut self) -> Statement {
        let mut statements: Vec<Statement> = vec![];

        while self.curr_token != Token::RBrace {
            statements.push(self.parse_statement());
            self.advance_token();
        }

        Statement::Block(statements)
    }

    fn parse_expression_statement(&mut self) -> Statement {
        let stmt = Statement::Expr(self.parse_expression(Precedence::Lowest));

        if self.peek_token == Token::Semicolon {
            self.advance_token();
        }

        stmt
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Expression {
        let mut left = self.parse_prefix();

        while self.peek_token != Token::Semicolon && precedence < self.peek_precedence() {
            self.advance_token();
            left = match self.curr_token {
                Token::LParen => self.parse_call_expression(left),
                _ => self.parse_infix_expression(left),
            }
        }

        left
    }

    fn parse_prefix(&mut self) -> Expression {
        match &self.curr_token {
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::Ident(value) => Expression::from(value.to_owned()),
            Token::Int(value) => Expression::from(value.to_owned()),
            Token::True => Expression::from(true),
            Token::False => Expression::from(false),
            Token::LParen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_fn_expression(),
            _ => {
                todo!("Prefix parsing for this token not implemented yet.")
            }
        }
    }

    fn parse_prefix_expression(&mut self) -> Expression {
        let operator = PrefixOperator::from(&self.curr_token);
        self.advance_token();
        Expression::Prefix {
            operator,
            right: Box::new(self.parse_expression(Precedence::Prefix)),
        }
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Expression {
        let precedence = self.curr_precedence();
        let operator = InfixOperator::from(&self.curr_token);

        self.advance_token();

        let right = self.parse_expression(precedence);

        Expression::Infix {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn parse_grouped_expression(&mut self) -> Expression {
        self.advance_token();

        let exp = self.parse_expression(Precedence::Lowest);

        assert_token!(self.peek_token, Token::RParen);
        self.advance_token();

        exp
    }

    fn parse_if_expression(&mut self) -> Expression {
        self.advance_token();

        assert_token!(self.curr_token, Token::LParen);
        self.advance_token();

        let cond = self.parse_expression(Precedence::Lowest);
        self.advance_token();

        assert_token!(self.curr_token, Token::RParen);
        self.advance_token();

        assert_token!(self.curr_token, Token::LBrace);
        self.advance_token();

        let then_ = match self.parse_block_statement() {
            Statement::Block(statements) => statements,
            _ => panic!("Wrong block statement return!"),
        };

        let else_ = if self.peek_token == Token::Else {
            self.advance_token();
            self.advance_token();

            assert_token!(self.curr_token, Token::LBrace);
            self.advance_token();

            Some(match self.parse_block_statement() {
                Statement::Block(statements) => statements,
                _ => panic!("Wrong block statement return!"),
            })
        } else {
            None
        };

        Expression::Cond {
            cond: Box::new(cond),
            then_,
            else_,
        }
    }

    fn parse_fn_expression(&mut self) -> Expression {
        self.advance_token();

        assert_token!(self.curr_token, Token::LParen);
        self.advance_token();

        let mut args: Vec<String> = vec![];

        while self.curr_token != Token::RParen {
            match &self.curr_token {
                Token::Ident(arg) => args.push(arg.to_string()),
                _ => panic!("Invalid function argument"),
            }

            self.advance_token();

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RParen => break,
                _ => panic!("Invalid token in function argument list"),
            }
        }

        self.advance_token();

        assert_token!(self.curr_token, Token::LBrace);
        self.advance_token();

        let body = match self.parse_block_statement() {
            Statement::Block(statements) => statements,
            _ => panic!("Wrong block statement return!"),
        };

        Expression::Func { args, body }
    }

    fn parse_call_expression(&mut self, func: Expression) -> Expression {
        self.advance_token();

        let mut args: Vec<Expression> = vec![];

        while self.curr_token != Token::RParen {
            args.push(self.parse_expression(Precedence::Lowest));

            self.advance_token();

            match &self.curr_token {
                Token::Comma => self.advance_token(),
                Token::RParen => break,
                _ => panic!("Invalid token in function argument list"),
            }
        }

        Expression::Call {
            func: Box::new(func),
            args,
        }
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
                        value: Expression::from(5)
                    },
                    Statement::Let {
                        name: String::from("ten"),
                        value: Expression::from(10)
                    },
                    Statement::Let {
                        name: String::from("foobar"),
                        value: Expression::from(838383)
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
                        value: Expression::from(5)
                    },
                    Statement::Return {
                        value: Expression::from(10)
                    },
                    Statement::Return {
                        value: Expression::from(993322)
                    },
                ]
            }
        );
    }

    #[test]
    fn base_expression() {
        let mut parser = Parser::init(
            "foobar; \n\
            5 \n\
            true;",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::from("foobar")),
                    Statement::Expr(Expression::from(5)),
                    Statement::Expr(Expression::from(true))
                ]
            }
        );
    }

    #[test]
    fn prefix_expressions() {
        let mut parser = Parser::init(
            "-5; \n\
            !15",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::Prefix {
                        operator: PrefixOperator::Neg,
                        right: Box::new(Expression::Int(5))
                    }),
                    Statement::Expr(Expression::Prefix {
                        operator: PrefixOperator::Not,
                        right: Box::new(Expression::Int(15))
                    })
                ]
            }
        );
    }

    #[test]
    fn infix_expressions() {
        let mut parser = Parser::init(
            "1 + 2; \n\
            3 - 4; \n\
            5 * 6; \n\
            7 / 8; \n\
            9 > 10; \n\
            11 < 12; \n\
            13 == 14; \n\
            15 != 16;",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Int(1)),
                        right: Box::new(Expression::Int(2))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Sub,
                        left: Box::new(Expression::Int(3)),
                        right: Box::new(Expression::Int(4))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::Int(5)),
                        right: Box::new(Expression::Int(6))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Div,
                        left: Box::new(Expression::Int(7)),
                        right: Box::new(Expression::Int(8))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Gt,
                        left: Box::new(Expression::Int(9)),
                        right: Box::new(Expression::Int(10))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Lt,
                        left: Box::new(Expression::Int(11)),
                        right: Box::new(Expression::Int(12))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::Int(13)),
                        right: Box::new(Expression::Int(14))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::NotEq,
                        left: Box::new(Expression::Int(15)),
                        right: Box::new(Expression::Int(16))
                    }),
                ]
            }
        );
    }

    #[test]
    fn op_precedence_expressions() {
        let mut parser = Parser::init(
            "-a * b; \n\
            !-a;                        \n\
            a + b + c;                  \n\
            a + b - c;                  \n\
            a * b * c;                  \n\
            a * b / c;                  \n\
            a + b / c;                  \n\
            a + b * c + d / e - f;      \n\
            3 + 4; -5 * 5;              \n\
            5 > 4 == 3 < 4;             \n\
            5 < 4 != 3 > 4;             \n\
            3 + 4 * 5 == 3 * 1 + 4 * 5; \n\
            true;                       \n\
            false;                      \n\
            3 > 5 == false;             \n\
            3 < 5 == true;",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::Prefix {
                            operator: PrefixOperator::Neg,
                            right: Box::new(Expression::from("a"))
                        }),
                        right: Box::new(Expression::from("b"))
                    }),
                    Statement::Expr(Expression::Prefix {
                        operator: PrefixOperator::Not,
                        right: Box::new(Expression::Prefix {
                            operator: PrefixOperator::Neg,
                            right: Box::new(Expression::from("a"))
                        })
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from("a")),
                            right: Box::new(Expression::from("b"))
                        }),
                        right: Box::new(Expression::from("c"))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Sub,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from("a")),
                            right: Box::new(Expression::from("b"))
                        }),
                        right: Box::new(Expression::from("c"))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from("a")),
                            right: Box::new(Expression::from("b"))
                        }),
                        right: Box::new(Expression::from("c"))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Div,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Mul,
                            left: Box::new(Expression::from("a")),
                            right: Box::new(Expression::from("b"))
                        }),
                        right: Box::new(Expression::from("c"))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::from("a")),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Div,
                            left: Box::new(Expression::from("b")),
                            right: Box::new(Expression::from("c"))
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
                            right: Box::new(Expression::from(5))
                        }),
                        right: Box::new(Expression::from(5))
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Gt,
                            left: Box::new(Expression::from(5)),
                            right: Box::new(Expression::from(4))
                        }),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Lt,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::from(4))
                        })
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::NotEq,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Lt,
                            left: Box::new(Expression::from(5)),
                            right: Box::new(Expression::from(4))
                        }),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Gt,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::from(4))
                        })
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::from(4)),
                                right: Box::new(Expression::from(5))
                            })
                        }),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::from(3)),
                                right: Box::new(Expression::from(1))
                            }),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Mul,
                                left: Box::new(Expression::from(4)),
                                right: Box::new(Expression::from(5))
                            })
                        })
                    }),
                    Statement::Expr(Expression::from(true)),
                    Statement::Expr(Expression::from(false)),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Gt,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::from(5))
                        }),
                        right: Box::new(Expression::from(false)),
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Eq,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Lt,
                            left: Box::new(Expression::from(3)),
                            right: Box::new(Expression::from(5))
                        }),
                        right: Box::new(Expression::from(true)),
                    }),
                ]
            }
        );
    }

    #[test]
    fn grouped_expressions_precedence() {
        let mut parser = Parser::init(
            "1 + (2 + 3) + 4;   \n\
            (5 + 5) * 2;                \n\
            2 / (5 + 5);                \n\
            -(5 + 5);                   \n\
            !(true == true);",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Add,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(1)),
                            right: Box::new(Expression::Infix {
                                operator: InfixOperator::Add,
                                left: Box::new(Expression::from(2)),
                                right: Box::new(Expression::from(3))
                            })
                        }),
                        right: Box::new(Expression::from(4)),
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Mul,
                        left: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(5)),
                            right: Box::new(Expression::from(5))
                        }),
                        right: Box::new(Expression::from(2)),
                    }),
                    Statement::Expr(Expression::Infix {
                        operator: InfixOperator::Div,
                        left: Box::new(Expression::from(2)),
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(5)),
                            right: Box::new(Expression::from(5))
                        }),
                    }),
                    Statement::Expr(Expression::Prefix {
                        operator: PrefixOperator::Neg,
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Add,
                            left: Box::new(Expression::from(5)),
                            right: Box::new(Expression::from(5))
                        }),
                    }),
                    Statement::Expr(Expression::Prefix {
                        operator: PrefixOperator::Not,
                        right: Box::new(Expression::Infix {
                            operator: InfixOperator::Eq,
                            left: Box::new(Expression::from(true)),
                            right: Box::new(Expression::from(true))
                        }),
                    }),
                ]
            }
        );
    }

    #[test]
    fn if_expressions() {
        let mut parser = Parser::init(
            "
        if (x < y) { x }; \n\
        if (x < y) { x } else { y };
        ",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
                    Statement::Expr(Expression::Cond {
                        cond: Box::new(Expression::Infix {
                            operator: InfixOperator::Lt,
                            left: Box::new(Expression::from("x")),
                            right: Box::new(Expression::from("y"))
                        }),
                        then_: vec![Statement::Expr(Expression::from("x"))],
                        else_: None
                    }),
                    Statement::Expr(Expression::Cond {
                        cond: Box::new(Expression::Infix {
                            operator: InfixOperator::Lt,
                            left: Box::new(Expression::from("x")),
                            right: Box::new(Expression::from("y"))
                        }),
                        then_: vec![Statement::Expr(Expression::from("x"))],
                        else_: Some(vec![Statement::Expr(Expression::from("y"))])
                    })
                ]
            }
        );
    }

    #[test]
    fn fn_expressions() {
        let mut parser = Parser::init(
            "
            fn() {}; \n\
            fn(x) {}; \n\
            fn(x, y, z) {}; \n\
            fn(x, y) { x + y; };
            ",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
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
                            right: Box::new(Expression::from("y"))
                        })],
                    }),
                ]
            }
        );
    }

    #[test]
    fn call_expressions() {
        let mut parser = Parser::init(
            "
            add(1, 2 * 3, 4 + 5);
            ",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![Statement::Expr(Expression::Call {
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
                    ]
                }),]
            }
        );
    }

    #[test]
    fn call_precedence() {
        let mut parser = Parser::init(
            "
            a + add(b * c) + d; \n\
            add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8)); \n\
            add(a + b + c * d / f + g);
            ",
        );
        let program = parser.parse_program();

        assert_eq!(
            program,
            Program {
                statements: vec![
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
                                }]
                            })
                        }),
                        right: Box::new(Expression::from("d"))
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
                                ]
                            }
                        ]
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
                        },]
                    }),
                ]
            }
        );
    }
}
