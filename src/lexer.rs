use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::str;

use crate::error::Error;
use crate::common::{Value};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]

pub enum Operator {
    // Arithmetic
    Plus,
    Minus,
    Mull,
    Div,

    Less,
    LessEq,
    Greater,
    GreaterEq,
    Equal,
    NotEqual,

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
            "<=" => Ok(Operator::LessEq),
            ">" => Ok(Operator::Greater),
            ">=" => Ok(Operator::GreaterEq),
            "==" => Ok(Operator::Equal),
            "!=" => Ok(Operator::NotEqual),

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
            Operator::LessEq => write!(f, "<="),
            Operator::Greater => write!(f, ">"),
            Operator::GreaterEq => write!(f, ">="),
            Operator::Equal => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),

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

    // Tries to extend self to a multichar token, if it fails it returns itself
    // Ex: > becomes >=, ! becomes !=
    fn try_single_char_to_multi(&self) -> Token{
        match self{
            Token::Op(Operator::Greater) => Token::Op(Operator::GreaterEq),
            Token::Op(Operator::Less) => Token::Op(Operator::LessEq),
            Token::Op(Operator::Not) => Token::Op(Operator::NotEqual),
            Token::Op(Operator::Assign) => Token::Op(Operator::Equal),
            _ => self.clone()
        }
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

#[derive(Debug)]
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


pub struct Lexer {
    s: Vec<u8>,
    str_start_i: usize,
    cursor : usize,
}

impl Lexer{
    pub fn new(s: &str) -> Lexer{
        Lexer { s : s.to_string().into_bytes(), str_start_i : 0, cursor : 0}
    }

    pub fn consume_str(&mut self) -> Option<&[u8]>{
        if self.str_start_i >= self.cursor - 1{
            self.str_start_i = self.cursor;
            return None
        }

        let token_str = &self.s[self.str_start_i..self.cursor - 1];
        self.str_start_i = self.cursor;
    
        return Some(token_str);
    }

    pub fn consume(&mut self) -> Option<u8>{
        if self.cursor >= self.s.len(){
            return None;
        }

        let ret = Some(self.s[self.cursor]);
        self.cursor += 1;
        ret
    }

    pub fn peek(&mut self) -> Option<u8>{
        if self.cursor >= self.s.len(){
            return None;
        }
        return Some(self.s[self.cursor]);
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, Error> {
        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = self.consume() {
            let parsed_char = ValidChar::from_char(c as char);
            match parsed_char {
                Ok(ValidChar::Sep) => {
                    if let Some(token_str) = self.consume_str(){
                        let token = Token::parse_str(unsafe { str::from_utf8_unchecked(token_str)})?;
                        tokens.push(token);
                    }
                }

                Ok(ValidChar::TokenChar(mut curr_token)) => {
                    // Maybe we went past a variable or value
                    if let Some(passed_token_str) = self.consume_str(){
                        let passed_token = Token::parse_str(unsafe { str::from_utf8_unchecked(passed_token_str)})?;
                        tokens.push(passed_token);
                    }

                    // Change the token to multichar if needed
                    let next_c = self.peek();
                    if let Some(next_c) = next_c && next_c == b'=' {
                        curr_token = curr_token.try_single_char_to_multi();
                        self.consume();
                        self.consume_str();
                    }

                    // Plus the token
                    tokens.push(curr_token);
                }

                _ => {continue;}
            }
        }

        // Maybe the last token was multi character
        if let Some(token_str) = self.consume_str(){
            let token = Token::parse_str(unsafe { str::from_utf8_unchecked(token_str)})?;
            tokens.push(token);
        }

        Ok(tokens)
    }

}
pub fn print_tokens(tokens: &Vec<Token>) {
    print!("[DBG]: Tokens: [");
    for token in tokens {
        print!("'{}', ", token);
    }
    println!("]");
}
