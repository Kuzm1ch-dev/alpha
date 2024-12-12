use enviroment::Environment;
use std::collections::HashMap;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use value::{PromiseState, Value};

use crate::error::{InterpreterError, InterpreterResult};
use crate::parser::{Expr, TryCatch};
use crate::tokenizer::TokenType;
pub mod enviroment;
pub mod native;
pub mod native_functions;
pub mod value;

pub struct Interpreter {
    environment: Arc<Mutex<Environment>>,
    line: usize,
    pub runtime: tokio::runtime::Runtime
}

impl Interpreter {
    pub fn new() -> Self {
        let path = PathBuf::new();
        let env = Arc::new(Mutex::new(Environment::new(path)));
        env.lock().unwrap().register_native_functions();
        let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
        Interpreter {
            environment: env,
            line: 0,
            runtime
        }
    }

    pub fn new_with_environment(env: Arc<Mutex<Environment>>) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();
        Interpreter {
            environment: env,
            line: 0,
            runtime
        }
    }

    pub fn new_with_base_path(base_path: PathBuf) -> Self {
        let env = Arc::new(Mutex::new(Environment::new(base_path)));
        env.lock().unwrap().register_native_functions();
        let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build().unwrap();
        Interpreter {
            environment: env,
            line: 0,
            runtime
        }
    }

    pub fn interpret(&mut self, expressions: Vec<(Expr, usize)>) -> InterpreterResult<Value> {
        let mut last_value = Value::Nil;
        //println!("expressions: {:#?}", expressions);
        for (expr, line) in expressions {
            self.line = line;
            //println!("{:?}", expr);
            match self.evaluate(&expr) {
                Ok(value) => {
                    last_value = value;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(last_value)
    }

    pub fn evaluate(&mut self, expr: &Expr) -> InterpreterResult<Value> {
        match expr {
            Expr::Literal(token, value) => match token.token_type {
                TokenType::Number => Ok(Value::Number(value.parse().unwrap())),
                TokenType::STRING => Ok(Value::String(value.clone())),
                TokenType::True => Ok(Value::Boolean(true)),
                TokenType::False => Ok(Value::Boolean(false)),
                TokenType::Nil => Ok(Value::Nil),
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidLiteral(token.line),
                )),
            },
            Expr::Variable(name) => {
                let value = self.environment.lock().unwrap().get(&name.lexeme);
                match value {
                    Some(value) => Ok(value.clone()),
                    None => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::UndefinedVariable(
                            self.line,
                            name.lexeme.clone(),
                        ),
                    )),
                }
            }
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate(element)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Dictionary(elements) => {
                let mut values = HashMap::new();
                for (key, value) in elements {
                    let key = self.evaluate(key)?;
                    let value = self.evaluate(value)?;
                    match key {
                        Value::String(key) => {
                            values.insert(key, value);
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidDictionaryKey(self.line),
                            ))
                        }
                    }
                }
                Ok(Value::Dictionary(values))
            }
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::Plus => self.add(left, right),
                    TokenType::Minus => self.subtract(left, right),
                    TokenType::Star => self.multiply(left, right),
                    TokenType::Modulo => self.modulo(left, right),
                    TokenType::Slash => self.divide(left, right),
                    TokenType::Greater => self.greater(left, right),
                    TokenType::GreaterEqual => self.greater_equal(left, right),
                    TokenType::Less => self.less(left, right),
                    TokenType::LessEqual => self.less_equal(left, right),
                    TokenType::EqualEqual => self.equal(left, right),
                    TokenType::BandEqual => self.not_equal(left, right),
                    _ => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidBinaryOperator(operator.line),
                    )),
                }
            }
            Expr::Unary(operator, expr) => {
                let right = self.evaluate(expr)?;

                match operator.token_type {
                    TokenType::Minus => self.negate(right),
                    TokenType::Bang => self.not(right),
                    _ => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidUnaryOperator(operator.line),
                    )),
                }
            }
            Expr::Assign(name, value) => {
                let evaluated_value = self.evaluate(value)?;
                self.environment
                    .lock()
                    .unwrap()
                    .assign(&name.lexeme, evaluated_value.clone())?;
                Ok(evaluated_value)
            }
            Expr::Set(object, name, value) => {
                let value_name = object.lexeme.clone();
                let object = self
                    .environment
                    .lock()
                    .unwrap()
                    .get(&value_name)
                    .ok_or_else(|| {
                        InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::UndefinedVariable(
                                self.line,
                                value_name.clone(),
                            ),
                        )
                    })?;
                let value = self.evaluate(value)?;
                let name = self.evaluate(name)?;
                match object {
                    Value::Instance(_, _) => match name {
                        Value::String(name) => {
                            self.environment
                                .lock()
                                .unwrap()
                                .assign(&name, value.clone())?;
                            return Ok(value);
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidSet(self.line),
                            ))
                        }
                    },
                    Value::Array(mut values) => match name {
                        Value::Number(index) => {
                            if index < values.len() as f64 {
                                let index = index as usize;
                                values[index] = value.clone();
                                self.environment
                                    .lock()
                                    .unwrap()
                                    .assign(&value_name, Value::Array(values.clone()))?;
                                return Ok(value);
                            } else {
                                return Err(InterpreterError::runtime_error(
                                    crate::error::RuntimeErrorKind::InvalidSet(self.line),
                                ));
                            }
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidSet(self.line),
                            ))
                        }
                    },
                    Value::Dictionary(mut values) => match name {
                        Value::String(key) => {
                            values.insert(key, value.clone());
                            self.environment
                                .lock()
                                .unwrap()
                                .assign(&value_name, Value::Dictionary(values.clone()))?;
                            return Ok(value);
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidSet(self.line),
                            ))
                        }
                    },
                    _ => {
                        return Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidSet(self.line),
                        ))
                    }
                }
            }
            Expr::Get(object, name) => {
                let object = self.evaluate(object)?;
                let name = self.evaluate(name)?;
                match object {
                    Value::Instance(_, _) => match name {
                        Value::String(name) => {
                            self.environment.lock().unwrap().get(&name).ok_or_else(|| {
                                InterpreterError::runtime_error(
                                    crate::error::RuntimeErrorKind::InvalidGet(self.line),
                                )
                            })
                        }
                        _ => Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidGet(self.line),
                        )),
                    },
                    Value::Array(values) => match name {
                        Value::Number(index) => {
                            if index < values.len() as f64 {
                                return Ok(values[index as usize].clone());
                            } else {
                                Err(InterpreterError::runtime_error(
                                    crate::error::RuntimeErrorKind::InvalidGet(self.line),
                                ))
                            }
                        }
                        _ => Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidGet(self.line),
                        )),
                    },
                    Value::Dictionary(values) => match name {
                        Value::String(key) => match values.get(&key) {
                            Some(value) => Ok(value.clone()),
                            None => Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidGet(self.line),
                            )),
                        },
                        _ => Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidGet(self.line),
                        )),
                    },
                    _ => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidGet(self.line),
                    )),
                }
            }
            Expr::Let(name, initializer) => {
                let value = self.evaluate(initializer)?;
                self.environment
                    .lock()
                    .unwrap()
                    .define(&name.lexeme, value.clone());
                Ok(value)
            }
            Expr::Block(statements) => {
                let environment =
                    Environment::new_with_enclosing(Some(Arc::clone(&self.environment)));
                self.execute_block(statements, environment)
            }
            Expr::Function(name, params, body) => {
                let function = Value::Function(
                    name.lexeme.clone(),
                    params.iter().map(|p| p.lexeme.clone()).collect(),
                    body.clone(),
                    // Some(environment),
                );
                self.environment
                    .lock()
                    .unwrap()
                    .define(&name.lexeme, function.clone());
                Ok(function)
            }
            Expr::AsyncFunction(name, params, body) => {
                let function = Value::AsyncFunction(
                    name.lexeme.clone(),
                    params.iter().map(|p| p.lexeme.clone()).collect(),
                    body.clone(),
                );
                self.environment
                    .lock()
                    .unwrap()
                    .define(&name.lexeme, function.clone());
                Ok(function)
            }
            Expr::Call(owner, callee, arguments) => {
                let mut evaluated_args = Vec::new();
                for arg in arguments {
                    evaluated_args.push(self.evaluate(arg)?);
                }
                if let Some(owner) = owner {
                    let owner = self.evaluate(owner)?;
                    if let Value::Instance(_, env) = owner.clone() {
                        let previous = self.environment.clone();
                        self.environment = env;
                        let callee = self.evaluate(callee)?;
                        let result = self.execute_call(Some(owner), callee, evaluated_args);
                        self.environment = previous;
                        return result;
                    }
                    Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidCall(0),
                    ))
                } else {
                    let callee = self.evaluate(callee)?;
                    match callee {
                        Value::Function(_, _, _) => {
                            let result = self.execute_call(None, callee, evaluated_args);
                            return result;
                        }
                        Value::AsyncFunction(_, _, _) => {
                            let future = self.execute_async_call(None, callee, evaluated_args);
                            return Ok(Value::create_promise(Box::pin(future)));
                        }
                        Value::NativeFunction(_) => {
                            let result = self.execute_call(None, callee, evaluated_args);
                            return result;
                        }
                        _ => Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::InvalidCall(0),
                        )),
                    }
                    // self.execute_call(None, callee, evaluated_args)
                }
            }
            Expr::Await(expr) => {
                let expr = self.evaluate(expr)?;
                if let Value::Promise(join_handle) = expr {
                    let mut promise = join_handle.lock().unwrap();
                    match &mut *promise {
                        PromiseState::Pending(join_handle) => {
                            let result = tokio::task::block_in_place(|| {
                                self.runtime.block_on(join_handle)                                                        
                            });
                            return result;
                        }
                        PromiseState::Fulfilled(value) => return Ok(value.clone()),
                        PromiseState::Rejected(_error) => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::PromiseRejected(self.line),
                            ))
                        }
                    }
                }
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::InvalidCall(0),
                ))
            }
            //     let mut evaluated_args = Vec::new();
            //     for arg in arguments {
            //         evaluated_args.push(self.evaluate(arg)?);
            //     }
            //     if let Some(owner) = owner {
            //         let owner = self.evaluate(owner)?;
            //         if let Value::Instance(_, env) = owner.clone() {
            //             let previous = self.environment.clone();
            //             //self.environment = env;
            //             let callee = self.evaluate(callee)?;
            //             let join_handle = self.execute_async_call(Some(owner), callee, evaluated_args);
            //             return Ok(Value::create_promise(join_handle));
            //         }
            //         Err(InterpreterError::runtime_error(
            //             crate::error::RuntimeErrorKind::InvalidCall(0),
            //         ))
            //     } else {
            //         let callee = self.evaluate(callee)?;
            //         let join_handle = self.execute_async_call(None, callee, evaluated_args);
            //         return Ok(Value::create_promise(join_handle));
            //     }
            // }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Nil => Ok(Value::Nil),
            Expr::If(condition, then_branch, else_branch) => {
                let condition = self.evaluate(condition)?;
                match self.is_truthy(&condition) {
                    true => self.evaluate(then_branch),
                    false => self.evaluate(else_branch),
                    // _ => Err(InterpreterError::runtime_error(
                    //     crate::error::RuntimeErrorKind::InvalidCondition(self.line),
                    // )),
                }
            }
            Expr::Logical(left, operator, right) => {
                let left_val = self.evaluate(left)?;
                match operator.token_type {
                    TokenType::Or => {
                        // If left is truthy, return immediately without evaluating right
                        if self.is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                        // Only evaluate right if left is falsy
                        self.evaluate(right)
                    }
                    TokenType::And => {
                        // If left is falsy, return immediately without evaluating right
                        if !self.is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                        // Only evaluate right if left is truthy
                        self.evaluate(right)
                    }
                    _ => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidLogicalOperator(operator.line),
                    )),
                }
            }
            Expr::While(condition, body) => {
                let mut result = Value::Nil;
                let mut _condition = self.evaluate(condition)?;
                while self.is_truthy(&_condition) {
                    result = self.evaluate(body)?;
                    _condition = self.evaluate(condition)?;
                }
                Ok(result)
            }
            Expr::For(initializer, condition, increment, body) => {
                let mut result = Value::Nil;
                self.evaluate(initializer)?;
                let mut _condition = self.evaluate(condition)?;
                while self.is_truthy(&_condition) {
                    result = self.evaluate(body)?;
                    self.evaluate(increment)?;
                    _condition = self.evaluate(condition)?;
                }
                Ok(result)
            }
            Expr::Return(_, value) => {
                let value = self.evaluate(value)?;
                Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::Return(value),
                ))
            }
            Expr::Import(path) => {
                let path = self.evaluate(path)?;
                match path {
                    Value::String(path) => {
                        self.environment.lock().unwrap().import_module(&path)?;
                        Ok(Value::String(path))
                    }
                    _ => Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::InvalidImport(self.line, path.to_string()),
                    )),
                }
            }
            Expr::Class(name, methods) => {
                let mut class_methods = HashMap::new();
                for method in methods {
                    match method {
                        Expr::Function(name, params, body) => {
                            let function = Value::Function(
                                name.lexeme.clone(),
                                params.iter().map(|p| p.lexeme.clone()).collect(),
                                body.clone(),
                                // self.environment.lock().unwrap().get_enclosing().clone(),
                            );
                            class_methods.insert(name.lexeme.clone(), function);
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidClassMethod(self.line),
                            ))
                        }
                    }
                }
                let class = Value::Class(name.lexeme.clone(), class_methods);
                self.environment
                    .lock()
                    .unwrap()
                    .define(&name.lexeme, class.clone());
                Ok(class)
            }
            Expr::TryCatch(try_catch) => self.execute_try_catch(try_catch),
        }
    }

    fn execute_block(
        &mut self,
        statements: &[Expr],
        environment: Arc<Mutex<Environment>>,
    ) -> InterpreterResult<Value> {
        let previous = self.environment.clone();
        self.environment = environment;
        let mut result = Value::Nil;
        for statement in statements {
            match self.evaluate(statement) {
                Err(InterpreterError::RuntimeError(crate::error::RuntimeErrorKind::Return(
                    value,
                ))) => {
                    return Ok(value.clone());
                }
                Err(e) => return Err(e),
                Ok(value) => result = value.clone(),
            }
        }
        self.environment = previous;
        Ok(result)
    }

    async fn execute_async_block(
        &mut self,
        statements: &[Expr],
        environment: Arc<Mutex<Environment>>,
    ) -> InterpreterResult<Value> {
        let previous = self.environment.clone();
        self.environment = environment;
        let mut result = Value::Nil;
        for statement in statements {
            match self.evaluate(statement) {
                Err(InterpreterError::RuntimeError(crate::error::RuntimeErrorKind::Return(
                    value,
                ))) => {
                    return Ok(value.clone());
                }
                Err(e) => return Err(e),
                Ok(value) => result = value.clone(),
            }
        }
        self.environment = previous;
        Ok(result)
    }

    fn execute_call(
        &mut self,
        _owner: Option<Value>,
        callee: Value,
        arguments: Vec<Value>,
    ) -> InterpreterResult<Value> {
        match callee {
            Value::Function(name, params, body) => {
                if arguments.len() != params.len() {
                    return Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::ExpextedArgument(
                            self.line,
                            arguments.len(),
                            params.len(),
                        ),
                    ));
                }
                let environment =
                    Environment::new_with_enclosing(Some(Arc::clone(&self.environment)));
                let mut env_lock = environment.lock().unwrap();
                for (param, arg) in params.iter().zip(arguments) {
                    env_lock.define(param, arg);
                }
                drop(env_lock);
                match *body {
                    Expr::Block(statements) => {
                        let result = self.execute_block(&statements, environment)?;
                        Ok(result)
                    }
                    _ => {
                        let result = self.evaluate(&body)?;
                        Ok(result)
                    }
                }
            }
            Value::AsyncFunction(name, params, body) => {
                if arguments.len() != params.len() {
                    return Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::ExpextedArgument(
                            self.line,
                            arguments.len(),
                            params.len(),
                        ),
                    ));
                }
                let environment =
                    Environment::new_with_enclosing(Some(Arc::clone(&self.environment)));
                let mut env_lock = environment.lock().unwrap();
                for (param, arg) in params.iter().zip(arguments) {
                    env_lock.define(param, arg);
                }
                drop(env_lock);
                match *body {
                    Expr::Block(statements) => {
                        let result = self.execute_block(&statements, environment)?;
                        Ok(result)
                    }
                    _ => {
                        let result = self.evaluate(&body)?;
                        Ok(result)
                    }
                }
            }
            Value::NativeFunction(function) => function.call(&arguments),
            Value::Class(name, methods) => {
                let environment =
                    Environment::new_with_enclosing(Some(Arc::clone(&self.environment)));
                if let Some(method) = methods.get("_construct") {
                    match method {
                        Value::Function(_, params, body) => {
                            // Тут переделать environment
                            for (param, arg) in params.iter().zip(arguments) {
                                environment.lock().unwrap().define(param, arg);
                            }
                            environment
                                .lock()
                                .unwrap()
                                .define("this", Value::Instance(name.clone(), environment.clone()));
                            self.execute_block(&[*body.clone()], Arc::clone(&environment))?;
                        }
                        _ => {
                            return Err(InterpreterError::runtime_error(
                                crate::error::RuntimeErrorKind::InvalidClassMethod(self.line),
                            ))
                        }
                    }
                    for (name, value) in methods {
                        environment.lock().unwrap().define(name.as_str(), value);
                    }
                }
                let instance = Value::Instance(name, environment);
                Ok(instance)
            }
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::UndefinedFunction(self.line),
            )),
        }
    }

    fn execute_async_call(
        &mut self,
        _owner: Option<Value>,
        callee: Value,
        arguments: Vec<Value>,
    ) -> impl Future<Output = Result<Value, InterpreterError>> {
        let environment = Arc::clone(&self.environment);
        let line = self.line.clone();
        async move {
            match callee {
                Value::AsyncFunction(_name, params, body) => {
                    if arguments.len() != params.len() {
                        return Err(InterpreterError::runtime_error(
                            crate::error::RuntimeErrorKind::ExpextedArgument(
                                line,
                                arguments.len(),
                                params.len(),
                            ),
                        ));
                    }
                    let mut env_lock = environment.lock().unwrap();
                    for (param, arg) in params.iter().zip(arguments) {
                        env_lock.define(param, arg);
                    }
                    drop(env_lock);
                    let mut interpreter =
                        Interpreter::new_with_environment(Arc::clone(&environment));
                    let result = match *body {
                        Expr::Block(statements) => {
                            let result = interpreter.execute_block(&statements, environment)?;
                            Ok(result)
                        }
                        _ => {
                            let result = interpreter.evaluate(&body)?;
                            Ok(result)
                        }
                    };
                    interpreter.runtime.shutdown_background();
                    return result;
                }
                _ => Err(InterpreterError::runtime_error(
                    crate::error::RuntimeErrorKind::UndefinedFunction(line),
                )),
            }
        }
    }
    fn execute_try_catch(&mut self, try_catch: &TryCatch) -> InterpreterResult<Value> {
        // Create new environment for catch block scope
        let previous_env = self.environment.clone();

        // Evaluate try block
        let result = self.evaluate(&try_catch.try_block);

        match result {
            Ok(value) => {
                // If no error occurred, return the value
                self.environment = previous_env;
                Ok(value)
            }
            Err(error) => {
                // Error occurred, execute catch block
                let catch_env = Environment::new_with_enclosing(Some(Arc::clone(&previous_env)));
                // Bind error to the catch parameter
                catch_env
                    .lock()
                    .unwrap()
                    .define(&try_catch.catch_param, Value::String(error.to_string()));
                // Set catch block environment
                self.environment = catch_env;
                // Evaluate catch block
                let catch_result = self.evaluate(&try_catch.catch_block);
                // Restore previous environment
                self.environment = previous_env;
                catch_result
            }
        }
    }

    fn add(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (a, b) => Ok(Value::String(a.to_string() + &b.to_string())),
            // _ => Err(InterpreterError::runtime_error(
            //     crate::error::RuntimeErrorKind::OperandsMustBeNumbersOrStrings(self.line),
            // )),
        }
    }

    fn subtract(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn multiply(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumbersOrStrings(self.line),
            )),
        }
    }
    fn modulo(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::DivisionByZero(self.line),
                    ))
                } else {
                    Ok(Value::Number(a % b))
                }
            }
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn divide(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::DivisionByZero(self.line),
                    ))
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn greater(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn greater_equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn less(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn less_equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        Ok(Value::Boolean(self.is_equal(&left, &right)))
    }

    fn not_equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        Ok(Value::Boolean(!self.is_equal(&left, &right)))
    }

    fn is_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            _ => false,
        }
    }

    fn negate(&self, value: Value) -> InterpreterResult<Value> {
        match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(InterpreterError::runtime_error(
                crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line),
            )),
        }
    }

    fn not(&self, value: Value) -> InterpreterResult<Value> {
        Ok(Value::Boolean(!self.is_truthy(&value)))
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            Value::String(_) => true,
            Value::Number(n) => *n != 0.0,
            _ => true,
        }
    }
}
