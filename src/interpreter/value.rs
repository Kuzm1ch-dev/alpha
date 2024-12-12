use std::{collections::HashMap, fmt::{self, Debug}, future::Future, pin::Pin, sync::{Arc, Mutex}};
use tokio::{net::{TcpListener, TcpSocket, TcpStream}, task::JoinHandle};
use crate::{error::{InterpreterError, InterpreterResult}, parser::Expr};

use super::{enviroment::Environment, native::NativeFunction, Interpreter};

#[derive(Clone)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    NativeFunction(NativeFunction),
    Promise(Arc<Mutex<PromiseState>>),
    Function(String, Vec<String>, Box<Expr>),
    AsyncFunction(String, Vec<String>, Box<Expr>),
    Class(String, HashMap<String, Value>),
    Instance(String, Arc<Mutex<Environment>>),
    Array(Vec<Value>),
    Dictionary(HashMap<String, Value>),
    Socket(Arc<Mutex<TcpStream>>),
    TlsSocket(Arc<Mutex<tokio_rustls::client::TlsStream<TcpStream>>>),
    Server(Arc<Mutex<TcpListener>>),
    Nil,
}



pub enum PromiseState {
    Pending(Pin<Box<dyn Future<Output = Result<Value, InterpreterError>>>>),
    Fulfilled(Value),
    Rejected(InterpreterError),
}

impl Value {
    pub fn create_promise(future: Pin<Box<dyn Future<Output = Result<Value, InterpreterError>>>>) -> Value {
        Value::Promise(Arc::new(Mutex::new(PromiseState::Pending(future))))
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Nil => write!(f, "nil"),
            Value::Function(name, _, _) => write!(f, "<function {}>", name),
            Value::AsyncFunction(name, _, _) => write!(f, "<async function {}>", name),
            Value::NativeFunction(nf) => write!(f, "<native function {}>", nf.name),
            Value::Class(name, _) => write!(f, "<class {}>", name),
            Value::Instance(name, _) => write!(f, "<instance {}>", name),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, "]")
            },
            Value::Dictionary(d) => {
                write!(f, "{{")?;
                for (i, (k, v)) in d.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {:?}", k, v)?;
                }
                write!(f, "}}")
            },
            Value::Socket(_) => write!(f, "<socket>"),
            Value::TlsSocket(_) => write!(f, "<tls socket>"),
            Value::Server(_) => write!(f, "<server>"),
            Value::Promise(_) => write!(f, "<promise>"),
        }
    }
}

impl PartialEq for Value{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Function(a, _, _), Value::Function(b, _, _)) => a == b,
            (Value::Class(a, _), Value::Class(b, _)) => a == b,
            (Value::Instance(a, a_en), Value::Instance(b, b_en)) => {
                if a != b {
                    return false;
                }
                let a_en = a_en.lock().unwrap();
                let b_en = b_en.lock().unwrap();
                a_en.get_values() == b_en.get_values()
            },
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Dictionary(a), Value::Dictionary(b)) => a == b,
            (Value::Socket(a), Value::Socket(b)) => Arc::ptr_eq(a, b),
            (Value::Server(a), Value::Server(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
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
            Value::Socket(_) => "socket".to_string(),
            Value::TlsSocket(_) => "tls socket".to_string(),
            Value::Server(_) => "server".to_string(),
            Value::AsyncFunction(name, _,_) => name.clone(),
            Value::Promise(_) => "promise".to_string(),
        }
    }

    pub fn get_type(&self) -> String {
        match self {
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Boolean(_) => "boolean".to_string(),
            Value::Nil => "nil".to_string(),
            Value::Function(_, _, _) => "function".to_string(),
            Value::AsyncFunction(_, _, _) => "async function".to_string(),
            Value::NativeFunction(_) => "native function".to_string(),
            Value::Class(_, _) => "class".to_string(),
            Value::Instance(_, _) => "instance".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Dictionary(_) => "dictionary".to_string(),
            Value::Socket(_) => "socket".to_string(),
            Value::TlsSocket(_) => "tls socket".to_string(),
            Value::Server(_) => "server".to_string(),
            Value::Promise(_) => "promise".to_string(),
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
            Value::AsyncFunction(name,_, _) => write!(f, "<async fn {}>", name),
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
            Value::Socket(_) => write!(f, "socket"),
            Value::TlsSocket(_) => write!(f, "tls socket"),
            Value::Server(_) => write!(f, "server"),
            Value::Promise(_) => write!(f, "promise"),
        }
    }
}