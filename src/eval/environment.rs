use super::object::Object;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct Environment {
    variables: Rc<RefCell<HashMap<String, Object>>>,
    outer: Option<Rc<Environment>>,
}

impl Environment {
    pub fn get(&self, var_name: impl AsRef<str>) -> Object {
        match self.variables.borrow().get(var_name.as_ref()) {
            Some(obj) => obj.to_owned(),
            None => match &self.outer {
                Some(env) => env.get(var_name),
                None => Object::Null,
            },
        }
    }

    pub fn set(&self, var_name: impl Into<String>, obj: impl Into<Object>) -> Object {
        let obj = obj.into();
        self.variables
            .borrow_mut()
            .entry(var_name.into())
            .and_modify(|curr| *curr = obj.clone())
            .or_insert(obj)
            .to_owned()
    }

    pub fn init_with_outer(outer: Rc<Self>) -> Self {
        Self {
            outer: Some(outer.clone()),
            ..Default::default()
        }
    }
}
