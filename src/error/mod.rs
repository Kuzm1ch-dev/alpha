use std::error::Error;
use std::fmt;

use crate::{interpreter::value::Value, tokenizer::{Token, TokenType}};
#[derive(Debug)]
pub struct InterpreterKind {
    message: String,
    line: usize,
}

impl InterpreterKind {
    pub fn new(message: String, line: usize) -> Self {
        InterpreterKind {
            message,
            line,
        }
    }
}
#[derive(Debug)]
pub enum TokenizerErrorKind {
    UnexpectedCharacter(char, usize),
    UnterminatedString(usize),
    InvalidNumber(usize),
    InvalidIdentifier(usize),
    InvalidToken(usize),
    UnknownError(usize),
}
impl fmt::Display for TokenizerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenizerErrorKind::UnexpectedCharacter(c, line) => {
                write!(f, "[line {}] Error: Unexpected character: {}", line, c)
            }
            TokenizerErrorKind::UnterminatedString(line) => {
                write!(f, "[line {}] Error: Unterminated string.", line)
            }
            TokenizerErrorKind::InvalidNumber(line) => {
                write!(f, "[line {}] Error: Invalid number.", line)
            }
            TokenizerErrorKind::InvalidIdentifier(line) => {
                write!(f, "[line {}] Error: Invalid identifier.", line)
            }
            TokenizerErrorKind::InvalidToken(line) => {
                write!(f, "[line {}] Error: Invalid token.", line)
            }
            TokenizerErrorKind::UnknownError(line) => {
                write!(f, "[line {}] Error: Unknown error.", line)
            }
        }
    }
}

