use std::fmt::Binary;

use crate::{
    error::{InterpreterError, InterpreterKind, InterpreterResult},
    tokenizer::{Token, TokenType},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Logical(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Token, String),
    Array(Vec<Expr>),
    Unary(Token, Box<Expr>),
    Nil,
    Variable(Token),                        // For variable references
    Assign(Token, Box<Expr>),               // For variable assignment 
    Let(Token, Box<Expr>),                  // For variable declaration
    Block(Vec<Expr>),                       // For block of expressions
    Function(Token, Vec<Token>, Box<Expr>), // Function declaration
    Class(Token, Vec<Expr>),                // Class declaration
    Call(Option<Box<Expr>>, Box<Expr>, Vec<Expr>),      // Function call (owner, func, args)
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    While(Box<Expr>, Box<Expr>),
    For(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
    Import(Box<Expr>),
    Return(Token, Box<Expr>),
    // Break(Token),
    Get(Box<Expr>, Box<Expr>),
    Set(Box<Expr>, Box<Expr>, Box<Expr>),
    // This(Token),
    // Super(Token, Token),
}

impl Expr {
    pub fn to_rpn(&self) -> String {
        match self {
            Expr::Binary(left, operator, right) => {
                format!("({} {} {})", operator.lexeme, left.to_rpn(), right.to_rpn())
            }
            Expr::Grouping(expr) => {
                format!("(group {})", expr.to_rpn())
            }
            Expr::Literal(token_type, literal) => {
                format!("{}", literal)
            }
            Expr::Unary(operator, expr) => {
                format!("({} {})", operator.lexeme, expr.to_rpn())
            }
            Expr::Nil => {
                format!("nil nil")
            }
            Expr::Variable(token) => {
                format!("var {}", token.lexeme)
            }
            Expr::Assign(token, expr) => {
                format!("assign {} {}", token.lexeme, expr.to_rpn())
            }
            Expr::Let(token, expr) => {
                format!("let {} {}", token.lexeme, expr.to_rpn())
            }
            Expr::Block(exprs) => {
                let mut rpn = String::new();
                for expr in exprs {
                    rpn.push_str(&expr.to_rpn());
                    rpn.push(' ');
                }
                format!("block {}", rpn)
            }
            Expr::Function(token, params, body) => {
                let mut rpn = String::new();
                for param in params {
                    rpn.push_str(&param.lexeme);
                    rpn.push(' ');
                }
                rpn.push_str(&body.to_rpn());
                format!("func {} {}", token.lexeme, rpn)
            }
            Expr::Call(owner, callee, arguments) => {
                let mut rpn = String::new();
                for argument in arguments {
                    rpn.push_str(&argument.to_rpn());
                    rpn.push(' ');
                }
                format!("call {}", rpn)
            }
            Expr::If(condition, then_branch, else_branch) => {
                format!("if {} {} {}", condition.to_rpn(), then_branch.to_rpn(), else_branch.to_rpn())
            }
            Expr::Logical(left, operator, right) => {
                format!("({} {} {})", operator.lexeme, left.to_rpn(), right.to_rpn())
            }
            Expr::While(condition, body) => {
                format!("while {} {}", condition.to_rpn(), body.to_rpn())
            }
            Expr::For(initializer, condition, increment, body) => {
                format!("for {} {} {} {}", initializer.to_rpn(), condition.to_rpn(), increment.to_rpn(), body.to_rpn())
            }
            Expr::Return(token, expr) => {
                format!("return {} {}", token.lexeme, expr.to_rpn())
            }
            Expr::Import(module) => {
                format!("import {}", module.to_rpn())
            }
            Expr::Class(token, methods) => {
                let mut rpn = String::new();
                for method in methods {
                    rpn.push_str(&method.to_rpn());
                    rpn.push(' ');
                }
                format!("class {} {}", token.lexeme, rpn)
            }
            // Expr::Break(token) => {
            //     format!("break {}", token.lexeme)
            // }            
            Expr::Get(object, name) => {
                format!("get {} {}", object.to_rpn(), name.to_rpn())
            }
            Expr::Set(object, name, value) => {
                format!("set {} {} {}", object.to_rpn(), name.to_rpn(), value.to_rpn())
            }
            Expr::Array(elements) => {
                let mut rpn = String::new();
                for element in elements {
                    rpn.push_str(&element.to_rpn());
                    rpn.push(' ');
                }
                format!("array {}", rpn)
            }
            _ => {
                format!("unknown")
            }
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> InterpreterResult<Vec<(Expr, usize)>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            let stmt = self.expression()?;
            statements.push((stmt, self.peek().line));
        }

        Ok(statements)
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expression(&mut self) -> InterpreterResult<Expr> {
        self.comparison()
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn comparison(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.logical()?;
        while self.match_tokens(vec![
            TokenType::GREATER,
            TokenType::GREATER_EQUAL,
            TokenType::LESS,
            TokenType::LESS_EQUAL,
            TokenType::BANG_EQUAL,
            TokenType::EQUAL_EQUAL,
        ]) {
            let operator = self.previous();
            let right = self.logical()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
    
        Ok(expr)
    }

    fn logical(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.term()?;
        while self.match_tokens(vec![
            TokenType::OR,
            TokenType::AND,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Logical(Box::new(expr), operator, Box::new(right));
        }
    
        Ok(expr)
    }

    fn term(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.factor()?;
        while self.match_tokens(vec![TokenType::MINUS, TokenType::PLUS]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn factor(&mut self) -> InterpreterResult<Expr> {
        let mut expr = self.unary()?;
        while self.match_tokens(vec![TokenType::SLASH, TokenType::STAR]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> InterpreterResult<Expr> {
        if self.match_tokens(vec![TokenType::BANG, TokenType::MINUS]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }
        self.primary()
    }

    fn primary(&mut self) -> InterpreterResult<Expr> {
        if self.match_tokens(vec![TokenType::LEFT_BRACE]) {
            match self.block() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::CLASS]) {
            match self.class_declaration() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::FUN]) {
            match self.function_declaration() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::RETURN]) {
            match self.return_statement() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::VAR]) {
            match self.var_declaration() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::IMPORT]) {
            match self.import_statement() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::IF]) {
            match self.if_statement() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::WHILE]) {
            match self.while_statement() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::FOR]) {

            match self.for_statement() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        // Придумать как отделить вызов функций класса, геттеры и сеттеры
        if self.match_tokens(vec![TokenType::NEW]) {
            match self.class_instantiation() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::IDENTIFIER]) {
            if (self.check(TokenType::LEFT_BRACKET)){
                match self.array_access() {
                    Ok(expr) => return Ok(expr),
                    Err(e) => return Err(e),  // If it looks like a call but isn't valid, return error
                }
            }
            if (self.check(TokenType::LEFT_PAREN)){
                match self.call() {
                    Ok(expr) => return Ok(expr),
                    Err(e) => return Err(e),  // If it looks like a call but isn't valid, return error
                }
            }
            if (self.check(TokenType::DOT)){
                match self.instance_or_get_or_set() {
                    Ok(expr) => return Ok(expr),
                    Err(e) => return Err(e),  // If it looks like a call but isn't valid, return error
                } 
            }
            if let Ok(expr) = self.assignment() {
                return Ok(expr);
            }
            return self.variable();
        }
        if self.match_tokens(vec![TokenType::LEFT_PAREN]) {
            match self.expression() {
                Ok(expr) => {
                    self.consume(TokenType::RIGHT_PAREN)?;
                    return Ok(Expr::Grouping(Box::new(expr)));
                }
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::FALSE]) {
            return Ok(Expr::Literal(self.previous(), "false".to_string()));
        }
        if self.match_tokens(vec![TokenType::TRUE]) {
            return Ok(Expr::Literal(self.previous(), "true".to_string()));
        }
        if self.match_tokens(vec![TokenType::NIL]) {
            return Ok(Expr::Literal(self.previous(), "nil".to_string()));
        }
        if self.match_tokens(vec![TokenType::NUMBER, TokenType::STRING]) {
            match self.previous().literal {
                Some(literal) => return Ok(Expr::Literal(self.previous(), literal)),
                None => return Ok(Expr::Literal(self.previous(), "null".to_string())),
            }
        }
        if self.match_tokens(vec![TokenType::LEFT_BRACKET]) {
            match self.array() {
                Ok(expr) => return Ok(expr),
                Err(e) => return Err(e),
            }
        }
        if self.match_tokens(vec![TokenType::SEMICOLON]) {
            return Ok(Expr::Nil);
        }
        Err(InterpreterError::parser_error(
            crate::error::ParserErrorKind::ExpectExpression(self.peek().lexeme,self.peek().line),
        ))
    }

    fn consume(&mut self, token_type: TokenType) -> InterpreterResult<Token> {
        if self.check(token_type.clone()) {
            return Ok(self.advance());
        }
        Err(InterpreterError::parser_error(
            crate::error::ParserErrorKind::ExpectExpression(self.previous().lexeme, self.peek().line),
        ))
    }

    fn match_tokens(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().token_type == token_type
    }

    fn variable(&mut self) -> InterpreterResult<Expr> {
        let name = self.previous();
        Ok(Expr::Variable(name))
    }
    fn array(&mut self) -> InterpreterResult<Expr>{
        let mut elements = Vec::new();
        if !self.check(TokenType::RIGHT_BRACKET) {
            loop {
                elements.push(self.expression()?);
                if !self.match_tokens(vec![TokenType::COMMA]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_BRACKET)?;
        Ok(Expr::Array(elements))
    }
    fn array_access(&mut self) -> InterpreterResult<Expr>{
        let name = self.previous();
        self.consume(TokenType::LEFT_BRACKET)?;
        let index = self.expression()?;
        self.consume(TokenType::RIGHT_BRACKET)?;
        Ok(Expr::Get(Box::new(Expr::Variable(name)), Box::new(index)))
    }
    fn assignment(&mut self) -> InterpreterResult<Expr> {;
        let name = self.previous();
        if self.match_tokens(vec![TokenType::EQUAL]) {
            let value = self.expression()?;
            return Ok(Expr::Assign(name, Box::new(value)));
        }
        Err(InterpreterError::parser_error(
            crate::error::ParserErrorKind::InvalidAssignmentTarget(self.peek().line),
        ))
    }

    fn instance_or_get_or_set(&mut self) -> InterpreterResult<Expr>{
        let name = self.previous();
        if self.match_tokens(vec![TokenType::DOT]) {
            let var = self.expression()?;
            if self.match_tokens(vec![TokenType::EQUAL]){
                let new_value = self.expression()?;
                return Ok(Expr::Set(Box::new(Expr::Variable(name)), Box::new(var), Box::new(new_value)));
            }else if (self.match_tokens(vec![TokenType::LEFT_PAREN])){
                let fun_name = var.clone();
                let arguments = self.arguments()?;
                self.consume(TokenType::RIGHT_PAREN)?;
                let call = Expr::Call(Some(Box::new(Expr::Variable(name))),Box::new(fun_name), arguments);
                return Ok(call);
            }
            return Ok(Expr::Get(Box::new(Expr::Variable(name)),Box::new(var)));
        }
        Err(InterpreterError::parser_error(
            crate::error::ParserErrorKind::InvalidAssignmentTarget(self.peek().line),
        ))
    }
    

    fn var_declaration(&mut self) -> InterpreterResult<Expr> {
        let name = self.consume(TokenType::IDENTIFIER)?;

        let initializer = if self.match_token(TokenType::EQUAL) {
            self.expression()?
        } else {
            Expr::Nil
        };

        Ok(Expr::Let(name, Box::new(initializer)))
    }

    fn call(&mut self) -> InterpreterResult<Expr> {
        let mut expr: Expr = Expr::Variable(self.previous());
        // Now handle the arguments if there are parentheses
        if self.match_tokens(vec![TokenType::DOT]){
            let fun_name = self.consume(TokenType::IDENTIFIER)?;
            let fun = Expr::Variable(fun_name);
            while self.match_tokens(vec![TokenType::LEFT_PAREN]) {
                let arguments = self.arguments()?;
                self.consume(TokenType::RIGHT_PAREN)?;
                expr = Expr::Call(Some(Box::new(expr)),Box::new(fun), arguments);
                println!("class call: {:?}", expr);
                return Ok(expr);
            }
        }
        while self.match_tokens(vec![TokenType::LEFT_PAREN]) {
            let arguments = self.arguments()?;
            self.consume(TokenType::RIGHT_PAREN)?;
            expr = Expr::Call(None,Box::new(expr), arguments);
        }
        if matches!(expr, Expr::Call(..)) {
            Ok(expr)
        } else {
            Err(InterpreterError::parser_error(
                crate::error::ParserErrorKind::ExpectExpression(self.previous().lexeme, self.peek().line),
            ))
        }
    }
    fn function_declaration(&mut self) -> InterpreterResult<Expr> {
        let name: Token = self.consume(TokenType::IDENTIFIER)?;
        self.consume(TokenType::LEFT_PAREN)?;
        let mut parameters = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                if parameters.len() >= 255 {
                    return Err(InterpreterError::parser_error(
                        crate::error::ParserErrorKind::InvalidParametsCount(self.previous().line),
                    ));
                }
                parameters.push(self.consume(TokenType::IDENTIFIER)?);;
                if !self.match_token(TokenType::COMMA) {
                    break;
                }
            }
        }
        self.consume(TokenType::RIGHT_PAREN)?;

        self.consume(TokenType::LEFT_BRACE)?;
        let body = self.block()?;

        Ok(Expr::Function(name, parameters, Box::new(body)))
    }
    fn block(&mut self) -> InterpreterResult<Expr> {
        let mut statements = Vec::new();

        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            statements.push(self.expression()?);
        }
        self.consume(TokenType::RIGHT_BRACE)?;
        Ok(Expr::Block(statements))
    }
    fn if_statement(&mut self) -> InterpreterResult<Expr> {
        self.consume(TokenType::LEFT_PAREN)?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN)?;
        let then_branch = self.expression()?;
        let else_branch = if (self.match_token(TokenType::SEMICOLON) && self.match_token(TokenType::ELSE)) 
        || self.match_token(TokenType::ELSE) {
            if self.match_token(TokenType::IF) {
                // This is an else-if
                self.if_statement()?
            } else {
                // This is a regular else
                self.expression()?
            }
        } else {
            Expr::Nil
        };
        Ok(Expr::If(Box::new(condition), Box::new(then_branch), Box::new(else_branch)))
    }
    fn while_statement(&mut self) -> InterpreterResult<Expr> {
        self.consume(TokenType::LEFT_PAREN)?;
        let condition = self.expression()?;
        self.consume(TokenType::RIGHT_PAREN)?;
        let body = self.expression()?;
        Ok(Expr::While(Box::new(condition), Box::new(body)))
    }
    fn for_statement(&mut self) -> InterpreterResult<Expr> {
        self.consume(TokenType::LEFT_PAREN)?;
        let initializer = if self.match_token(TokenType::SEMICOLON) {
            Expr::Nil
        } else if self.match_token(TokenType::VAR) {
            self.var_declaration()?
        } else {
            self.expression()?
        };
        if initializer != Expr::Nil {
            self.consume(TokenType::SEMICOLON)?;
        }
        let condition = if self.check(TokenType::SEMICOLON) {
            Expr::Literal(Token{
                token_type: TokenType::TRUE, 
                lexeme: "true".to_string(), 
                literal: None, 
                line: self.peek().line}, "true".to_string())
        } else {
            self.expression()?
        };
        self.consume(TokenType::SEMICOLON)?;
        let increment = if self.check(TokenType::RIGHT_PAREN) {
            Expr::Nil
        } else {
            self.expression()?
        };
        self.consume(TokenType::RIGHT_PAREN)?;
        let body = self.expression()?;
        Ok(Expr::For(Box::new(initializer),Box::new(condition),Box::new(increment), Box::new(body)))
    }
    fn import_statement(&mut self) -> InterpreterResult<Expr> {
        self.consume(TokenType::STRING)?;
        match self.previous().literal {
            Some(literal) => Ok(Expr::Import(Box::new(Expr::Literal(self.previous(), literal)))),
            None => Err(InterpreterError::parser_error(
                crate::error::ParserErrorKind::InvalidImport(self.peek().line),
            ))
        }
    }
    fn class_declaration(&mut self) -> InterpreterResult<Expr> {
        let name = self.consume(TokenType::IDENTIFIER)?;
        self.consume(TokenType::LEFT_BRACE)?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RIGHT_BRACE) && !self.is_at_end() {
            methods.push(self.expression()?);
        }
        self.consume(TokenType::RIGHT_BRACE)?;
        Ok(Expr::Class(name, methods))
    }
    fn class_instantiation(&mut self) -> InterpreterResult<Expr> {
        let class_name = self.consume(TokenType::IDENTIFIER)?;
        let class = Expr::Variable(class_name.clone());
        self.consume(TokenType::LEFT_PAREN)?;
        let arguments = self.arguments()?;
        self.consume(TokenType::RIGHT_PAREN)?;
        Ok(Expr::Call(None, Box::new(class), arguments))
    }

    fn arguments(&mut self) -> InterpreterResult<Vec<Expr>> {
        let mut args = Vec::new();
        if !self.check(TokenType::RIGHT_PAREN) {
            loop {
                args.push(self.expression()?);
                if !self.match_tokens(vec![TokenType::COMMA]) {
                    break;
                }
            }
        }
        Ok(args)
    }
    fn return_statement(&mut self) -> InterpreterResult<Expr> {
        let keyword = self.previous();
        let value = if !self.check(TokenType::SEMICOLON) {
            self.expression()?
        } else {
            Expr::Nil
        };
        Ok(Expr::Return(keyword, Box::new(value)))
    }
}
