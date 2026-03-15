// TODO: Add messages to errors!
// TODO: Negative values will cause issues
//      - Solution: Handle them in when evaluating
// TODO: Floats will cause issues
//      - Solution let the language parser handle numeric conversion, 
//      - Just identify the slice that represents the number

use std::i32;
use std::str::FromStr;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};

use std::collections::HashMap;

pub enum Error{
    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
    ParseToken,
    InvalidChar,
    ParseAutomata,
    ParseTransition,
}

// The operator's precedence is it's value
// Higher value, higher precedence, happens before others
enum Operator {
    Add,
    Mull,
    None
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

// Helper for state/automaton consturction
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

struct Transition {
    from_id: usize,
    letter: TokenKind,
    to_id: usize
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

impl FromStr for Transition{
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_s: Vec<&str> = s.split_whitespace().collect();
        if split_s.len() != 3{
            return Err(Error::ParseTransition);
        }
        
        let from_id = split_s[0].parse::<usize>().map_err(|e| Error::ParseTransition)?;
        let letter = split_s[1].parse::<TokenKind>()?;
        let to_id = split_s[2].parse::<usize>().map_err(|e| Error::ParseTransition)?;

        Ok(Transition{from_id, letter, to_id})
    }
}

// usize is the id of another State
type NeighList = HashMap<TokenKind, usize>;
struct State{
    id: usize,
    is_final: bool,
    neigh: NeighList
}

impl State{
    fn new_final(id: usize) -> State{
        State{id: id, is_final: true, neigh: NeighList::new()}
    }

    fn add_neigh(&mut self, token: TokenKind, to_state_id: usize) -> Option<usize>{
        self.neigh.insert(token, to_state_id)
    }
}
struct Automata{
    states: Vec<State>,
}

impl Automata{
    fn print_info(&self){
        print!("[DBG]: States: ");
        for state in &self.states{
            print!("{} ", state.id);
        }

        println!("\n[DBG]: Transitions:");
        for state in &self.states{
            for (letter, neigh_id) in &state.neigh{
                print!("[DBG]: ({}, {}, {})\n",state.id, letter.to_str(), neigh_id);
            }
        }
    }

    fn new() -> Automata{
        Automata{states: Vec::new()}
    }

    fn load(path: &str) -> Result<Automata, Error> {
        let file = File::open(path).map_err(|e| Error::ParseAutomata)?; 
        let mut reader = BufReader::new(file);

        let mut automata = Automata::new();
        automata._load_states(&mut reader)?
                ._set_fin_states(&mut reader)?
                ._load_trans(&mut reader)?;
        Ok(automata)
    }

    // Loads the states in place and returns the updated automata
    fn _load_states(&mut self, reader: &mut BufReader<File>) -> Result<&mut Self, Error>{
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| Error::ParseAutomata)?;

        for state_id_str in line.split_whitespace() {
            let id = state_id_str
                .parse::<usize>()
                .map_err(|e| Error::ParseAutomata)?;

            self.states.push(State::new_final(id));
        }

        Ok(self)
    }

    // Loads the transitions in place and returns the automata
    fn _load_trans(&mut self, reader: &mut BufReader<File>) -> Result<&mut Self, Error>{
        for line in reader.lines(){
            let line = line.map_err(|e| Error::ParseAutomata)?;
            let tran = line.parse::<Transition>()?;

            // Invalid to state
            if tran.to_id >= self.states.len(){
                return Err(Error::ParseAutomata)
            }

            let from_state = self.states.get_mut(tran.from_id).ok_or(Error::ParseAutomata)?;

            from_state.add_neigh(tran.letter, tran.to_id);
        }
        Ok(self)
    }

    // Reads and sets the final states from the file, returns the updated automata
    fn _set_fin_states(&mut self, reader: &mut BufReader<File>) -> Result<&mut Self, Error>{
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|e| Error::ParseAutomata)?;

        for state_id_str in line.split_whitespace() {
            let id = state_id_str
                .parse::<usize>()
                .map_err(|e| Error::ParseAutomata)?;
            
            let state = self.states.get_mut(id).ok_or(Error::ParseAutomata)?;
            state.is_final = true;
        }

        Ok(self)
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
// enum MathExpr<'a>{
//     Val(i32),
//     Add(&'a MathExpr<'a>, &'a MathExpr<'a>),
// }

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
    let s = "  ( 5  + 3 + (2   ) )  ";
    let tokens = tokenize(s).ok();
    match tokens{
        None => {print_err("Tokenizer");}
        Some (tokens) => {print_tokens(&tokens);}
    }

    // Test automata
    let automata = Automata::load("automata.txt").ok();
    match automata{
        None => {print_err("Automata error");}
        Some (aut) => aut.print_info(),
    }
}
