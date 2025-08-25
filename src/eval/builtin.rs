use crate::eval::Object;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BuiltinFunction {
    Len,
    First,
    Last,
    Rest,
    Push,
}

impl fmt::Display for BuiltinFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BuiltinFunction::Len => write!(f, "len"),
            BuiltinFunction::First => write!(f, "first"),
            BuiltinFunction::Last => write!(f, "last"),
            BuiltinFunction::Rest => write!(f, "rest"),
            BuiltinFunction::Push => write!(f, "push"),
        }
    }
}

impl BuiltinFunction {
    pub fn call(&self, args: Vec<Object>) -> Object {
        match &self {
            BuiltinFunction::Len => self.call_len(args),
            BuiltinFunction::First => self.call_first(args),
            BuiltinFunction::Last => self.call_last(args),
            BuiltinFunction::Rest => self.call_rest(args),
            BuiltinFunction::Push => self.call_push(args),
        }
    }

    fn call_len(&self, args: Vec<Object>) -> Object {
        if args.len() != 1 {
            {
                println!(
                    "Builtin function `len` expects 1 arg, found {}.",
                    args.len()
                );
                panic!()
            }
        }
        match args.first() {
            Some(Object::String(string)) => Object::Int(string.len().try_into().unwrap()),
            Some(Object::Array(content)) => Object::Int(content.len().try_into().unwrap()),
            Some(o) => {
                println!(
                    "Invalid argument for builtin function `len`, expected string or array, found {o}"
                );
                panic!();
            }
            None => unreachable!(),
        }
    }

    fn call_first(&self, args: Vec<Object>) -> Object {
        if args.len() != 1 {
            {
                println!(
                    "Builtin function `first` expects 1 arg, found {}.",
                    args.len()
                );
                panic!()
            }
        }
        match args.first() {
            Some(Object::String(string)) if string.is_empty() => Object::Null,
            Some(Object::String(string)) => Object::String(string.chars().next().unwrap().into()),
            Some(Object::Array(content)) if content.is_empty() => Object::Null,
            Some(Object::Array(content)) => content.first().unwrap().to_owned(),
            Some(o) => {
                println!(
                    "Invalid argument for builtin function `first`, expected string or array, found {o}"
                );
                panic!();
            }
            None => unreachable!(),
        }
    }

    fn call_last(&self, args: Vec<Object>) -> Object {
        if args.len() != 1 {
            {
                println!(
                    "Builtin function `last` expects 1 arg, found {}.",
                    args.len()
                );
                panic!()
            }
        }
        match args.first() {
            Some(Object::String(string)) if string.is_empty() => Object::Null,
            Some(Object::String(string)) => Object::String(string.chars().last().unwrap().into()),
            Some(Object::Array(content)) if content.is_empty() => Object::Null,
            Some(Object::Array(content)) => content.last().unwrap().to_owned(),
            Some(o) => {
                println!(
                    "Invalid argument for builtin function `last`, expected string or array, found {o}"
                );
                panic!();
            }
            None => unreachable!(),
        }
    }

    fn call_rest(&self, args: Vec<Object>) -> Object {
        if args.len() != 1 {
            {
                println!(
                    "Builtin function `rest` expects 1 arg, found {}.",
                    args.len()
                );
                panic!()
            }
        }
        match args.first() {
            Some(Object::String(string)) if string.is_empty() => Object::Null,
            Some(Object::String(string)) if string.len() == 1 => Object::String("".into()),
            Some(Object::String(string)) => Object::String(string[1..].into()),
            Some(Object::Array(content)) if content.is_empty() => Object::Null,
            Some(Object::Array(content)) if content.len() == 1 => Object::Array(vec![]),
            Some(Object::Array(content)) => Object::Array(content[1..].into()),
            Some(o) => {
                println!(
                    "Invalid argument for builtin function `rest`, expected string or array, found {o}"
                );
                panic!();
            }
            None => unreachable!(),
        }
    }

    fn call_push(&self, args: Vec<Object>) -> Object {
        if args.len() != 2 {
            {
                println!(
                    "Builtin function `push` expects 2 args, found {}.",
                    args.len()
                );
                panic!()
            }
        }
        let (arg1, arg2) = (&args[0], &args[1]);

        match arg1 {
            Object::String(string1) => match arg2 {
                Object::String(string2) => Object::String(format!("{string1}{string2}")),
                _ => {
                    println!(
                        "Invalid second argument for builtin function `push`, expected string or array, found {arg2}"
                    );
                    panic!();
                }
            },
            Object::Array(content) => {
                let mut new_content = content.clone();
                new_content.push(arg2.to_owned());
                Object::Array(new_content)
            }
            o => {
                println!(
                    "Invalid first argument for builtin function `push`, expected string or array, found {o}"
                );
                panic!();
            }
        }
    }
}
