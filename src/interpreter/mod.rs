use std::collections::HashMap;
use std::path::PathBuf;
use enviroment::{Environment, Value};

use crate::error::{InterpreterError, InterpreterResult};
use crate::parser::Expr;
use crate::tokenizer::TokenType;
pub mod enviroment;
pub struct Interpreter {
    environment: Box<Environment>,
    line: usize
}

impl Interpreter {
    pub fn new() -> Self {
        let path = PathBuf::new();
        Interpreter {
            environment: Box::new(Environment::new(path)),
            line: 0
        }
    }

    pub fn new_with_base_path(base_path: PathBuf) -> Self {
        Interpreter {
            environment: Box::new(Environment::new(base_path)),
            line: 0
        }
    }

    pub fn interpret(&mut self, expressions: Vec<(Expr, usize)>) -> InterpreterResult<Value> {
        let mut last_value = Value::Nil;
        //println!("expressions: {:#?}", expressions);
        for (expr, line) in expressions {
            self.line = line;
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
            Expr::Literal(token, value) => {
                match token.token_type {
                    TokenType::NUMBER => {
                        Ok(Value::Number(value.parse().unwrap()))
                    }
                    TokenType::STRING => {
                        Ok(Value::String(value.clone()))
                    }
                    TokenType::TRUE => Ok(Value::Boolean(true)),
                    TokenType::FALSE => Ok(Value::Boolean(false)),
                    TokenType::NIL => Ok(Value::Nil),
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidLiteral(token.line)))
                }
            }
            Expr::Binary(left, operator, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                match operator.token_type {
                    TokenType::PLUS => self.add(left, right),
                    TokenType::MINUS => self.subtract(left, right),
                    TokenType::STAR => self.multiply(left, right),
                    TokenType::SLASH => self.divide(left, right),
                    TokenType::GREATER => self.greater(left, right),
                    TokenType::GREATER_EQUAL => self.greater_equal(left, right),
                    TokenType::LESS => self.less(left, right),
                    TokenType::LESS_EQUAL => self.less_equal(left, right),
                    TokenType::EQUAL_EQUAL => self.equal(left, right),
                    TokenType::BANG_EQUAL => self.not_equal(left, right),
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidBinaryOperator(operator.line)))
                }
            }
            Expr::Unary(operator, expr) => {
                let right = self.evaluate(expr)?;
                
                match operator.token_type {
                    TokenType::MINUS => self.negate(right),
                    TokenType::BANG => self.not(right),
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidUnaryOperator(operator.line)))
                }
            }
            Expr::Variable(name) => {
                self.environment.get(&name.lexeme)
            }
            Expr::Assign(name, value) => {
                let value = self.evaluate(value)?;
                self.environment.assign(&name.lexeme, value.clone())?;
                Ok(value)
            }
            Expr::Let(name, initializer) => {
                let value = self.evaluate(initializer)?;
                self.environment.define(&name.lexeme, value.clone());
                Ok(value)
            }
            Expr::Block(statements) => {
                let environment = Environment::new_with_enclosing(self.environment.clone());
                self.execute_block(statements, environment)
            }
            Expr::Function(name, params, body) => {
                let function = Value::Function(
                    name.lexeme.clone(),
                    params.iter().map(|p| p.lexeme.clone()).collect(),
                    body.clone(),
                    self.environment.get_enclosing().clone()
                );
                self.environment.define(&name.lexeme, function.clone());
                Ok(function)
            }
            Expr::Call(callee, arguments) => {
                let callee = self.evaluate(callee)?;
                let mut evaluated_args = Vec::new();
                for arg in arguments {
                    evaluated_args.push(self.evaluate(arg)?);
                }
                self.execute_call(callee, evaluated_args)
            }
            Expr::Grouping(expr) => self.evaluate(expr),
            Expr::Nil => Ok(Value::Nil),
            Expr::If(condition, then_branch, else_branch) => {
                let condition = self.evaluate(condition)?;
                match self.is_truthy(&condition) {
                    true => self.evaluate(then_branch),
                    false => self.evaluate(else_branch),
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidCondition(self.line)))
                }
            },
            Expr::Logical(left, operator, right) => {
                let left_val = self.evaluate(left)?;
                match operator.token_type {
                    TokenType::OR => {
                        // If left is truthy, return immediately without evaluating right
                        if self.is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                        // Only evaluate right if left is falsy
                        self.evaluate(right)
                    },
                    TokenType::AND => {
                        // If left is falsy, return immediately without evaluating right
                        if !self.is_truthy(&left_val) {
                            return Ok(left_val);
                        }
                        // Only evaluate right if left is truthy
                        self.evaluate(right)
                    }
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidLogicalOperator(operator.line)))
                }
            },
            Expr::While(condition, body) => {
                let mut result = Value::Nil;
                let mut _condition = self.evaluate(condition)?;
                while self.is_truthy(&_condition) {
                    result = self.evaluate(body)?;
                    _condition = self.evaluate(condition)?;
                }
                Ok(result)
            },
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
            },
            Expr::Return(_, value) => {
                let value = self.evaluate(value)?;
                Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::Return(value)))
            }, 
            Expr::Import(path) => {
                let path = self.evaluate(path)?;
                match path {
                    Value::String(path) => {
                        self.environment.import_module(&path)?;
                        Ok(Value::String(path))
                    }
                    _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidImport(self.line, path.to_string())))
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
                                self.environment.get_enclosing().clone()
                            );
                            class_methods.insert(name.lexeme.clone(), function);
                        }
                        _ => return Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidClassMethod(self.line)))
                    }
                }
                let class = Value::Class(name.lexeme.clone(), class_methods);
                self.environment.define(&name.lexeme, class.clone());
                Ok(class)
            }, 
        }
    }

    fn execute_block(&mut self, statements: &[Expr], environment: Environment) -> InterpreterResult<Value> {
        let previous = self.environment.clone();
        self.environment = Box::new(environment);

        let mut result = Value::Nil;
        for statement in statements {
            match self.evaluate(statement) {
                Err(InterpreterError::RuntimeError(crate::error::RuntimeErrorKind::Return(value))) => {
                    result = value.clone();
                    break;
                }
                Err(e) => return Err(e),
                Ok(_) => continue,
            }
        }
        // self.environment = previous;
        let enclosing = self.environment.get_enclosing();
        match enclosing {
            Some(enclosing) => {
                self.environment = enclosing;
            }
            None => {
                self.environment = previous;
            }
        }
        match result {
            Value::Nil => Ok(Value::Nil),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::Return(result)))
        }
    }

    fn execute_call(&mut self, callee: Value, arguments: Vec<Value>) -> InterpreterResult<Value> {
        println!("callee {:?}", callee);
        match callee {
            Value::Function(name, params, body, closure_env) => {
                if arguments.len() != params.len() {
                    return Err(InterpreterError::runtime_error(
                        crate::error::RuntimeErrorKind::ExpextedArgument(self.line,arguments.len(),params.len())
                    ));
                }
                let mut environment = Environment::new_with_enclosing(self.environment.clone());
                for (param, arg) in params.iter().zip(arguments) {
                    environment.define(param, arg);
                }
                if let Some(closure_env) = closure_env {
                    for value in closure_env.get_values(){
                        environment.define(&value.0, value.1.clone());
                    }
                }
                match self.execute_block(&[*body], environment) {
                    Err(InterpreterError::RuntimeError(crate::error::RuntimeErrorKind::Return(value))) => {
                        Ok(value)
                    }
                    Err(e) => Err(e),
                    Ok(_) => Ok(Value::Nil)
                    // ... other cases ...
                }
                // self.execute_block(&[*body], environment);
            }
            Value::NativeFunction(function) => function.call(&arguments),
            Value::Class(name, methods) => {
                let instance = Value::Instance(name, HashMap::new());
                if let Some(method) = methods.get("_construct") {
                    match method {
                        Value::Function(_, params, body, _) => {
                            let mut environment = Environment::new_with_enclosing(self.environment.clone());
                            environment.define("this", instance.clone());
                            for (param, arg) in params.iter().zip(arguments) {
                                environment.define(param, arg);
                            }
                            self.execute_block(&[*body.clone()], environment)?;
                        }
                        _ => return Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::InvalidClassMethod(self.line)))
                    }
                }
                Ok(instance)
            }, 
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::UndefinedFunction(self.line)))
        }
    }
    fn add(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumbersOrStrings(self.line)))
        }
    }

    fn subtract(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn multiply(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumbersOrStrings(self.line)))
        }
    }

    fn divide(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0.0 {
                    Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::DivisionByZero(self.line)))
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn greater(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn greater_equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn less(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn less_equal(&self, left: Value, right: Value) -> InterpreterResult<Value> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
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
            _ => Err(InterpreterError::runtime_error(crate::error::RuntimeErrorKind::OperandsMustBeNumber(self.line)))
        }
    }

    fn not(&self, value: Value) -> InterpreterResult<Value> {
        Ok(Value::Boolean(!self.is_truthy(&value)))
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            Value::String(s) => true,
            Value::Number(n) => *n != 0.0,
            _ => true,
        }
    }
}