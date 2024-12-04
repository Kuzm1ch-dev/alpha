use std::{collections::HashMap, io::Write, path::{Path, PathBuf}, sync::{Arc, Mutex}};

use rustc_hash::FxHashMap;

use crate::{
    error::{InterpreterError, InterpreterResult},
    parser::Parser, tokenizer::Tokenizer,
};

use super::{native::NativeFunction, value::Value, Interpreter};

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub environment: Arc<Mutex<Environment>>,
    pub path: String,
}

#[derive(Debug, Clone)]
pub struct Environment {
    // Use FxHashMap instead of HashMap for better performance
    values: FxHashMap<String, Value>,
    // Avoid Box for small environments
    pub enclosing: Option<Arc<Mutex<Environment>>>,
    // Separate native functions to global environment only
    natives: FxHashMap<String, NativeFunction>,
    // Consider using string interning for module names
    modules: FxHashMap<String, Module>,
    pub depth: usize,
    // Cache frequently accessed values
    pub base_path: PathBuf,
}

impl Environment {
    pub fn new(base_path: PathBuf) -> Self {
        let mut env = Environment {
            values: FxHashMap::default(),
            natives: FxHashMap::default(),
            modules: FxHashMap::default(),
            enclosing: None,
            depth: 0,
            base_path
        };

        // Define native functions
        //System

        env.define_native("exit", 1, |args| {
            if let Value::Number(code) = args[0] {
                std::process::exit(code as i32);
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                ))
            }
        });
        env.define_native("random", 0, |_args| {
            Ok(Value::Number(rand::random::<f64>()))
        });
        env.define_native("clock", 0, |_args| {
            Ok(Value::Number(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64(),
            ))
        });
        env.define_native("print", 1, |args| {
            println!("{}", args[0]);
            Ok(Value::Nil)
        });
        env.define_native("input", 0, |_args| {
            let mut input = String::new();
            match std::io::stdin().read_line(&mut input) {
                Ok(_) => {
                    // Trim the trailing newline
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
        });
        env.define_native("einput", 1, |args| {
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
        //File
        env.define_native("readFile", 1, |args| {
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
        env.define_native("writeFile", 2, |args| {
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
        env.define_native("appendFile", 2, |args| {
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
                        crate::error::RuntimeErrorKind::IoError(e.to_string())
                    ))
                }
            } else {
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0)
                ))
            }
        });
        // toString function - converts any value to a string
        env.define_native("toString", 1, |args| {
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
                // Add other value types as needed
            };
            Ok(Value::String(string_value))
        });
        // toNumber function - attempts to convert a value to a number
        env.define_native("toNumber", 1, |args| {
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
        env.define_native("toBool", 1, |args| {
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
        //Math
        env.define_native("sqrt", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(n.sqrt())),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("pow", 2, |args| {
            let value = &args[0];
            let power = &args[1];
            match (value, power) {
                (Value::Number(n), Value::Number(p)) => Ok(Value::Number(n.powf(*p))),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("abs", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(n.abs())),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("round", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(n.round())),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("floor", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(n.floor())),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("ceil", 1, |args| {
            let value = &args[0];
            match value {
                Value::Number(n) => Ok(Value::Number(n.ceil())),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        //String
        env.define_native("len", 1, |args| match &args[0] {
            Value::String(s) => Ok(Value::Number(s.len() as f64)),
            Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::InvalidArgumentType(0),
            )),
        });
        env.define_native("concat", 2, |args| {
            let value = &args[0];
            let value2 = &args[1];
            match (value, value2) {
                (Value::String(s), Value::String(s2)) => Ok(Value::String(s.clone() + s2)),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        env.define_native("substring", 3, |args| {
            let value = &args[0];
            let start = &args[1];
            let end = &args[2];
            match (value, start, end) {
                (Value::String(s), Value::Number(n), Value::Number(n2)) => {
                    let start = *n as usize;
                    let end = *n2 as usize;
                    if start > s.len() || end > s.len() || start > end {
                        return Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                        ));
                    }
                    Ok(Value::String(s[start..end].to_string()))
                }
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidArgumentType(0),
                )),
            }
        });
        // Add more native functions here
        env
    }

    pub fn new_with_enclosing(enclosing: Option<Arc<Mutex<Environment>>>) -> Arc<Mutex<Self>> {
        let depth = enclosing.as_ref().map_or(0, |e| e.lock().unwrap().depth + 1);
        Arc::new(Mutex::new(Self {
            natives: FxHashMap::default(),
            modules: FxHashMap::default(),
            values: FxHashMap::default(),
            enclosing,
            depth,
            base_path: PathBuf::from(".".to_string())
        }))
    }

    pub fn new_empty() -> Self{
        let env = Environment {
            values: FxHashMap::default(),
            natives: FxHashMap::default(),
            modules: FxHashMap::default(),
            enclosing: None,
            depth: 0,
            base_path: PathBuf::from(".".to_string())
        };
        env
    }

    pub fn define_native(
        &mut self,
        name: &str,
        arity: usize,
        func: fn(&Vec<Value>) -> InterpreterResult<Value>,
    ) {
        self.natives
            .insert(name.to_string(), NativeFunction::new(name, arity, func));
    }

    pub fn define_class(&mut self, name: String, methods: HashMap<String, Value>) {
        self.values.insert(name.clone(), Value::Class(name, methods));
    }

    pub fn get_values(&self) -> FxHashMap<String, Value> {
        self.values.clone()
    }
    pub fn get_enclosing(&self) -> Option<Arc<Mutex<Environment>>> {
        self.enclosing.clone()
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(value) = self.natives.get(name) {
            Some(Value::NativeFunction(value.clone()))
        }else if let Some(enclosing) = &self.enclosing {
            let enclosing_lock = enclosing.lock().unwrap();
            enclosing_lock.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> InterpreterResult<Value> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value.clone());
            Ok(value)
        } 
        else if let Some(enclosing) = &self.enclosing {
            enclosing.lock().unwrap().assign(name, value)
        } 
        else {
            Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::UndefinedVariable(0, name.to_string())
            ))
        }
    }
    
    pub fn resolve_module_path(&self, import_path: &str) -> InterpreterResult<PathBuf> {
        let path = Path::new(import_path);
        
        // If the path is absolute, use it directly
        if path.is_absolute() {
            return Ok(path.to_path_buf());
        }

        // Try to resolve relative to the current module's base path
        let resolved_path = self.base_path.join(path);
        
        // Check if the file exists
        if resolved_path.exists() {
            Ok(resolved_path)
        } else {
            // You might want to add additional search paths here
            Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::RuntimeError(0,
                    format!("Could not find module: {}", import_path)
                )
            ))
        }
    }

    pub fn import_module(&mut self, path: &str) -> InterpreterResult<()> {
        let full_path = self.resolve_module_path(path)?;
        // Normalize path and get module name
        let path = std::path::Path::new(path);
        let module_name = full_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::RuntimeError(0,"Invalid module path".to_string())
            ))?;

        // Check if module is already imported
        if self.modules.contains_key(module_name) {
            return Ok(());
        }

        // Read file content
        let content = std::fs::read_to_string(&full_path).map_err(|_| {
            InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::RuntimeError(0,
                    format!("Could not read module file: {}", full_path.display())
                )
            )
        })?;
        // Parse and execute module code
        let mut tokenizer = Tokenizer::new();
        tokenizer.tokenize(&content)?;
        let tokens: Vec<crate::tokenizer::Token> = tokenizer.get_tokens();
        let expresions = Parser::new(tokens).parse()?;
        // Create interpreter for module
        let mut interpreter = Interpreter::new();
        interpreter.interpret(expresions)?;
        let module_env = interpreter.environment;
        // Store module
        let module = Module {
            name: module_name.to_string(),
            environment: module_env,
            path: path.to_str().unwrap().to_string(),
        };

        self.modules.insert(module_name.to_string(), module);
        Ok(())
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn get_from_module(&self, var_name: &str) -> Option<Value> {
        for module in self.modules.values() {
            if let Some(value) = module.environment.lock().unwrap().get(var_name) {
                return Some(value.clone());
            }
        }
        None
    }
}
