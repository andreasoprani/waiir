use crate::eval::{HashMapKey, Object};
use anyhow::{Result, bail};
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
    pub fn call(&self, args: Vec<Object>) -> Result<Object> {
        match &self {
            BuiltinFunction::Len => self.call_len(args),
            BuiltinFunction::First => self.call_first(args),
            BuiltinFunction::Last => self.call_last(args),
            BuiltinFunction::Rest => self.call_rest(args),
            BuiltinFunction::Push => self.call_push(args),
        }
    }

    fn call_len(&self, args: Vec<Object>) -> Result<Object> {
        if args.len() != 1 {
            bail!(
                "Builtin function `len` expects 1 arg, found {}.",
                args.len()
            );
        }
        Ok(match args.first() {
            Some(Object::String(string)) => Object::Int(string.len().try_into().unwrap()),
            Some(Object::Array(content)) => Object::Int(content.len().try_into().unwrap()),
            Some(Object::Hash(hashmap)) => Object::Int(hashmap.len().try_into().unwrap()),
            Some(o) => bail!(
                "Invalid argument for builtin function `len`, expected string or array, found {o}"
            ),
            None => unreachable!(),
        })
    }

    fn call_first(&self, args: Vec<Object>) -> Result<Object> {
        if args.len() != 1 {
            bail!(
                "Builtin function `first` expects 1 arg, found {}.",
                args.len()
            );
        }
        let arg = args.first().unwrap();
        Ok(match arg {
            Object::String(string) if string.is_empty() => Object::Null,
            Object::String(string) => Object::String(string.chars().next().unwrap().into()),
            Object::Array(content) if content.is_empty() => Object::Null,
            Object::Array(content) => content.first().unwrap().to_owned(),
            o => bail!(
                "Invalid argument for builtin function `first`, expected string or array, found {o}"
            ),
        })
    }

    fn call_last(&self, args: Vec<Object>) -> Result<Object> {
        if args.len() != 1 {
            bail!(
                "Builtin function `last` expects 1 arg, found {}.",
                args.len()
            );
        }
        let arg = args.first().unwrap();
        Ok(match arg {
            Object::String(string) if string.is_empty() => Object::Null,
            Object::String(string) => Object::String(string.chars().last().unwrap().into()),
            Object::Array(content) if content.is_empty() => Object::Null,
            Object::Array(content) => content.last().unwrap().to_owned(),
            o => bail!(
                "Invalid argument for builtin function `last`, expected string or array, found {o}"
            ),
        })
    }

    fn call_rest(&self, args: Vec<Object>) -> Result<Object> {
        if args.len() != 1 {
            bail!(
                "Builtin function `rest` expects 1 arg, found {}.",
                args.len()
            );
        }
        let arg = args.first().unwrap();

        Ok(match arg {
            Object::String(string) if string.is_empty() => Object::Null,
            Object::String(string) if string.len() == 1 => Object::String("".into()),
            Object::String(string) => Object::String(string[1..].into()),
            Object::Array(content) if content.is_empty() => Object::Null,
            Object::Array(content) if content.len() == 1 => Object::Array(vec![]),
            Object::Array(content) => Object::Array(content[1..].into()),
            o => bail!(
                "Invalid argument for builtin function `rest`, expected string or array, found {o}"
            ),
        })
    }

    fn call_push(&self, args: Vec<Object>) -> Result<Object> {
        if args.len() < 2 {
            bail!(
                "Builtin function `push` expects 2 args, found {}.",
                args.len()
            );
        }
        let (arg1, arg2) = (&args[0], &args[1]);

        Ok(match arg1 {
            Object::String(string1) => match arg2 {
                Object::String(string2) => Object::String(format!("{string1}{string2}")),
                _ => bail!(
                    "Invalid second argument for builtin function `push`, expected string or array, found {arg2}"
                ),
            },
            Object::Array(content) => {
                let mut new_content = content.clone();
                new_content.push(arg2.to_owned());
                Object::Array(new_content)
            }
            Object::Hash(content1) => {
                let mut new_content = content1.clone();
                match arg2 {
                    Object::Array(content2) if content2.len() == 2 => {
                        new_content.insert(
                            match content2[0].clone() {
                                Object::Bool(c) => HashMapKey::Bool(c),
                                Object::Int(c) => HashMapKey::Int(c),
                                Object::String(c) => HashMapKey::String(c),
                                _ => bail!(
                                    "Invalid object type for an hash key, must be int, str or bool!"
                                ),
                            },
                            content2[1].clone(),
                        );
                    }
                    Object::Array(_) => bail!(
                        "Invalid second argument for builtin function `push`, expected array with 2 elements"
                    ),
                    Object::Hash(content2) => {
                        for (k, v) in content2 {
                            new_content.insert(k.clone(), v.clone());
                        }
                    }
                    _ => bail!(
                        "Invalid second argument for builtin function `push`, expected array with 2 elements or another hashmap"
                    ),
                }
                Object::Hash(new_content)
            }
            o => bail!(
                "Invalid first argument for builtin function `push`, expected string or array, found {o}"
            ),
        })
    }
}
