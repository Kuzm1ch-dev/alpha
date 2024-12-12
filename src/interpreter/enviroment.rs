use std::{collections::HashMap, future::Future, path::{Path, PathBuf}, pin::Pin, sync::{Arc, Mutex}};

use rustc_hash::FxHashMap;

use crate::{
    error::{InterpreterError, InterpreterResult,RuntimeErrorKind},
    parser::Parser, tokenizer::Tokenizer,
};

use super::{native::NativeFunction, value::{self, Value}, Interpreter};

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub environment: Arc<Mutex<Environment>>,
    pub path: String,
}

#[derive(Clone, Debug)]
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
        Environment {
            values: FxHashMap::default(),
            natives: FxHashMap::default(),
            modules: FxHashMap::default(),
            enclosing: None,
            depth: 0,
            base_path
        }
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
        }else if let Some(value) = &self.get_from_module(name) {
            Some(value.clone())
        }  else {
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
        let module_env = interpreter.environment.clone();
        // Store module
        let module = Module {
            name: module_name.to_string(),
            environment: module_env,
            path: path.to_str().unwrap().to_string(),
        };
        interpreter.runtime.shutdown_background();
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
