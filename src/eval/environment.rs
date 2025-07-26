use super::object::Object;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Default)]
pub struct Environment {
    variables: Rc<RefCell<HashMap<String, Object>>>,
}

impl Environment {
    pub fn get_var(&self, var_name: impl AsRef<str>) -> Object {
        match self.variables.borrow().get(var_name.as_ref()) {
            Some(obj) => obj.to_owned(),
            None => Object::Null,
        }
    }

    pub fn set_var(&self, var_name: impl Into<String>, obj: impl Into<Object>) -> Object {
        let obj = obj.into();
        self.variables
            .borrow_mut()
            .entry(var_name.into())
            .and_modify(|curr| *curr = obj.clone())
            .or_insert(obj)
            .to_owned()
    }
}
