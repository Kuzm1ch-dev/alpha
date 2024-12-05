use std::error::Error;

use crate::error::{InterpreterError, InterpreterResult};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Modulo,
    Comma,
    Dict,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BandEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    IDENTIfIER,
    STRING,
    Number,
    And,
    Class,
    New,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Return,
    Super,
    True,
    Try,
    Catch,
    Var,
    While,
    Eof,
    Import,
    Async,
    Await
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
        let chars: Vec<char> = input.chars().collect();
        while self.current < chars.len() {
            let c = chars[self.current];
            match c {
                '(' => self.add_token(Token {
                    token_type: TokenType::LeftParen,
                    lexeme: "(".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ')' => self.add_token(Token {
                    token_type: TokenType::RightParen,
                    lexeme: ")".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '[' => self.add_token(Token {
                    token_type: TokenType::LeftBracket,
                    lexeme: "[".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ']' => self.add_token(Token {
                    token_type: TokenType::RightBracket,
                    lexeme: "]".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '{' => self.add_token(Token {
                    token_type: TokenType::LeftBrace,
                    lexeme: "{".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '}' => self.add_token(Token {
                    token_type: TokenType::RightBrace,
                    lexeme: "}".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ':' => self.add_token(Token {
                    token_type: TokenType::Colon,
                    lexeme: ":".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ',' => self.add_token(Token {
                    token_type: TokenType::Comma,
                    lexeme: ",".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '.' => self.add_token(Token {
                    token_type: TokenType::Dot,
                    lexeme: ".".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '-' => self.add_token(Token {
                    token_type: TokenType::Minus,
                    lexeme: "-".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '%' => self.add_token(Token {
                    token_type: TokenType::Modulo,
                    lexeme: "%".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '+' => self.add_token(Token {
                    token_type: TokenType::Plus,
                    lexeme: "+".to_string(),
                    literal: None,
                    line: self.line,
                }),
                ';' => self.add_token(Token {
                    token_type: TokenType::Semicolon,
                    lexeme: ";".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '*' => self.add_token(Token {
                    token_type: TokenType::Star,
                    lexeme: "*".to_string(),
                    literal: None,
                    line: self.line,
                }),
                '!' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::BandEqual,
                            lexeme: "!=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::Bang,
                            lexeme: "!".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '=' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::EqualEqual,
                            lexeme: "==".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::Equal,
                            lexeme: "=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '<' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::LessEqual,
                            lexeme: "<=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::Less,
                            lexeme: "<".to_string(),
                            literal: None,
                            line: self.line,
                        });
                    }
                }
                '>' => {
                    if self.peek_next(&chars) == '=' {
                        self.add_token(Token {
                            token_type: TokenType::GreaterEqual,
                            lexeme: ">=".to_string(),
                            literal: None,
                            line: self.line,
                        });
                        self.current += 1;
                    } else {
                        self.add_token(Token {
                            token_type: TokenType::Greater,
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
                            token_type: TokenType::Slash,
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
            token_type: TokenType::Eof,
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
        if fractional_length > 0 {
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
            token_type: TokenType::Number,
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
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "new" => TokenType::New,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "dict" => TokenType::Dict,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "return" => TokenType::Return,
            "try" => TokenType::Try,
            "catch" => TokenType::Catch,
            "super" => TokenType::Super,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            "import" => TokenType::Import,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            _ => TokenType::IDENTIfIER,
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