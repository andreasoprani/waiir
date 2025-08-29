use crate::Statement;
use crate::eval::Environment;
use crate::eval::builtin::BuiltinFunction;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum HashMapKey {
    Bool(bool),
    Int(i64),
    String(String),
}

impl fmt::Display for HashMapKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            HashMapKey::Bool(value) => write!(f, "{value}"),
            HashMapKey::Int(value) => write!(f, "{value}"),
            HashMapKey::String(value) => write!(f, "{value}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Object {
    Null,
    Int(i64),
    Bool(bool),
    String(String),
    Return(Box<Object>),
    Function {
        parameters: Vec<String>,
        body: Vec<Statement>,
        environment: Environment,
    },
    Builtin(BuiltinFunction),
    Array(Vec<Object>),
    Hash(HashMap<HashMapKey, Object>),
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Null => write!(f, "null"),
            Object::Int(value) => write!(f, "{value}"),
            Object::Bool(value) => write!(f, "{value}"),
            Object::String(value) => write!(f, "{value}"),
            Object::Return(value) => write!(f, "Return {value}"),
            Object::Function {
                parameters,
                body: _,
                environment: _,
            } => {
                let params = parameters.join(", ");
                write!(f, "fn({params}) {{...}}")
            }
            Object::Builtin(value) => write!(f, "Builtin function '{value}'"),
            Object::Array(content) => {
                write!(
                    f,
                    "[{}]",
                    content
                        .iter()
                        .map(|c| format!("{c}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Object::Hash(map) => {
                write!(
                    f,
                    "{{ {} }}",
                    map.iter()
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
        }
    }
}

impl Object {
    pub fn to_bool(&self) -> bool {
        match self {
            Object::Bool(value) => *value,
            Object::Int(value) => *value != 0,
            Object::String(value) => !value.is_empty(),
            Object::Null => false,
            Object::Return(value) => value.to_bool(),
            Object::Function { .. } => true,
            Object::Builtin(_) => true,
            Object::Array(content) => !content.is_empty(),
            Object::Hash(map) => !map.is_empty(),
        }
    }
}
