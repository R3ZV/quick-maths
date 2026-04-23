use crate::ast::{Expr, MathExpr};
use crate::error::Error;
use crate::lexer::{Operator, Parenthesis, Token};

pub enum UnaryOp{
    Not,
    Minus,
}

impl TryFrom<Operator> for UnaryOp {
    type Error = Error;

    fn try_from(token: Operator) -> Result<Self, Error> {
        match token {
            Operator::Minus => Ok(UnaryOp::Minus),
            Operator::Not => Ok(UnaryOp::Not),

            _ => Err(Error::InvalidUnaryOp)
        }
    }
}

impl UnaryOp{
    pub fn apply(&self, val: i32) -> i32 {
        match self {
            UnaryOp::Minus => -val,
            UnaryOp::Not => (!(val != 0)) as i32,
        }
    }
}

pub enum BinaryOp{
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
}

impl TryFrom<Operator> for BinaryOp {
    type Error = Error;

    fn try_from(token: Operator) -> Result<Self, Error> {
        match token {
            Operator::Plus => Ok(BinaryOp::Plus),
            Operator::Minus => Ok(BinaryOp::Minus),
            Operator::Mull => Ok(BinaryOp::Mull),
            Operator::Div => Ok(BinaryOp::Div),

            Operator::Less => Ok(BinaryOp::Less),
            Operator::Greater => Ok(BinaryOp::Greater),
            Operator::Equal => Ok(BinaryOp::Equal),

            Operator::And => Ok(BinaryOp::And),
            Operator::Or => Ok(BinaryOp::Or),

            _ => Err(Error::InvalidBinOp)
        }
    }
}

impl BinaryOp {
    pub fn get_precedence(&self) -> i32 {
        match self {
            BinaryOp::Or => 0,
            BinaryOp::And => 1,

            BinaryOp::Equal => 2,
            BinaryOp::Less => 3,
            BinaryOp::Greater => 3,

            BinaryOp::Plus => 4,
            BinaryOp::Minus => 4,
            BinaryOp::Mull => 5,
            BinaryOp::Div => 5,
        }
    }

    // TODO: Should make the difference between numeric and boolean values
    pub fn apply(&self, val1: i32, val2: i32) -> i32 {
        match self {
            BinaryOp::Plus => val1 + val2,
            BinaryOp::Minus => val1 - val2,
            BinaryOp::Mull => val1 * val2,
            BinaryOp::Div => val1 / val2,

            BinaryOp::Less => (val1 < val2) as i32,
            BinaryOp::Greater => (val1 > val2) as i32,
            BinaryOp::Equal => (val1 == val2) as i32,

            BinaryOp::Or => ((val1 != 0) || (val2 != 0)) as i32,
            BinaryOp::And => ((val1 != 0) && (val2 != 0)) as i32,
        }
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
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

    fn parse_unary(&mut self) -> Result<MathExpr, Error> {
        if let Some(Token::Op(op)) = self.current() {
            if let Ok(un_op) = UnaryOp::try_from(op) {
                self.consume();
                return Ok(MathExpr::UnaryOp(un_op, Box::new(self.parse_unary()?)));
            }
        }

        match self.current() {
            None => Err(Error::ParseToken),
            _ => self.parse_primary(),
        }
    }

    // TODO: Colapse parse_binary and parse_math_expr into one(as Pratt parsing normally does)
    fn parse_binary(&mut self, left: MathExpr, op: BinaryOp) -> Result<MathExpr, Error> {
        let mut right = self.parse_unary()?;

        while let Some(Token::Op(next_op)) = self.current(){
            if let Ok(next_op) = BinaryOp::try_from(next_op){
                if next_op.get_precedence() > op.get_precedence() {
                        self.consume();
                        right = self.parse_binary(right, next_op)?;
                } else {
                    break;
                }
            }
            else{
                return Err(Error::UnexpectedToken)
            }
        }

        Ok(MathExpr::BinOp(op, Box::new(left), Box::new(right)))
    }

    fn parse_math_expr(&mut self) -> Result<MathExpr, Error> {
        let mut left = self.parse_unary()?;

        while let Some(Token::Op(op)) = self.current(){
            if let Ok(bin_op) = BinaryOp::try_from(op){
                self.consume();
                left = self.parse_binary(left, bin_op)?;
            }
            else{
                return Err(Error::UnexpectedToken)
            }
        }
        Ok(left)
    }

    fn parse_attrib(&mut self) -> Result<Expr, Error> {
        let left = self.consume().ok_or(Error::UnexpectedToken)?;
        match left {
            Token::Var(var) => {
                let next = self.consume().ok_or(Error::NotAssign)?;
                if next != Token::Op(Operator::Assign) {
                    return Err(Error::NotAssign);
                }
                Ok(Expr::Assign(var, self.parse_math_expr()?))
            }
            _ => Err(Error::NotAssign),
        }
    }

    pub fn parse_expr(&mut self) -> Result<Expr, Error> {
        match self.parse_attrib() {
            Err(Error::NotAssign) => {
                self.reset();
                self.parse_math_expr().map(Expr::Math)
            }
            other => other,
        }
    }
}