// TODO: Add messages to errors!
// TODO: Negative values will cause issues
//      - Solution: Handle them in when evaluating
// TODO: Floats will cause issues
//      - Solution let the language parser handle numeric conversion, 
//      - Just identify the slice that represents the number
//TODO: Why don't consume and current work with refereces?
use std::fmt;
use std::io;
use std::i32;
use std::str::FromStr;
use std::hash::Hash;
use regex::Regex;

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
pub enum Error{
    ParseToken,
    UnexpectedToken,
    NotAttrib,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum MathOperator {
    Add,
    Subt,
    Mull,
    Div
}

impl FromStr for MathOperator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(MathOperator::Add),
            "-" => Ok(MathOperator::Subt),
            "*" => Ok(MathOperator::Mull),
            "/" => Ok(MathOperator::Div),
            _ => Err(Error::ParseToken),
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Operator {
    Math(MathOperator),
    Attrib
}

impl FromStr for Operator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<MathOperator>()
            .map(Operator::Math)
            .or_else(|_|
                match s {
                    "=" => Ok(Operator::Attrib),
                    _ => Err(Error::ParseToken),
                }
        )
    }
}

impl Operator{
    fn to_str(&self) -> String{
        match self{
            Operator::Math(op) => op.to_str(),
            Operator::Attrib => "=".to_string(),
        }
    }

    // fn apply(&self, expr1: Expr, expr2: Expr) {

    // }
}

impl MathOperator{
    fn get_precedence(&self) -> i32{
        match self{
            MathOperator::Add => 0,
            MathOperator::Subt => 0,
            MathOperator::Mull => 1,
            MathOperator::Div => 1
        }
    }

    fn to_str(&self) -> String{
        match self{
            MathOperator::Add => "+".to_string(),
            MathOperator::Subt => "-".to_string(),
            MathOperator::Mull => "*".to_string(),
            MathOperator::Div => "/".to_string(),
        }
    }

    fn apply(&self, val1: i32, val2: i32) -> i32{
        match self{
            MathOperator::Add => val1 + val2,
            MathOperator::Subt => val1 - val2,
            MathOperator::Mull => val1 * val2,
            MathOperator::Div => val1 / val2
        }
    }
}


#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Parenthesis {
    Open,
    Closed
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
struct Variable{
    name: String,
    val: i32
}

impl FromStr for Variable{
    type Err = Error;
    
    // Expects str to be the name of the variable!
    fn from_str(s: &str) -> Result<Self, Self::Err>{
        //TODO; How do I make this static?
        let re = Regex::new("^[A-Za-z_][A-Za-z_0-9]*$").map_err(|e| Error::ParseToken)?;
        if !re.is_match(s){
            return Err(Error::ParseToken)
        }
        return Ok(Variable{name: s.to_string(), val: 0})
    }
}


#[derive(Debug, Hash, Eq, PartialEq, Clone)]
enum Token {
    Val(i32),
    Var(Variable),
    Op(Operator),
    Par(Parenthesis),
}

impl Token {
    fn to_str(&self) -> String{
        match self {
            Token::Val(val) => val.to_string(),
            Token::Var(var) => var.name.to_string(),
            Token::Op(op) => op.to_str(),
            Token::Par(Parenthesis::Open) => "(".to_string(),
            Token::Par(Parenthesis::Closed) => ")".to_string(),
        }
    }
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i32>()
            .map(Token::Val)
            .or_else(|_| s.parse::<Parenthesis>().map(Token::Par))
            .or_else(|_| s.parse::<Operator>().map(Token::Op))
            .or_else(|_| s.parse::<Variable>().map(Token::Var))
    }
}

enum ValidChar{
    TokenChar(Token),
    Sep
}

impl ValidChar {
    fn from_char(c: char) -> Result<Self, Error> {
        if c == ' '{
            return Ok(ValidChar::Sep)
        }

        c.to_string().parse::<Token>().map(ValidChar::TokenChar)
    }
}

// TODO: Handle floats as well
fn tokenize(s: &str) -> Result<Vec<Token>, Error> {
    let mut tokens: Vec<Token> = Vec::new();

    let mut curr_nr: Option<i32> = None;

    for c in s.chars() {
        let parsed_char = ValidChar::from_char(c)?;

        match parsed_char {
            ValidChar::TokenChar(Token::Val(d)) => {
                curr_nr = Some(curr_nr.unwrap_or(0) * 10 + d);
            }
            
            ValidChar::Sep => {
                if let Some(num) = curr_nr.take() {
                    tokens.push(Token::Val(num));
                }
            }
            
            ValidChar::TokenChar(t) => {
                if let Some(num) = curr_nr.take() {
                    tokens.push(Token::Val(num));
                }
                tokens.push(t);
            }
        }
    }

    // Possibly ending with a number
    if let Some(num) = curr_nr.take() {
        tokens.push(Token::Val(num));
    }

    Ok(tokens)
}

