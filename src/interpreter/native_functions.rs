use std::{io::Write, sync::{Arc, Mutex}, time::Duration};

use tokio::{io::AsyncReadExt, net::TcpStream, stream, time::sleep};

use crate::error::{InterpreterError, InterpreterResult, RuntimeErrorKind};
use super::{enviroment::Environment, native::NativeFunction, value::Value};

impl Environment {
    pub fn register_native_functions(&mut self) {
        self.register_system_functions();
        self.register_io_functions();
        self.register_conversion_functions();
        self.register_async_functions();
    }

    fn register_system_functions(&mut self) {
        self.define_native("exit", 1, |args| {
            if let Value::Number(code) = args[0] {
                std::process::exit(code as i32);
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                ))
            }
        });
        self.define_native("random", 0, |_args| {
            Ok(Value::Number(rand::random::<f64>()))
        });
        self.define_native("clock", 0, |_args| {
            Ok(Value::Number(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ))
        });
        self.define_native("typeOf", 1, |args| {
            Ok(Value::String(args[0].get_type()))
        });
        self.define_native("assert", 2, |args| {
            if args[0] == args[1] {
                Ok(Value::Nil)
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::AssertionFailed,
                ))
            }
        });
    }

    fn register_io_functions(&mut self) {
        self.define_native("readFile", 1, |args| {
            if let Value::String(filename) = &args[0] {
                match std::fs::read_to_string(filename) {
                    Ok(contents) => Ok(Value::String(contents)),
                    Err(e) => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::IoError(e.to_string())
                    ))
                }
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0)
                ))
            }
        });
        self.define_native("writeFile", 2, |args| {
            if let (Value::String(filename), Value::String(contents)) = (&args[0], &args[1]) {
                match std::fs::write(filename, contents) {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::IoError(e.to_string())
                    ))
                }
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0)
                ))
            }
        });
        self.define_native("appendFile", 2, |args| {
            if let (Value::String(filename), Value::String(contents)) = (&args[0], &args[1]) {
                use std::fs::OpenOptions;
                use std::io::Write;
        
                match OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(filename)
                    .and_then(|mut file| file.write_all(contents.as_bytes()))
                {
                    Ok(_) => Ok(Value::Nil),
                    Err(e) => Err(InterpreterError::runtime_error(
                        RuntimeErrorKind::IoError(e.to_string())
                    ))
                }
            } else {
                Err(InterpreterError::runtime_error(
                    RuntimeErrorKind::InvalidArgumentType(0)
                ))
            }
        });

        self.define_native("print", 1, |args| {
            println!("{}", args[0]);
            Ok(Value::Nil)
        });
        self.define_native("input", 0, |_args| {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    // Trim the trailing newline
                    input = input.trim().to_string();
                    Ok(Value::String(input))
                }
                Err(_) => Err(InterpreterError::runtime_error(
                    RuntimeErrorKind::RuntimeError(
                        0,
                        "Failed to read input".to_string(),
                    ),
                )),
            }
        });
        self.define_native("einput", 1, |args| {
            match &args[0] {
                Value::String(prompt) => {
                    print!("{}", prompt);
                    // Ensure the prompt is displayed before reading input
                    if let Err(_) = std::io::stdout().flush() {
                        return Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::RuntimeError(
                                0,
                                "Failed to flush stdout".to_string(),
                            ),
                        ));
                    }

                    let mut input = String::new();
                    match std::io::stdin().read_line(&mut input) {
                        Ok(_) => {
                            input = input.trim().to_string();
                            Ok(Value::String(input))
                        }
                        Err(_) => Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::RuntimeError(
                                0,
                                "Failed to read input".to_string(),
                            ),
                        )),
                    }
                }
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
    }

    fn register_conversion_functions(&mut self) {
        self.define_native("toString", 1, |args| {
            let value = &args[0];
            let string_value = match value {
                Value::Number(n) => n.to_string(),
                Value::String(s) => s.clone(),
                Value::Boolean(b) => b.to_string(),
                Value::Nil => "nil".to_string(),
                Value::Function(name, _, _) => format!("<fn {}>", name),
                Value::NativeFunction(nf) => format!("<native fn {}>", nf.name),
                Value::Class(name, _) => format!("<class {}>", name),
                Value::Instance(name, _) => format!("<instance {}>", name),
                Value::Array(arr) => {
                    let mut result = "".to_string();
                    for (i, v) in arr.iter().enumerate() {
                        if i > 0 {
                            result.push_str(",");
                        }
                        result.push_str(&v.to_string());
                    }
                    result
                },
                Value::Dictionary(d) => {
                    let mut result = "".to_string();
                    for (i, (k, v)) in d.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&k);
                        result.push_str(": ");
                        result.push_str(&v.to_string());
                    }
                    result
                }
                Value::Socket(_) => "socket".to_string(),
                Value::Server(_) => "server".to_string(),
                Value::AsyncFunction(name, _, _) => format!("<async fn {}>", name),
                Value::Promise(_) => "promise".to_string(),
                // Add other value types as needed
            };
            Ok(Value::String(string_value))
        });
        // toNumber function - attempts to convert a value to a number
        self.define_native("toNumber", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(*n)),
                Value::String(s) => match s.parse::<f64>() {
                    Ok(num) => Ok(Value::Number(num)),
                    Err(_) => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::RuntimeError(
                            0,
                            "Could not convert string to number".to_string(),
                        ),
                    )),
                },
                Value::Boolean(b) => Ok(Value::Number(if *b { 1.0 } else { 0.0 })),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        // toBool function - converts value to boolean
        self.define_native("toBool", 1, |args| {
            let value = &args[0];
            let bool_value = match value {
                Value::Boolean(b) => *b,
                Value::Number(n) => *n != 0.0,
                Value::String(s) => !s.is_empty(),
                Value::Nil => false,
                _ => true,
                // Add other value types as needed
            };
            Ok(Value::Boolean(bool_value))
        });
    }
    fn register_async_functions(&mut self){
        self.define_native("delay", 1, |args| {
            let duration = match args[0] {
                Value::Number(n) => Duration::from_secs_f64(n),
                _ => return Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            };
            let future = tokio::spawn(async move {
                sleep(duration).await;
                Ok(Value::Nil)
            });
            Ok(Value::create_promise(future))
        });
    }
}