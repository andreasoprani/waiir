mod environment;
mod object;

use crate::{Expression, InfixOperator, Parser, PrefixOperator, Program, Statement};
use environment::Environment;
use object::Object;
use std::rc::Rc;

pub trait Eval {
    fn eval(self, env: Rc<Environment>) -> Object;
}

impl Eval for Program {
    fn eval(self, env: Rc<Environment>) -> Object {
        let mut result: Object = Object::Null;
        for statement in self.statements {
            result = statement.eval(Rc::clone(&env));
            if let Object::Return(_res) = result {
                return *_res;
            }
        }
        result
    }
}

impl Eval for Statement {
    fn eval(self, env: Rc<Environment>) -> Object {
        match self {
            Statement::Expr(expr) => expr.eval(Rc::clone(&env)),
            Statement::Block(stmts) => {
                let mut result: Object = Object::Null;
                for statement in stmts {
                    result = statement.eval(Rc::clone(&env));
                    if matches!(result, Object::Return(_)) {
                        return result;
                    }
                }
                result
            }
            Statement::Let { name, value } => {
                let obj = value.eval(Rc::clone(&env));
                env.set_var(name, obj)
            }
            Statement::Return { value } => Object::Return(Box::new(value.eval(Rc::clone(&env)))),
        }
    }
}

impl Eval for Expression {
    fn eval(self, env: Rc<Environment>) -> Object {
        match self {
            Expression::Bool(value) => Object::Bool(value),
            Expression::Int(value) => Object::Int(value),
            Expression::Ident(ident) if ident == "null" => Object::Null,
            Expression::Ident(ident) => env.get_var(ident),
            Expression::Prefix { operator, right } => {
                Expression::eval_prefix(operator, right.eval(env))
            }
            Expression::Infix {
                operator,
                left,
                right,
            } => Expression::eval_infix(operator, left.eval(Rc::clone(&env)), right.eval(env)),
            Expression::Cond { cond, then_, else_ } => {
                let evaluated_cond = cond.eval(Rc::clone(&env)).to_bool();

                if evaluated_cond {
                    Statement::Block(then_).eval(env)
                } else if let Some(stmts) = else_ {
                    Statement::Block(stmts).eval(env)
                } else {
                    Object::Null
                }
            }
            _ => todo!(),
        }
    }
}

impl Expression {
    fn eval_prefix(operator: PrefixOperator, right: Object) -> Object {
        match operator {
            PrefixOperator::Neg => Self::eval_neg(right),
            PrefixOperator::Not => Object::Bool(Self::eval_bang(right)),
        }
    }

    fn eval_neg(right: Object) -> Object {
        match right {
            Object::Int(value) => Object::Int(-value),
            Object::Return(value) => Self::eval_neg(*value),
            _ => {
                println!("{right:?} cannot be negated!");
                panic!();
            }
        }
    }

    fn eval_bang(right: Object) -> bool {
        match right {
            Object::Bool(value) => !value,
            Object::Int(value) => value == 0,
            Object::Null => true,
            Object::Return(value) => Self::eval_bang(*value),
        }
    }

    fn eval_infix(operator: InfixOperator, left: Object, right: Object) -> Object {
        match (left, right) {
            (Object::Null, Object::Null) => Object::Null,
            (Object::Bool(l), Object::Bool(r)) => match operator {
                InfixOperator::Eq => Object::Bool(l == r),
                InfixOperator::NotEq => Object::Bool(l != r),
                _ => {
                    println!("Cannot perform operation {operator:?} on booleans!");
                    panic!();
                }
            },
            (Object::Int(l), Object::Int(r)) => match operator {
                InfixOperator::Add => Object::Int(l + r),
                InfixOperator::Sub => Object::Int(l - r),
                InfixOperator::Mul => Object::Int(l * r),
                InfixOperator::Div => Object::Int(l / r),
                InfixOperator::Eq => Object::Bool(l == r),
                InfixOperator::NotEq => Object::Bool(l != r),
                InfixOperator::Gt => Object::Bool(l > r),
                InfixOperator::Lt => Object::Bool(l < r),
            },
            (l, r) => {
                println!("Invalid operation ({operator:?}) between {l:?} and {r:?}!");
                panic!();
            }
        }
    }
}

