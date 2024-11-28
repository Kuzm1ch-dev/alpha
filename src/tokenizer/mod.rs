use std::error::Error;

use crate::error::{InterpreterError, InterpreterResult};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LEFT_PAREN,
    RIGHT_PAREN,
    LEFT_BRACE,
    RIGHT_BRACE,
    COMMA,
    DOT,
    MINUS,
    PLUS,
    SEMICOLON,
    SLASH,
    STAR,
    BANG,
    BANG_EQUAL,
    EQUAL,
    EQUAL_EQUAL,
    GREATER,
    GREATER_EQUAL,
    LESS,
    LESS_EQUAL,
    IDENTIFIER,
    STRING,
    NUMBER,
    AND,
    CLASS,
    ELSE,
    FALSE,
    FUN,
    FOR,
    IF,
    NIL,
    OR,
    RETURN,
    SUPER,
    THIS,
    TRUE,
    VAR,
    WHILE,
    EOF,
    IMPORT
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<String>,
    pub line: usize,
}

pub struct Tokenizer {
    pub current: usize,
    pub tokens: Vec<Token>,
    pub line: usize,
    pub errors: Vec<Box<dyn Error>>,
}

fn some_function() -> InterpreterResult<()> {
    Ok(())
}

impl Tokenizer {
    pub fn new() -> Self {
        Tokenizer {
            current: 0,
            tokens: Vec::new(),
            line: 1,
            errors: Vec::new(),
        }
    }
    pub fn get_tokens(&self) -> Vec<Token> {
        return self.tokens.clone();
    }
    pub fn tokenize(&mut self, input: &str) -> InterpreterResult<()> {
        let mut chars: Vec<char> = input.chars().collect();
        while self.current < chars.len() {
            let c = chars[self.current];
            match c {
                '(' => self.add_token(Token {
                    token_type: TokenType::LEFT_PAREN,
                    lexeme: "(".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ')' => self.add_token(Token {
                    token_type: TokenType::RIGHT_PAREN,
                    lexeme: ")".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '{' => self.add_token(Token {
                    token_type: TokenType::LEFT_BRACE,
                    lexeme: "{".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '}' => self.add_token(Token {
                    token_type: TokenType::RIGHT_BRACE,
                    lexeme: "}".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ',' => self.add_token(Token {
                    token_type: TokenType::COMMA,
                    lexeme: ",".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '.' => self.add_token(Token {
                    token_type: TokenType::DOT,
                    lexeme: ".".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '-' => self.add_token(Token {
                    token_type: TokenType::MINUS,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '+' => self.add_token(Token {
                    token_type: TokenType::PLUS,
                    lexeme: "+".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ';' => self.add_token(Token {
                    token_type: TokenType::SEMICOLON,
                    lexeme: ";".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '*' => self.add_token(Token {
                    token_type: TokenType::STAR,
                    lexeme: "*".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '!' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::BANG_EQUAL,
                            lexeme: "!=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::BANG,
                            lexeme: "!".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '=' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::EQUAL_EQUAL,
                            lexeme: "==".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::EQUAL,
                            lexeme: "=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '<' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::LESS_EQUAL,
                            lexeme: "<=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::LESS,
                            lexeme: "<".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '>' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::GREATER_EQUAL,
                            lexeme: ">=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::GREATER,
                            lexeme: ">".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '/' => {
                    if self.peek_next(&chars) == '/' {
                        while self.current < chars.len() && chars[self.current] != '\n' {
                            self.current += 1;
                        }
                        self.line += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::SLASH,
                            lexeme: "/".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                x if ['\n'].contains(&x) => {
                    self.line += 1;
                }
                x if [' ', '\t', '\r'].contains(&x) => {
                    // Handle whitespace
                }
                '"' => {
                    match self.string(&chars) {
                        Ok(_) => (),
                        Err(e) => {
                            self.add_error(Box::new(e));
                        }
                    }
                }
                x if x.is_digit(10) => {
                    self.number(&chars);
                }
                x if x.is_alphabetic() || ['_'].contains(&x) => {
                    self.identifier(&chars);
                }
                _ => {
                    let err = InterpreterError::tokenizer_error(crate::error::TokenizerErrorKind::UnexpectedCharacter(c, self.line));
                    self.add_error(Box::new(err));
                }
            }
            self.current += 1;
        }

        self.tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: "".to_string(),
            literal: None,
            line: self.line,
        });
        Ok(())
    }

    fn add_error(&mut self, error: Box<dyn Error>) {
        eprintln!("{}", error);
        self.errors.push(error);
    }
    fn add_token(&mut self, token: Token) {
        self.tokens.push(token);
    }
    
    fn peek_next(&self, chars: &Vec<char>) -> char {
        if self.current + 1 >= chars.len() {
            '\0'
        } else {
            chars[self.current + 1]
        }
    }

    fn string(&mut self, chars: &Vec<char>) -> InterpreterResult<()> {

        self.current += 1;
        let start = self.current;
        while self.current < chars.len() && chars[self.current] != '"' {
            self.current += 1;
        }

        if self.current >= chars.len() {
            return Err(InterpreterError::tokenizer_error(crate::error::TokenizerErrorKind::UnterminatedString(self.line)));
        }

        let value = chars[start..self.current].iter().collect::<String>();
        self.tokens.push(Token {
            token_type: TokenType::STRING,
            lexeme: format!("\"{}\"", value.clone()),
            literal: Some(value),
            line: self.line,
        });
        Ok(())
    }

    fn number(&mut self, chars: &Vec<char>) {
        let start = self.current;
        let mut fractional_start = self.current;
        let mut fractional_length = 0;
        while self.current < chars.len() && chars[self.current].is_digit(10) {
            self.current += 1;
            fractional_start += 1;
        }
        if self.current < chars.len()
            && chars[self.current] == '.'
            && self.current + 1 < chars.len()
            && chars[self.current + 1].is_digit(10)
        {
            self.current += 1;
            while self.current < chars.len() && chars[self.current].is_digit(10) {
                self.current += 1;
                fractional_length += 1;
            }
        }
        let left = chars[start..fractional_start].iter().collect::<String>();
        let mut right = "".to_string();
        if (fractional_length > 0){
            right = chars[fractional_start+1..fractional_start+fractional_length+1].iter().collect::<String>();
            while right.ends_with('0') {
                right.pop();
            }
        }
        if right.is_empty(){
            right.push('0');
        }
        let mut value = left.clone();
        value.push_str(".");
        value.push_str(&right);
        let lexeme = chars[start..self.current].iter().collect::<String>();
        self.tokens.push(Token {
            token_type: TokenType::NUMBER,
            lexeme,
            literal: Some(value),
            line: self.line,
        });
        self.current -= 1;
    }

    fn identifier(&mut self, chars: &Vec<char>) {
        let start = self.current;
        while self.current < chars.len()
            && (chars[self.current].is_alphanumeric() || ['_', '-'].contains(&chars[self.current]))
        {
            self.current += 1;
        }
        let value = chars[start..self.current].iter().collect::<String>();
        let token_type = match value.as_str() {
            "and" => TokenType::AND,
            "class" => TokenType::CLASS,
            "else" => TokenType::ELSE,
            "false" => TokenType::FALSE,
            "for" => TokenType::FOR,
            "fun" => TokenType::FUN,
            "if" => TokenType::IF,
            "nil" => TokenType::NIL,
            "or" => TokenType::OR,
            "return" => TokenType::RETURN,
            "super" => TokenType::SUPER,
            "this" => TokenType::THIS,
            "true" => TokenType::TRUE,
            "var" => TokenType::VAR,
            "while" => TokenType::WHILE,
            "import" => TokenType::IMPORT,
            _ => TokenType::IDENTIFIER,
        };
        self.tokens.push(Token {
            token_type,
            lexeme: value.clone(),
            literal: None,
            line: self.line,
        });
        self.current -= 1;
    }
}