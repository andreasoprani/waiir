use crate::Expression;
use crate::eval::Object;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinFunction {
    Len,
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuiltinFunction::Len => write!(f, "len"),
        }
    }
}

impl BuiltinFunction {
    pub fn call(&self, args: Vec<Object>) -> Object {
        self.eval_args_len(args.len());
        match &self {
            BuiltinFunction::Len => self.call_len(args),
        }
    }

    fn call_len(&self, args: Vec<Object>) -> Object {
        match args.first() {
            Some(Object::String(string)) => Object::Int(string.len().try_into().unwrap()),
            Some(o) => {
                println!("Invalid argument for builtin function `len`, expected string, found {o}");
                panic!();
            }
            None => unreachable!(),
        }
    }

    fn eval_args_len(&self, n_args: usize) {
        match (&self, n_args) {
            (BuiltinFunction::Len, 1) => (),
            (BuiltinFunction::Len, n) => {
                println!("Builtin {} function expects 1 arg, found {}.", &self, n);
                panic!()
            }
        }
    }
}