pub fn eval_input(input: &str) -> Object {
    let env = Environment::default();
    Parser::init(input).parse_program().eval(Rc::new(env))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_expression() {
        assert_eq!(eval_input("5"), Object::Int(5));
        assert_eq!(eval_input("10"), Object::Int(10));
        assert_eq!(eval_input("-5"), Object::Int(-5));
        assert_eq!(eval_input("-10"), Object::Int(-10));
        assert_eq!(eval_input("5 + 5 + 5 + 5 - 10"), Object::Int(10));
        assert_eq!(eval_input("2 * 2 * 2 * 2 * 2"), Object::Int(32));
        assert_eq!(eval_input("-50 + 100 + -50"), Object::Int(0));
        assert_eq!(eval_input("5 * 2 + 10"), Object::Int(20));
        assert_eq!(eval_input("5 + 2 * 10"), Object::Int(25));
        assert_eq!(eval_input("20 + 2 * -10"), Object::Int(0));
        assert_eq!(eval_input("50 / 2 * 2 + 10"), Object::Int(60));
        assert_eq!(eval_input("2 * (5 + 10)"), Object::Int(30));
        assert_eq!(eval_input("3 * 3 * 3 + 10"), Object::Int(37));
        assert_eq!(eval_input("3 * (3 * 3) + 10"), Object::Int(37));
        assert_eq!(
            eval_input("(5 + 10 * 2 + 15 / 3) * 2 + -10"),
            Object::Int(50)
        );
    }

    #[test]
    fn boolean_expression() {
        assert_eq!(eval_input("true"), Object::Bool(true));
        assert_eq!(eval_input("false"), Object::Bool(false));
        assert_eq!(eval_input("!true"), Object::Bool(false));
        assert_eq!(eval_input("!false"), Object::Bool(true));
        assert_eq!(eval_input("!5"), Object::Bool(false));
        assert_eq!(eval_input("!!true"), Object::Bool(true));
        assert_eq!(eval_input("!!false"), Object::Bool(false));
        assert_eq!(eval_input("!!5"), Object::Bool(true));
        assert_eq!(eval_input("!null"), Object::Bool(true));
        assert_eq!(eval_input("!!null"), Object::Bool(false));
        assert_eq!(eval_input("1 < 2"), Object::Bool(true));
        assert_eq!(eval_input("1 > 2"), Object::Bool(false));
        assert_eq!(eval_input("1 < 1"), Object::Bool(false));
        assert_eq!(eval_input("1 > 1"), Object::Bool(false));
        assert_eq!(eval_input("1 == 1"), Object::Bool(true));
        assert_eq!(eval_input("1 != 1"), Object::Bool(false));
        assert_eq!(eval_input("1 == 2"), Object::Bool(false));
        assert_eq!(eval_input("1 != 2"), Object::Bool(true));
        assert_eq!(eval_input("true == true"), Object::Bool(true));
        assert_eq!(eval_input("false == false"), Object::Bool(true));
        assert_eq!(eval_input("true == false"), Object::Bool(false));
        assert_eq!(eval_input("true != false"), Object::Bool(true));
        assert_eq!(eval_input("false != true"), Object::Bool(true));
        assert_eq!(eval_input("(1 < 2) == true"), Object::Bool(true));
        assert_eq!(eval_input("(1 < 2) == false"), Object::Bool(false));
        assert_eq!(eval_input("(1 > 2) == true"), Object::Bool(false));
        assert_eq!(eval_input("(1 > 2) == false"), Object::Bool(true));
    }

    #[test]
    fn if_else_expressions() {
        assert_eq!(eval_input("if (true) { 10 }"), Object::Int(10));
        assert_eq!(eval_input("if (false) { 10 }"), Object::Null);
        assert_eq!(eval_input("if (1) { 10 }"), Object::Int(10));
        assert_eq!(eval_input("if (1 < 2) { 10 }"), Object::Int(10));
        assert_eq!(eval_input("if (1 > 2) { 10 }"), Object::Null);
        assert_eq!(eval_input("if (1 > 2) { 10 } else { 20 }"), Object::Int(20));
        assert_eq!(eval_input("if (1 < 2) { 10 } else { 20 }"), Object::Int(10));
    }

    #[test]
    fn return_stms() {
        assert_eq!(eval_input("return 10;"), Object::Int(10));
        assert_eq!(eval_input("return 10; 9;"), Object::Int(10));
        assert_eq!(eval_input("return 2 * 5; 9;"), Object::Int(10));
        assert_eq!(eval_input("9; return 2 * 5; 9;"), Object::Int(10));
        assert_eq!(
            eval_input("if (10 > 1) { if (10 > 1) { return 10; } return 1; }"),
            Object::Int(10)
        );
    }

    #[test]
    fn let_stmts() {
        assert_eq!(eval_input("let a = 5; a;"), Object::Int(5));
        assert_eq!(eval_input("let a = 5 * 5; a;"), Object::Int(25));
        assert_eq!(eval_input("let a = 5; let b = a; b;"), Object::Int(5));
        assert_eq!(
            eval_input("let a = 5; let b = a; let c = a + b + 5; c;"),
            Object::Int(15)
        );
    }

    // #[test]
    // fn fn_calls() {
    //     assert_eq!(
    //         eval_input("let identity = fn(x) { x; }; identity(5);"),
    //         Object::Int(5)
    //     );
    //     assert_eq!(
    //         eval_input("let identity = fn(x) { return x; }; identity(5);"),
    //         Object::Int(5)
    //     );
    //     assert_eq!(
    //         eval_input("let double = fn(x) { x * 2; }; double(5);"),
    //         Object::Int(10)
    //     );
    //     assert_eq!(
    //         eval_input("let add = fn(x, y) { x + y; }; add(5, 5);"),
    //         Object::Int(10)
    //     );
    //     assert_eq!(
    //         eval_input("let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));"),
    //         Object::Int(20)
    //     );
    //     assert_eq!(eval_input("fn(x) { x; }(5)"), Object::Int(5));
    // }
}