#[derive(Debug)]
pub enum ParserErrorKind {
    ExpectedSemilicon(usize),
    UnexpectedToken(usize, TokenType),
    InvalidAssignmentTarget(usize),
    InvalidParametsCount(usize),
    InvalidExpression(usize),
    InvalidStatement(usize),
    UnknownError(usize),
    ExpectedUnary(usize),
    UnexpectedEOF(usize),
    InvalidImport(usize),
    ExpectExpression(String,usize)
}
impl fmt::Display for ParserErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserErrorKind::ExpectedSemilicon(line) => {
                write!(f, "[line {}] Error: Expect semilicon after value.", line)
            }
            ParserErrorKind::UnexpectedToken(line, token) => {
                write!(f, "[line {}] Error: Expected token {}.", line, token)
            }
            ParserErrorKind::InvalidAssignmentTarget(line) => {
                write!(f, "[line {}] Error: Invalid assignment target.", line)
            }
            ParserErrorKind::InvalidExpression(line) => {
                write!(f, "[line {}] Error: Invalid expression.", line)
            }
            ParserErrorKind::InvalidStatement(line) => {
                write!(f, "[line {}] Error: Invalid statement.", line)
            }
            ParserErrorKind::UnknownError(line) => {
                write!(f, "[line {}] Error: Unknown error.", line)
            }
            ParserErrorKind::InvalidParametsCount(line) => {
                write!(f, "[line {}] Error: Cannot have more than 255 parameters.", line)
            }
            ParserErrorKind::ExpectedUnary(line) => {
                write!(f, "[line {}] Error: Expected unary operator.", line)
            }
            ParserErrorKind::UnexpectedEOF(line) => {
                write!(f, "[line {}] Error: Unexpected end of file.", line)
            }
            ParserErrorKind::ExpectExpression(ch,line) => {
                write!(f, "[line {}] Error at '{}': Expected expression.", line, ch)
            }
            ParserErrorKind::InvalidImport(line) => {
                write!(f, "[line {}] Error: Invalid import.", line)
            }
        }
    }
}
#[derive(Debug)]
pub enum RuntimeErrorKind {
    InvalidTailCall(usize),
    InvalidNumber(usize),
    InvalidLiteral(usize),
    InvalidBinaryOperator(usize),
    InvalidUnaryOperator(usize),
    OperandsMustBeNumbersOrStrings(usize),
    OperandsMustBeNumber(usize),
    InvalidParametsCount(usize),
    UndefinedVariable(usize, String),
    UnknownBinaryOperator(usize),
    DivisionByZero(usize),
    UnknownError(usize),
    UnknownExpression(usize),
    UndefinedFunction(usize),
    ExpextedArgument(usize, usize, usize),
    InvalidCondition(usize),
    InvalidLogicalOperator(usize),
    InvalidReturnValue(usize),
    InvalidArgumentType(usize),
    RuntimeError(usize, String),
    InvalidImport(usize, String),
    InvalidClassMethod(usize),
    InvalidDictionaryKey(usize),
    InvalidSet(usize),
    InvalidGet(usize),
    IoError(String),
    InvalidCall(usize),
    Return(Value),
}
impl fmt::Display for RuntimeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeErrorKind::InvalidNumber(line) => {
                write!(f, "[line {}] Error: Invalid number.", line)
            }
            RuntimeErrorKind::InvalidLiteral(line) => {
                write!(f, "[line {}] Error: Invalid literal.", line)
            }
            RuntimeErrorKind::OperandsMustBeNumber(line) => {
                write!(f, "[line {}] Error: Operand must be a number.", line)
            }
            RuntimeErrorKind::OperandsMustBeNumbersOrStrings(line) => {
                write!(f, "[line {}] Error: Operands must be two numbers or two strings.", line)
            }
            RuntimeErrorKind::UndefinedVariable(line, name) => {
                write!(f, "[line {}] Error: Undefined variable {}.", line, name)
            }
            RuntimeErrorKind::UnknownBinaryOperator(line) => {
                write!(f, "[line {}] Error: Unknown binary operator.", line)
            }
            RuntimeErrorKind::DivisionByZero(line) => {
                write!(f, "[line {}] Error: Division by zero.", line)
            }
            RuntimeErrorKind::UnknownError(line) => {
                write!(f, "[line {}] Error: Unknown error.", line)
            }
            RuntimeErrorKind::UnknownExpression(line) => {
                write!(f, "[line {}] Error: Unknown expression.", line)
            }
            RuntimeErrorKind::InvalidBinaryOperator(line) => {
                write!(f, "[line {}] Error: Invalid binary operator.", line)
            }
            RuntimeErrorKind::InvalidUnaryOperator(line) => {
                write!(f, "[line {}] Error: Invalid unary operator.", line)
            }
            RuntimeErrorKind::ExpextedArgument(line, exptected, got) => {
                write!(f, "[line {}] Error: Expected {} arguments but got {}",line, exptected, got)
            }
            RuntimeErrorKind::UndefinedFunction(line) => {
                write!(f, "[line {}] Error: Undefined function.", line)
            }
            RuntimeErrorKind::InvalidCondition(line) => {
                write!(f, "[line {}] Error: Invalid condition.", line)
            }
            RuntimeErrorKind::InvalidLogicalOperator(line) => {
                write!(f, "[line {}] Error: Invalid logical operator.", line)
            }
            RuntimeErrorKind::InvalidReturnValue(line) => {
                write!(f, "[line {}] Error: Invalid return value.", line)
            }
            RuntimeErrorKind::InvalidParametsCount(line) => {
                write!(f, "[line {}] Error: Invalid argument count.", line)
            }
            RuntimeErrorKind::InvalidArgumentType(line) => {
                write!(f, "[line {}] Error: Invalid argument type.", line)
            }
            RuntimeErrorKind::RuntimeError(line, message) => {
                write!(f, "[line {}] Error: {}", line, message)
            }
            RuntimeErrorKind::InvalidImport(line, module) => {
                write!(f, "[line {}] Error: Invalid import module '{}'", line, module)
            }
            RuntimeErrorKind::Return(value) => {
                write!(f, "Return unwind the call stack: {}", value)
            }
            RuntimeErrorKind::IoError(message) => {
                write!(f, "IO Error: {}", message)
            }
            RuntimeErrorKind::InvalidClassMethod(line) => {
                write!(f, "[line {}] Error: Invalid class method.", line)
            }
            RuntimeErrorKind::InvalidCall(line) => {
                write!(f, "[line {}] Error: Invalid call.", line)
            }
            RuntimeErrorKind::InvalidSet(line) => {
                write!(f, "[line {}] Error: Invalid set.", line)
            }
            RuntimeErrorKind::InvalidGet(line) => {
                write!(f, "[line {}] Error: Invalid get.", line)
            }
            RuntimeErrorKind::InvalidDictionaryKey(line) => {
                write!(f, "[line {}] Error: Invalid dictionary key.", line)
            }
            RuntimeErrorKind::InvalidTailCall(line) => {
                write!(f, "[line {}] Error: Invalid Tail Call.", line)
            }
        }
    }
}


#[derive(Debug)]
pub enum UnknownErrorKind {
    UnknownError,
}


#[derive(Debug)]
pub enum InterpreterError {
    TokenizerError(TokenizerErrorKind),
    ParserError(ParserErrorKind),
    RuntimeError(RuntimeErrorKind),
    UnknownError(UnknownErrorKind),
}

impl Error for InterpreterError {}

impl fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpreterError::TokenizerError(kind) => {
                write!(f, "{}", kind)
            }
            InterpreterError::ParserError(kind) => {
                write!(f, "{}", kind)
            }
            InterpreterError::RuntimeError(kind) => {
                write!(f, "{}", kind)
            }
            InterpreterError::UnknownError(kind) => {
                write!(f, "UnknownError: {:?}", kind)
            }
        }
    }
}

impl InterpreterError {
    pub fn tokenizer_error(kind: TokenizerErrorKind) -> Self {
        InterpreterError::TokenizerError(kind)
    }
    pub fn runtime_error(kind: RuntimeErrorKind) -> Self {
        InterpreterError::RuntimeError(kind)
    }
    pub fn parser_error(kind: ParserErrorKind) -> Self {
        InterpreterError::ParserError(kind)
    }
    pub fn unknown_error(kind: UnknownErrorKind) -> Self {
        InterpreterError::UnknownError(kind)
    }
}

pub type InterpreterResult<T> = Result<T, InterpreterError>;