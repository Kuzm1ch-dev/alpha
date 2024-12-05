use crate::error::{InterpreterError, InterpreterResult};

use super::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeFunction {
    pub name: String,
    arity: usize,
    func: fn(&Vec<Value>) -> InterpreterResult<Value>,
}

impl NativeFunction {
    pub fn new(
        name: &str,
        arity: usize,
        func: fn(&Vec<Value>) -> InterpreterResult<Value>,
    ) -> Self {
        NativeFunction {
            name: name.to_string(),
            arity,
            func,
        }
    }

    pub fn call(&self, args: &Vec<Value>) -> InterpreterResult<Value> {
        if args.len() != self.arity {
            return Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::InvalidParametsCount(self.arity),
            ));
        }
        (self.func)(args)
    }
}

