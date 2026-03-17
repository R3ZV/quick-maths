// TODO: Add messages to errors!
// TODO: Negative values will cause issues
//      - Solution: Handle them in when evaluating
// TODO: Floats will cause issues
//      - Solution let the language parser handle numeric conversion, 
//      - Just identify the slice that represents the number

use std::i32;
use std::str::FromStr;
use std::hash::Hash;

pub enum Error{
    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
    InvalidState,
    ParseToken,
    InvalidChar,
    UnexpectedToken,
    ParseAutomata,
    ParseTransition,
}


#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
// The operator's precedence is it's value
// Higher value, higher precedence, happens before others
enum Operator {
    Add,
    Mull,
}

impl FromStr for Operator {
    type Err = Error; // We must specify the error type

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+" => Ok(Operator::Add),
            "*" => Ok(Operator::Mull),
            _ => Err(Error::ParseToken),
        }
    }
}

impl Operator{
    fn get_precedence(&self) -> i32{
        match self{
            Operator::Add => 0,
            Operator::Mull => 1
        }
    }

    fn to_str(&self) -> String{
        match self{
            Operator::Add => "+".to_string(),
            Operator::Mull => "*".to_string()
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

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum TokenKind {
    Val,
    Op,
    ParOpen,
    ParClosed
}

impl TokenKind{
    fn to_str(&self) -> &str {
        match self {
            TokenKind::Val => "Val",
            TokenKind::Op => "Op",
            TokenKind::ParOpen => "(",
            TokenKind::ParClosed => ")"
        }
    }
}

impl FromStr for TokenKind{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Val" => Ok(TokenKind::Val),
            "Op" => Ok(TokenKind::Op),
            "(" => Ok(TokenKind::ParOpen),
            ")" => Ok(TokenKind::ParClosed),
            _ => Err(Error::ParseAutomata),
        }
    }
   
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
enum Token {
    Val(i32),
    Op(Operator),
    Par(Parenthesis)
}

impl Token {
    fn kind(&self) -> TokenKind {
        match self {
            Token::Val(_) => TokenKind::Val,
            Token::Op(_) => TokenKind::Op,
            Token::Par(Parenthesis::Open) => TokenKind::ParOpen,
            Token::Par(Parenthesis::Closed) => TokenKind::ParClosed,
        }
    }

    fn to_string(&self) -> String{
        match self {
            Token::Val(n) => n.to_string(),
            Token::Op(_) => "+".to_string(),
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

enum MathExpr{
    Val(i32),
    BinOp(Operator, Box<MathExpr>, Box<MathExpr>),
}

impl MathExpr{
    fn to_str(&self) -> String{
        match self {
            MathExpr::Val(v) => {format!("{}", v)},
            MathExpr::BinOp(op, e1 , e2) => {
                format!("{} ({}, {})", op.to_str(), e1.to_str(), e2.to_str())
            }
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

    fn consume(&mut self) -> Option<Token>{
        if self.cursor >= self.tokens.len(){
            return None;
        }

        let res = self.tokens[self.cursor];
        self.cursor += 1;
        Some(res)
    }

    fn current(&self) -> Option<Token>{
        self.tokens.get(self.cursor).copied()
    }

    fn parse_primary(&mut self) -> Result<MathExpr, Error>{
        let token = self.consume().ok_or(Error::UnexpectedToken)?;

        match token{
            Token::Val(v) => {Ok(MathExpr::Val(v))},
            Token::Par(Parenthesis::Open) => {
                let expr = self.parse_expr();
                if self.consume() != Some(Token::Par(Parenthesis::Closed)){
                    return Err(Error::UnexpectedToken);
                }
                return expr;
            }
            _ => {Err(Error::UnexpectedToken)}
        }
    }

    fn parse_bin(&mut self, left: MathExpr, op: Operator) -> Result<MathExpr, Error>{
        let mut right = self.parse_primary()?;

        loop{
            let token = self.current();
            match token{
                Some(Token::Op(next_op)) => {
                    if next_op.get_precedence() > op.get_precedence(){
                        self.consume();
                        right = self.parse_bin(right, next_op)?;
                    } else {
                        break;
                    }
                }
                _ =>{break;}
            }
        }

        Ok(MathExpr::BinOp(op, Box::new(left), Box::new(right)))
    }

    fn parse_expr(&mut self) -> Result<MathExpr, Error>{
        let mut left = self.parse_primary()?;
        loop{
            let token = self.current();
            match token{
                Some(Token::Op(op)) => {
                    self.consume();
                    left = self.parse_bin(left, op)?;
                }
                _ =>{break;}
            }
        }
        Ok(left)
    }
}

// impl MathExpr<'a>{
//     fn from_str<'a>(str_expr: &str) -> MathExpr<'a>{
//         for token in str_expr.split(" "){
//             if (token.parse())
//         }
//     }
// }

fn print_err(err: &str){
    println!("[ERR]: {}", err);
}

fn print_tokens(tokens: &Vec<Token>){
    print!("[DBG]: Tokens: ");
    for token in tokens{
        print!("{}", token.to_string());
    }
    println!();
}

fn main() {
    // Test tokenizer
    let s = " ( 3 +  5) +  2 * 7 ";
    let tokens = tokenize(s).ok().unwrap();
    // match tokens{
    //     None => {print_err("Tokenizer");}
    //     Some (tokens) => {print_tokens(&tokens);}
    // }

    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr().ok().unwrap();
    println!("{}", expr.to_str())
}
