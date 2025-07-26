use crate::Statement;
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Object {
    Null,
    Int(i64),
    Bool(bool),
    Return(Box<Object>),
    // Function {
    //     parameters: Vec<String>,
    //     body: Vec<Statement>,
    //     environment: Environment,
    // },
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Null => write!(f, "null"),
            Object::Int(value) => write!(f, "{value}"),
            Object::Bool(value) => write!(f, "{value}"),
            Object::Return(value) => write!(f, "Return {value}"),
        }
    }
}

impl Object {
    pub fn to_bool(&self) -> bool {
        match self {
            Object::Bool(value) => *value,
            Object::Int(value) => *value != 0,
            Object::Null => false,
            Object::Return(value) => value.to_bool(),
        }
    }
}
