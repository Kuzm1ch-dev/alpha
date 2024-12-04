use std::{collections::HashMap, fmt, sync::{Arc, Mutex}};

use crate::parser::Expr;

use super::{enviroment::Environment, native::NativeFunction};

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    NativeFunction(NativeFunction),
    Function(String, Vec<String>, Box<Expr>),
    Class(String, HashMap<String, Value>), // (class name, methods)
    Instance(String, Arc<Mutex<Environment>>), // (class name, fields)
    Array(Vec<Value>),
    Dictionary(HashMap<String, Value>),
    Nil,
}

impl Value {
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Nil => "nil".to_string(),
            Value::Function(name, _, _) => name.clone(),
            Value::NativeFunction(nf) => nf.name.clone(),
            Value::Class(name, _) => name.clone(),
            Value::Instance(name, _) => name.clone(),
            Value::Array(arr) => {
                let mut s = String::new();
                s.push('[');
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&v.to_string());
                }
                s.push(']');
                s
            },
            Value::Dictionary(d) => {
                let mut s = String::new();
                s.push('{');
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        s.push_str(", ");
                    }
                    s.push_str(&k);
                    s.push_str(": ");
                    s.push_str(&v.to_string());
                }
                s.push('}');
                s
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Function(name, _, _) => write!(f, "<fn {}>", name),
            Value::NativeFunction(nf) => write!(f, "<native fn {}>", nf.name),
            Value::Class(name, _) => write!(f, "<class {}>", name),
            Value::Instance(name, values) => write!(f, "<instance {} {:#?}>", name, values),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            },
            Value::Dictionary(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
        }
    }
}