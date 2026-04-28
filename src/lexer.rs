use regex::Regex;
use std::fmt;
use std::str::FromStr;

use crate::error::Error;
use crate::common::{Value};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]

pub enum Operator {
    // Arithmetic
    Plus,
    Minus,
    Mull,
    Div,

    // Comparison(TODO: Add <=, >= and !=)
    Less,
    Greater,
    Equal,

    // Logical operators
    And,
    Or,
    Not,

    // Assigment operator
    Assign
}


impl FromStr for Operator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Operator::Plus),
            "-" => Ok(Operator::Minus),
            "*" => Ok(Operator::Mull),
            "/" => Ok(Operator::Div),

            "<" => Ok(Operator::Less),
            ">" => Ok(Operator::Greater),
            "==" => Ok(Operator::Equal),

            "&" => Ok(Operator::And),
            "|" => Ok(Operator::Or),
            "!" => Ok(Operator::Not),

            "=" => Ok(Operator::Assign),
            
            _ => Err(Error::ParseToken),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Mull => write!(f, "*"),
            Operator::Div => write!(f, "/"),

            Operator::Less => write!(f, "<"),
            Operator::Greater => write!(f, ">"),
            Operator::Equal => write!(f, "=="),

            Operator::And => write!(f, "&"),
            Operator::Or => write!(f, "|"),
            Operator::Not => write!(f, "!"),

            Operator::Assign => write!(f, "="),
        }
    }
}


#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Parenthesis {
    Open,
    Closed,
}

impl FromStr for Parenthesis {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "(" => Ok(Parenthesis::Open),
            ")" => Ok(Parenthesis::Closed),
            _ => Err(Error::ParseToken),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum Token {
    Val(Value),
    Var(String),
    Op(Operator),
    Par(Parenthesis),
}

impl Token {
    // Returns a token if char is by itself one(operator or parantheses), an error otherwise
    fn parse_char(c: char) -> Result<Token, Error> {
        let s = c.to_string();
        s.parse::<Parenthesis>()
            .map(Token::Par)
            .or_else(|_| s.parse::<Operator>().map(Token::Op))
    }

    // Should be used to parse multi character tokens like values, variables
    fn parse_str(s: &str) -> Result<Token, Error> {
        // First try to convert to true or false to not mistake it for a variable
        if let Ok(b) = s.parse::<bool>(){
            return Ok(Token::Val(Value::Bool(b)))
        }

        // Variables
        let var_re = Regex::new("^[A-Za-z_][A-Za-z_0-9]*$").unwrap();
        if var_re.is_match(s) {
            return Ok(Token::Var(s.to_string()));
        }

        // Numeric values
        let val_re = Regex::new("^[1-9][0-9]*$").unwrap();
        s.parse::<i32>()
            .map(|n| Token::Val(Value::Int(n))) 
            .map_err(|_| {
                if val_re.is_match(s) {
                    Error::ValOutOfBounds
                } else {
                    Error::ParseToken
                }
            })
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Val(val) => write!(f, "{}", val),
            Token::Var(var) => write!(f, "{}", var),
            Token::Op(op) => write!(f, "{}", op),
            Token::Par(Parenthesis::Open) => write!(f, "("),
            Token::Par(Parenthesis::Closed) => write!(f, ")"),
        }
    }
}

enum ValidChar {
    TokenChar(Token),
    Sep,
}

impl ValidChar {
    fn from_char(c: char) -> Result<Self, Error> {
        if c == ' ' || c == '\n' || c == '\r' {
            return Ok(ValidChar::Sep);
        }

        Token::parse_char(c).map(ValidChar::TokenChar)
    }
}


// TODO: Handle floats as well
pub fn tokenize(s: &str) -> Result<Vec<Token>, Error> {
    println!("[DBG]: tokenize: {}", s);
    let mut tokens: Vec<Token> = Vec::new();

    let mut start_i = 0;
    let mut iter = s.char_indices().peekable();
    
    while let Some((i, c)) = iter.next() {
        let parsed_char = ValidChar::from_char(c);

        match parsed_char {
            Ok(ValidChar::Sep) => {
                // Maybe we went passed a variable or value?
                let token_str = &s[start_i..i];
                start_i = i + 1;
                if token_str.is_empty() {
                    continue;
                }
                let token = Token::parse_str(token_str)?;
                tokens.push(token);
            }
            // Special case for = as it might be ==
            Ok(ValidChar::TokenChar(Token::Op(Operator::Assign))) => {
                let token_str = &s[start_i..i];
                start_i = i + 1;
                let mut op = Token::Op(Operator::Assign);

                if let Some(&(next_i, next_c)) = iter.peek(){
                    if next_c == '=' {
                        op = Token::Op(Operator::Equal);
                        start_i += 1;
                        iter.next();
                    }
                }

                if token_str.is_empty() {
                    // Plus the single-char token
                    tokens.push(op);
                    continue;
                }

                let token = Token::parse_str(token_str)?;
                tokens.push(token);

                // Plus the single-char token
                tokens.push(op);
            }

            Ok(ValidChar::TokenChar(t)) => {
                // Maybe we went past a variable or value
                let token_str = &s[start_i..i];
                start_i = i + 1;
                if token_str.is_empty() {
                    // Plus the single-char token
                    tokens.push(t);
                    continue;
                }
                let token = Token::parse_str(token_str)?;
                tokens.push(token);

                // Plus the single-char token
                tokens.push(t);
            }

            _ => {
                continue;
            }
        }
    }

    // Maybe the last token was multi character
    let token_str = &s[start_i..];
    if !token_str.is_empty() {
        let token = Token::parse_str(&s[start_i..])?;
        tokens.push(token);
    }

    Ok(tokens)
}

pub fn print_tokens(tokens: &Vec<Token>) {
    print!("[DBG]: Tokens: [");
    for token in tokens {
        print!("'{}', ", token);
    }
    println!("]");
}