enum Expr{
    Math(MathExpr),
    Attrib(Variable, MathExpr)
}

enum MathExpr{
    Val(i32),
    Var(Variable),
    BinOp(MathOperator, Box<MathExpr>, Box<MathExpr>),
}

impl MathExpr{
    fn to_str(&self) -> String{
        match self {
            MathExpr::Val(val) => {format!("{}", val.to_string())},
            MathExpr::Var(var) => {var.name.to_string()},
            MathExpr::BinOp(op, e1 , e2) => {
                format!("{} ({}, {})", op.to_str(), e1.to_str(), e2.to_str())
            },
        }
    }

    fn eval(&self) -> i32{
        match self{
            MathExpr::Val(val) => {*val},
            MathExpr::Var(var) => {var.val},
            MathExpr::BinOp(op, e1, e2) => op.apply(e1.eval(), e2.eval())
        }
    }
}

struct Parser{
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser{
    fn new(tokens: Vec<Token>) -> Parser{
        Parser{tokens: tokens, cursor: 0}
    }

    fn reset(&mut self){
        self.cursor = 0;
    }

    fn consume(&mut self) -> Option<Token>{
        if self.cursor >= self.tokens.len(){
            return None;
        }

        let res = self.tokens[self.cursor].clone();
        self.cursor += 1;
        Some(res)
    }

    fn current(&self) -> Option<Token>{
        self.tokens.get(self.cursor).cloned()
    }

    fn parse_primary(&mut self) -> Result<MathExpr, Error>{
        let token = self.consume().ok_or(Error::UnexpectedToken)?;

        match token{
            Token::Val(val) => {Ok(MathExpr::Val(val))},
            Token::Var(var) => {Ok(MathExpr::Var(var))}
            Token::Par(Parenthesis::Open) => {
                let expr = self.parse_math_expr();
                if self.consume() != Some(Token::Par(Parenthesis::Closed)){
                    return Err(Error::UnexpectedToken);
                }
                return expr;
            }
            _ => {Err(Error::UnexpectedToken)}
        }
    }

    fn parse_bin(&mut self, left: MathExpr, op: MathOperator) -> Result<MathExpr, Error>{
        let mut right = self.parse_primary()?;

        loop{
            let token = self.current();
            match token{
                Some(Token::Op(Operator::Math(next_op))) => {
                    if next_op.get_precedence() > op.get_precedence(){
                        self.consume();
                        right = self.parse_bin(right, next_op)?;
                    } else {
                        break;
                    }
                }
                Some(Token::Op(Operator::Attrib)) => {return Err(Error::UnexpectedToken);}
                _ =>{break;}
            }
        }

        Ok(MathExpr::BinOp(op, Box::new(left), Box::new(right)))
    }

    fn parse_math_expr(&mut self) -> Result<MathExpr, Error>{
        let mut left = self.parse_primary()?;
        loop{
            let token = self.current();
            match token{
                Some(Token::Op(Operator::Math(op))) => {
                    self.consume();
                    left = self.parse_bin(left, op)?;
                }
                Some(Token::Op(Operator::Attrib)) => {return Err(Error::UnexpectedToken);}
                _ =>{break;}
            }
        }
        Ok(left)
    }

    fn parse_attrib(&mut self) -> Result<Expr, Error>{
        let left = self.consume().ok_or(Error::UnexpectedToken)?;
        match left{
            Token::Var(var) => {
                let next = self.consume().ok_or(Error::NotAttrib)?;
                if next != Token::Op(Operator::Attrib){
                    return Err(Error::NotAttrib);
                }
                return Ok(Expr::Attrib(var, self.parse_math_expr()?));
            }
            _ => Err(Error::NotAttrib)
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, Error>{
        match self.parse_attrib() {
            Err(Error::NotAttrib) => {
                print!("Not Attrrib!");
                self.reset();
                self.parse_math_expr().map(Expr::Math)
            }
            other => other, 
        }   
    }

}


fn print_err(err: &str){
    println!("[ERR]: {}", err);
}

fn print_tokens(tokens: &Vec<Token>){
    print!("[DBG]: Tokens: ");
    for token in tokens{
        print!("{}", token.to_str());
    }
    println!();
}

fn main() {
    // Test tokenizer
    let s = "a = -5";
    let tokens = tokenize(s).ok().unwrap();
    print_tokens(&tokens);

    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr().ok().unwrap();
    // println!("{}={}", expr.to_str(), expr.eval());
}
