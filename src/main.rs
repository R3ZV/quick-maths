// TODO: Plus messages to errors!
// TODO: Negative values will cause issues
//      - Solution: Handle them in when evaluating
// TODO: Floats will cause issues
//      - Solution let the language parser handle numeric conversion,
//      - Just identify the slice that represents the number
//TODO: Why don't consume and current work with refereces?
//TODO: Make parse functions for operators and parant that accepts a single char
//TODO: Refactor tokenize a bit
use regex::Regex;
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::io::{Write, stdin, stdout};
use std::str::FromStr;
// #[derive(Debug)]
// struct ErrInfo{
//     instr: String,
//     col: usize,
// }

// impl fmt::Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             Error::ParseToken(Some(err_info)) => write!(f, "[ERR]: Parse Token: {}", err_info.col),
//         }
//     }
// }

#[derive(Debug)]
pub enum Error {
    ParseToken,
    UnexpectedToken,
    ValOutOfBounds,
    NotAttrib,
    UndeclaredVar,
    None,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum MathOperator {
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
enum Operator {
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
    fn get_precedence(&self) -> i32 {
        match self {
            MathOperator::Plus => 0,
            MathOperator::Minus => 0,
            MathOperator::Mull => 1,
            MathOperator::Div => 1,
        }
    }

    fn apply(&self, val1: i32, val2: i32) -> i32 {
        match self {
            MathOperator::Plus => val1 + val2,
            MathOperator::Minus => val1 - val2,
            MathOperator::Mull => val1 * val2,
            MathOperator::Div => val1 / val2,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Parenthesis {
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
enum Token {
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
        s.parse::<i32>()
            .map(Token::Val)
            .map_err(|_| 
                if val_re.is_match(s){Error::ValOutOfBounds}
                else{Error::ParseToken}
            )
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
fn tokenize(s: &str) -> Result<Vec<Token>, Error> {
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

enum Expr {
    Math(MathExpr),
    Attrib(String, MathExpr),
}

impl Expr {
    fn eval(self, pgm_state: &mut HashMap<String, i32>) -> Result<i32, Error> {
        match self {
            Expr::Math(m_expr) => m_expr.eval(pgm_state),
            Expr::Attrib(var, m_expr) => {
                let right = m_expr.eval(pgm_state)?;
                pgm_state.insert(var, right);
                Ok(right)
            }
        }
    }
}

enum MathExpr {
    Val(i32),
    Var(String),
    BinOp(MathOperator, Box<MathExpr>, Box<MathExpr>),
    UnMinus(Box<MathExpr>)
}

impl fmt::Display for MathExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathExpr::Val(val) => write!(f, "{}", val),
            MathExpr::Var(var) => write!(f, "{}", var),
            MathExpr::BinOp(op, e1, e2) => {
                write!(f, "{} ({}, {})", op, e1, e2)
            }
            MathExpr::UnMinus(e) => write!(f, "-{}", e),
        }
    }
}

impl MathExpr {
    fn eval(&self, pgm_state: &HashMap<String, i32>) -> Result<i32, Error> {
        match self {
            MathExpr::Val(val) => Ok(*val),
            MathExpr::Var(var) => pgm_state.get(var).copied().ok_or(Error::UndeclaredVar),
            MathExpr::BinOp(op, e1, e2) => {
                let v1 = e1.eval(pgm_state)?;
                let v2 = e2.eval(pgm_state)?;
                Ok(op.apply(v1, v2))
            }
            MathExpr::UnMinus(e) => {
                Ok(-e.eval(pgm_state)?)
            }
        }
    }
}

struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, cursor: 0 }
    }

    fn reset(&mut self) {
        self.cursor = 0;
    }

    fn consume(&mut self) -> Option<Token> {
        if self.cursor >= self.tokens.len() {
            return None;
        }

        let res = self.tokens[self.cursor].clone();
        self.cursor += 1;
        Some(res)
    }

    fn current(&self) -> Option<Token> {
        self.tokens.get(self.cursor).cloned()
    }

    fn parse_primary(&mut self) -> Result<MathExpr, Error> {
        let token = self.consume().ok_or(Error::UnexpectedToken)?;

        match token {
            Token::Val(val) => Ok(MathExpr::Val(val)),
            Token::Var(var) => Ok(MathExpr::Var(var)),
            Token::Par(Parenthesis::Open) => {
                let expr = self.parse_math_expr();
                if self.consume() != Some(Token::Par(Parenthesis::Closed)) {
                    return Err(Error::UnexpectedToken);
                }
                expr
            }
            _ => Err(Error::UnexpectedToken),
        }
    }

    fn parse_factor(&mut self) -> Result<MathExpr, Error>{
        match self.current(){
            Some(Token::Op(Operator::Math(MathOperator::Minus))) => {
                self.consume();
                Ok(MathExpr::UnMinus(Box::new(self.parse_primary()?)))
            }
            None => {Err(Error::ParseToken)}
            _ => self.parse_primary()
        }
    }

    fn parse_bin(&mut self, left: MathExpr, op: MathOperator) -> Result<MathExpr, Error> {
        let mut right = self.parse_factor()?;

        loop {
            let token = self.current();
            match token {
                Some(Token::Op(Operator::Math(next_op))) => {
                    if next_op.get_precedence() > op.get_precedence() {
                        self.consume();
                        right = self.parse_bin(right, next_op)?;
                    } else {
                        break;
                    }
                }
                Some(Token::Op(Operator::Attrib)) => {
                    return Err(Error::UnexpectedToken);
                }
                _ => {
                    break;
                }
            }
        }

        Ok(MathExpr::BinOp(op, Box::new(left), Box::new(right)))
    }

    fn parse_math_expr(&mut self) -> Result<MathExpr, Error> {
        let mut left = self.parse_factor()?;
        loop {
            let token = self.current();
            match token {
                Some(Token::Op(Operator::Math(op))) => {
                    self.consume();
                    left = self.parse_bin(left, op)?;
                }
                Some(Token::Op(Operator::Attrib)) => {
                    return Err(Error::UnexpectedToken);
                }
                _ => {
                    break;
                }
            }
        }
        Ok(left)
    }

    fn parse_attrib(&mut self) -> Result<Expr, Error> {
        let left = self.consume().ok_or(Error::UnexpectedToken)?;
        match left {
            Token::Var(var) => {
                let next = self.consume().ok_or(Error::NotAttrib)?;
                if next != Token::Op(Operator::Attrib) {
                    return Err(Error::NotAttrib);
                }
                Ok(Expr::Attrib(var, self.parse_math_expr()?))
            }
            _ => Err(Error::NotAttrib),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, Error> {
        match self.parse_attrib() {
            Err(Error::NotAttrib) => {
                self.reset();
                self.parse_math_expr().map(Expr::Math)
            }
            other => other,
        }
    }
}

struct Interpreter {
    pgm_state: HashMap<String, i32>,
}

impl Interpreter {
    fn new() -> Self {
        Interpreter {
            pgm_state: HashMap::new(),
        }
    }

    fn run(&mut self, instr: &str) -> Result<i32, Error> {
        let tokens = tokenize(instr)?;
        print_tokens(&tokens);

        let mut parser: Parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        println!("[DBG]: Passed parser!");

        expr.eval(&mut self.pgm_state)
    }
}

fn print_err(err: &str) {
    println!("[ERR]: {}", err);
}

fn print_tokens(tokens: &Vec<Token>) {
    print!("[DBG]: Tokens: [");
    for token in tokens {
        print!("'{}', ", token);
    }
    println!("]");
}

fn main() {
    let mut interpreter: Interpreter = Interpreter::new();
    println!("You are in the quick maths interactive shell!\n");
    loop {
        print!(">> ");

        // Get user instruction
        let mut instr = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut instr)
            .expect("Did not enter a correct string");
        
        // println!("[DBG]: {}", instr);
        
        if instr == "q"{
            println!("Exiting...");
            return;
        }

        let instr_res = interpreter.run(&instr);

        match instr_res {
            Ok(val) => {
                println!("{}", val)
            }
            Err(_) => print_err("Amogus"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn prop_tokenizer_never_panics(input: String) -> bool {
        let _ = tokenize(&input);
        true
    }

    #[quickcheck]
    fn prop_interpreter_never_panics_on_garbage(input: String) -> bool {
        let mut interpreter = Interpreter::new();
        let _ = interpreter.run(&input);
        true
    }

    #[quickcheck]
    fn prop_Plusition_is_commutative(a: i32, b: i32) -> TestResult {
        if a.checked_add(b).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }
}
