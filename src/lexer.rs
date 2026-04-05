use regex::Regex;
use std::fmt;
use std::str::FromStr;

use crate::error::Error;

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum MathOperator {
    Plus,
    Minus,
    Mull,
    Div,
}

impl FromStr for MathOperator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(MathOperator::Plus),
            "-" => Ok(MathOperator::Minus),
            "*" => Ok(MathOperator::Mull),
            "/" => Ok(MathOperator::Div),
            _ => Err(Error::ParseToken),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Operator {
    Math(MathOperator),
    Attrib,
}

impl FromStr for Operator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<MathOperator>()
            .map(Operator::Math)
            .or_else(|_| match s {
                "=" => Ok(Operator::Attrib),
                _ => Err(Error::ParseToken),
            })
    }
}

impl fmt::Display for MathOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathOperator::Plus => write!(f, "+"),
            MathOperator::Minus => write!(f, "-"),
            MathOperator::Mull => write!(f, "*"),
            MathOperator::Div => write!(f, "/"),
        }
    }
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Operator::Math(op) => write!(f, "{}", op),
            Operator::Attrib => write!(f, "="),
        }
    }
}

impl MathOperator {
    pub fn get_precedence(&self) -> i32 {
        match self {
            MathOperator::Plus => 0,
            MathOperator::Minus => 0,
            MathOperator::Mull => 1,
            MathOperator::Div => 1,
        }
    }

    pub fn apply(&self, val1: i32, val2: i32) -> i32 {
        match self {
            MathOperator::Plus => val1 + val2,
            MathOperator::Minus => val1 - val2,
            MathOperator::Mull => val1 * val2,
            MathOperator::Div => val1 / val2,
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
    Val(i32),
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

    // Should be used to parse multi character tokens like values and variables
    fn parse_str(s: &str) -> Result<Token, Error> {
        let var_re = Regex::new("^[A-Za-z_][A-Za-z_0-9]*$").unwrap();
        if var_re.is_match(s) {
            return Ok(Token::Var(s.to_string()));
        }

        let val_re = Regex::new("^[1-9][0-9]*$").unwrap();
        s.parse::<i32>().map(Token::Val).map_err(|_| {
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
    for (i, c) in s.char_indices() {
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
