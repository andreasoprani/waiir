mod builtin;
mod environment;
mod object;

use crate::{Expression, InfixOperator, Parser, PrefixOperator, Program, Statement};
use anyhow::{Result, bail};
use builtin::BuiltinFunction;
pub use environment::Environment;
use object::{HashMapKey, Object};
use std::{collections::HashMap, rc::Rc};

pub trait Eval {
    fn eval(self, env: Rc<Environment>) -> Result<Object>;
}

impl Eval for Program {
    fn eval(self, env: Rc<Environment>) -> Result<Object> {
        let mut result = Object::Null;
        for statement in self.statements {
            result = statement.eval(Rc::clone(&env))?;
            if let Object::Return(_res) = result {
                return Ok(*_res);
            }
        }
        Ok(result)
    }
}

impl Eval for Statement {
    fn eval(self, env: Rc<Environment>) -> Result<Object> {
        match self {
            Statement::Expr(expr) => expr.eval(Rc::clone(&env)),
            Statement::Block(stmts) => {
                let mut result: Object = Object::Null;
                for statement in stmts {
                    result = statement.eval(Rc::clone(&env))?;
                    if matches!(result, Object::Return(_)) {
                        return Ok(result);
                    }
                }
                Ok(result)
            }
            Statement::Let { name, value } => {
                let obj = value.eval(Rc::clone(&env))?;
                Ok(env.set(name, obj))
            }
            Statement::Return { value } => {
                Ok(Object::Return(Box::new(value.eval(Rc::clone(&env))?)))
            }
        }
    }
}

impl Eval for Expression {
    fn eval(self, env: Rc<Environment>) -> Result<Object> {
        Ok(match self {
            Expression::Bool(value) => Object::Bool(value),
            Expression::Int(value) => Object::Int(value),
            Expression::String(string) => Object::String(string),
            Expression::Ident(ident) if ident == "null" => Object::Null,
            Expression::Ident(ident) => match ident.as_str() {
                "len" => Object::Builtin(BuiltinFunction::Len),
                "first" => Object::Builtin(BuiltinFunction::First),
                "last" => Object::Builtin(BuiltinFunction::Last),
                "rest" => Object::Builtin(BuiltinFunction::Rest),
                "push" => Object::Builtin(BuiltinFunction::Push),
                _ => env.get(ident),
            },
            Expression::Array(content) => Object::Array(
                content
                    .iter()
                    .map(|e| e.to_owned().eval(Rc::clone(&env)))
                    .collect::<Result<Vec<Object>>>()?,
            ),
            Expression::Hash(hash_vec) => {
                let mut _map = HashMap::new();
                for (k, v) in hash_vec {
                    let key_obj = k.eval(Rc::clone(&env))?;
                    let value = v.eval(Rc::clone(&env))?;
                    let key = match key_obj {
                        Object::Int(key) => HashMapKey::Int(key),
                        Object::String(key) => HashMapKey::String(key),
                        Object::Bool(key) => HashMapKey::Bool(key),
                        _ => {
                            bail!("Invalid object type for an hash key, must be int, str or bool!",);
                        }
                    };
                    _map.insert(key, value);
                }
                Object::Hash(_map)
            }
            Expression::Prefix { operator, right } => {
                Expression::eval_prefix(operator, right.eval(Rc::clone(&env))?)?
            }
            Expression::Infix {
                operator,
                left,
                right,
            } => Expression::eval_infix(
                operator,
                left.eval(Rc::clone(&env))?,
                right.eval(Rc::clone(&env))?,
            )?,
            Expression::Cond { cond, then_, else_ } => {
                let evaluated_cond = cond.eval(Rc::clone(&env))?.to_bool();

                if evaluated_cond {
                    Statement::Block(then_).eval(env)?
                } else if let Some(stmts) = else_ {
                    Statement::Block(stmts).eval(env)?
                } else {
                    Object::Null
                }
            }
            Expression::Func { args, body } => Object::Function {
                parameters: args,
                body,
                environment: Environment::init_with_outer(Rc::clone(&env)),
            },
            Expression::Call { func, args } => {
                let func_to_call = func.eval(Rc::clone(&env))?;

                let arguments = args
                    .into_iter()
                    .map(|arg| arg.eval(Rc::clone(&env)))
                    .collect::<Result<Vec<Object>>>()?;

                match func_to_call {
                    Object::Function {
                        parameters,
                        body,
                        environment: func_env,
                    } => {
                        let func_env = Rc::new(Environment::init_with_outer(Rc::new(func_env)));

                        let n_params = parameters.len();
                        let n_args = arguments.len();
                        if n_params != n_args {
                            bail!(
                                "Invalid function call argument counts, {n_params} requested, {n_args} provided.",
                            );
                        }

                        for (name, val) in parameters.iter().zip(arguments) {
                            func_env.set(name, val);
                        }

                        let evaluated_func = Statement::Block(body).eval(Rc::clone(&func_env))?;
                        if let Object::Return(obj) = evaluated_func {
                            *obj
                        } else {
                            evaluated_func
                        }
                    }
                    Object::Builtin(builtin_fn) => builtin_fn.call(arguments)?,
                    _ => bail!("{func_to_call} is not a function"),
                }
            }
        })
    }
}

impl Expression {
    fn eval_prefix(operator: PrefixOperator, right: Object) -> Result<Object> {
        match operator {
            PrefixOperator::Neg => Self::eval_neg(right),
            PrefixOperator::Not => Ok(Object::Bool(!right.to_bool())),
        }
    }

    fn eval_neg(right: Object) -> Result<Object> {
        match right {
            Object::Int(value) => Ok(Object::Int(-value)),
            Object::Return(value) => Self::eval_neg(*value),
            _ => bail!("{right} cannot be negated!"),
        }
    }

    fn eval_infix(operator: InfixOperator, left: Object, right: Object) -> Result<Object> {
        match (left, right, operator) {
            (Object::Null, Object::Null, _) => Ok(Object::Null),
            (Object::Bool(l), Object::Bool(r), InfixOperator::Eq) => Ok(Object::Bool(l == r)),
            (Object::Bool(l), Object::Bool(r), InfixOperator::NotEq) => Ok(Object::Bool(l != r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Add) => Ok(Object::Int(l + r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Sub) => Ok(Object::Int(l - r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Mul) => Ok(Object::Int(l * r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Div) => Ok(Object::Int(l / r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Eq) => Ok(Object::Bool(l == r)),
            (Object::Int(l), Object::Int(r), InfixOperator::NotEq) => Ok(Object::Bool(l != r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Gt) => Ok(Object::Bool(l > r)),
            (Object::Int(l), Object::Int(r), InfixOperator::Lt) => Ok(Object::Bool(l < r)),
            (Object::String(l), Object::String(r), InfixOperator::Add) => {
                Ok(Object::String(l + &r))
            }
            (Object::Array(content), Object::Int(index), InfixOperator::Index) => {
                if index < 0 || index >= content.len().try_into().unwrap() {
                    return Ok(Object::Null);
                }
                Ok(content[index as usize].clone())
            }
            (Object::Hash(map), key_object, InfixOperator::Index) => {
                let value = match key_object {
                    Object::Bool(key) => map.get(&HashMapKey::Bool(key)),
                    Object::Int(key) => map.get(&HashMapKey::Int(key)),
                    Object::String(key) => map.get(&HashMapKey::String(key)),
                    _ => {
                        bail!(
                            "Invalid operation ({}) between {} and {key_object}!",
                            Object::Hash(map),
                            InfixOperator::Index
                        );
                    }
                };
                Ok(match value {
                    Some(v) => v.clone(),
                    None => Object::Null,
                })
            }
            (l, r, op) => {
                bail!("Invalid operation ({op}) between {l} and {r}!");
            }
        }
    }
}

pub fn eval_with_env(input: &str, env: Rc<Environment>) -> Result<Object> {
    Parser::init(input).parse_program()?.eval(env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_eval(input: &str, expected: Object) {
        let env = Environment::default();
        let output = eval_with_env(input, Rc::new(env)).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn integer_expression() {
        assert_eval("5", Object::Int(5));
        assert_eval("10", Object::Int(10));
        assert_eval("-5", Object::Int(-5));
        assert_eval("-10", Object::Int(-10));
        assert_eval("5 + 5 + 5 + 5 - 10", Object::Int(10));
        assert_eval("2 * 2 * 2 * 2 * 2", Object::Int(32));
        assert_eval("-50 + 100 + -50", Object::Int(0));
        assert_eval("5 * 2 + 10", Object::Int(20));
        assert_eval("5 + 2 * 10", Object::Int(25));
        assert_eval("20 + 2 * -10", Object::Int(0));
        assert_eval("50 / 2 * 2 + 10", Object::Int(60));
        assert_eval("2 * (5 + 10)", Object::Int(30));
        assert_eval("3 * 3 * 3 + 10", Object::Int(37));
        assert_eval("3 * (3 * 3) + 10", Object::Int(37));
        assert_eval("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Int(50));
    }

    #[test]
    fn boolean_expression() {
        assert_eval("true", Object::Bool(true));
        assert_eval("false", Object::Bool(false));
        assert_eval("!true", Object::Bool(false));
        assert_eval("!false", Object::Bool(true));
        assert_eval("!5", Object::Bool(false));
        assert_eval("!!true", Object::Bool(true));
        assert_eval("!!false", Object::Bool(false));
        assert_eval("!!5", Object::Bool(true));
        assert_eval("!null", Object::Bool(true));
        assert_eval("!!null", Object::Bool(false));
        assert_eval("1 < 2", Object::Bool(true));
        assert_eval("1 > 2", Object::Bool(false));
        assert_eval("1 < 1", Object::Bool(false));
        assert_eval("1 > 1", Object::Bool(false));
        assert_eval("1 == 1", Object::Bool(true));
        assert_eval("1 != 1", Object::Bool(false));
        assert_eval("1 == 2", Object::Bool(false));
        assert_eval("1 != 2", Object::Bool(true));
        assert_eval("true == true", Object::Bool(true));
        assert_eval("false == false", Object::Bool(true));
        assert_eval("true == false", Object::Bool(false));
        assert_eval("true != false", Object::Bool(true));
        assert_eval("false != true", Object::Bool(true));
        assert_eval("(1 < 2) == true", Object::Bool(true));
        assert_eval("(1 < 2) == false", Object::Bool(false));
        assert_eval("(1 > 2) == true", Object::Bool(false));
        assert_eval("(1 > 2) == false", Object::Bool(true));
    }

    #[test]
    fn if_else_expressions() {
        assert_eval("if (true) { 10 }", Object::Int(10));
        assert_eval("if (false) { 10 }", Object::Null);
        assert_eval("if (1) { 10 }", Object::Int(10));
        assert_eval("if (1 < 2) { 10 }", Object::Int(10));
        assert_eval("if (1 > 2) { 10 }", Object::Null);
        assert_eval("if (1 > 2) { 10 } else { 20 }", Object::Int(20));
        assert_eval("if (1 < 2) { 10 } else { 20 }", Object::Int(10));
    }

    #[test]
    fn return_stms() {
        assert_eval("return 10;", Object::Int(10));
        assert_eval("return 10; 9;", Object::Int(10));
        assert_eval("return 2 * 5; 9;", Object::Int(10));
        assert_eval("9; return 2 * 5; 9;", Object::Int(10));
        assert_eval(
            "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
            Object::Int(10),
        );
    }

    #[test]
    fn let_stmts() {
        assert_eval("let a = 5; a;", Object::Int(5));
        assert_eval("let a = 5 * 5; a;", Object::Int(25));
        assert_eval("let a = 5; let b = a; b;", Object::Int(5));
        assert_eval(
            "let a = 5; let b = a; let c = a + b + 5; c;",
            Object::Int(15),
        );
    }

    #[test]
    fn fn_calls() {
        assert_eval("let identity = fn(x) { x; }; identity(5);", Object::Int(5));
        assert_eval(
            "let identity = fn(x) { return x; }; identity(5);",
            Object::Int(5),
        );
        assert_eval("let double = fn(x) { x * 2; }; double(5);", Object::Int(10));
        assert_eval("let add = fn(x, y) { x + y; }; add(5, 5);", Object::Int(10));
        assert_eval(
            "let add = fn(x, y) { x + y; }; add(5 + 5, add(5, 5));",
            Object::Int(20),
        );
        assert_eval("fn(x) { x; }(5)", Object::Int(5));
    }

    #[test]
    fn closure() {
        assert_eval(
            " \n\
                let newAdder = fn(x) { \n\
                    fn(y) { x + y }; \n\
                }; \n\
                let addTwo = newAdder(2); \n\
                addTwo(2)
                ",
            Object::Int(4),
        )
    }

    #[test]
    fn string_expression() {
        assert_eval(
            "\"Hello World!\"",
            Object::String(String::from("Hello World!")),
        );
        assert_eval(
            "\"Hello\" + \" \" + \"World!\"",
            Object::String(String::from("Hello World!")),
        );
        assert_eval("!\"Hello World!\"", Object::Bool(false));
        assert_eval("!\"\"", Object::Bool(true));
    }

    #[test]
    fn builtin_functions() {
        assert_eval("len(\"\")", Object::Int(0));
        assert_eval("len(\"four\")", Object::Int(4));
        assert_eval("len(\"hello world\")", Object::Int(11));
    }

    #[test]
    fn array_literals() {
        assert_eval(
            "[1, 2 * 2, 3 + 3]",
            Object::Array(vec![Object::Int(1), Object::Int(4), Object::Int(6)]),
        );
    }

    #[test]
    fn index_operations() {
        assert_eval("[1, 2, 3][0]", Object::Int(1));
        assert_eval("[1, 2, 3][1]", Object::Int(2));
        assert_eval("[1, 2, 3][2]", Object::Int(3));
        assert_eval("let i = 0; [1][i]", Object::Int(1));
        assert_eval("[1, 2, 3][1 + 1]", Object::Int(3));
        assert_eval("let myArray = [1, 2, 3]; myArray[2]", Object::Int(3));
        assert_eval(
            "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2]",
            Object::Int(6),
        );
        assert_eval(
            "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            Object::Int(2),
        );
        assert_eval("[1, 2, 3][3]", Object::Null);
        assert_eval("[1, 2, 3][-1]", Object::Null);
    }

    #[test]
    fn map_impl() {
        let input = "
            let map = fn(arr, f) { \n\
                let iter = fn(arrin, accumulated) { \n\
                    if (len(arrin) == 0) { \n\
                        accumulated \n\
                    } else { \n\
                        iter(rest(arrin), push(accumulated, f(first(arrin)))); \n\
                    } \n\
                }; \n\
                iter(arr, []); \n\
            }; \n\
            let a = [1, 2, 3, 4];
            let double = fn(x) { x * 2 };
            map(a, double)
        ";

        assert_eval(
            input,
            Object::Array(vec![
                Object::Int(2),
                Object::Int(4),
                Object::Int(6),
                Object::Int(8),
            ]),
        );
    }

    #[test]
    fn reduce_impl() {
        let input = "
            let reduce = fn(arr, initial, f) {
                let iter = fn(arrin, result) {
                    if (len(arrin) == 0) {
                        result
                    } else {
                        iter(rest(arrin), f(result, first(arrin)));
                    }
                };
                iter(arr, initial);
            };
            let sum = fn(arr) {
                reduce(arr, 0, fn(initial, el) { initial + el });
            };
            sum([1,2,3,4,5]);
        ";

        assert_eval(input, Object::Int(15));
    }

    #[test]
    fn hash_literals() {
        let input = "
            let two = \"two\"; \n\
            { \n\
                \"one\": 10 - 9,\n\
                two: 1 + 1,\n\
                \"thr\" + \"ee\": 6 / 2,\n\
                4: 4,\n\
                true: 5,\n\
                false: 6\n\
            }\n\
        ";

        assert_eval(
            input,
            Object::Hash(HashMap::from([
                (HashMapKey::String(String::from("one")), Object::Int(1)),
                (HashMapKey::String(String::from("two")), Object::Int(2)),
                (HashMapKey::String(String::from("three")), Object::Int(3)),
                (HashMapKey::Int(4), Object::Int(4)),
                (HashMapKey::Bool(true), Object::Int(5)),
                (HashMapKey::Bool(false), Object::Int(6)),
            ])),
        );
    }

    #[test]
    fn hash_index_expressions() {
        assert_eval("{\"foo\": 5}[\"foo\"]", Object::Int(5));
        assert_eval("{\"foo\": 5}[\"bar\"]", Object::Null);
        assert_eval("let key = \"foo\"; {\"foo\": 5}[key]", Object::Int(5));
        assert_eval("{}[\"foo\"]", Object::Null);
        assert_eval("{5: 5}[5]", Object::Int(5));
        assert_eval("{true: 5}[true]", Object::Int(5));
        assert_eval("{false: 5}[false]", Object::Int(5));
    }
}
