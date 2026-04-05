use crate::ast::{Expr, MathExpr};
use crate::error::Error;
use crate::lexer::{MathOperator, Operator, Parenthesis, Token};

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

    fn parse_factor(&mut self) -> Result<MathExpr, Error> {
        match self.current() {
            Some(Token::Op(Operator::Math(MathOperator::Minus))) => {
                self.consume();
                Ok(MathExpr::UnMinus(Box::new(self.parse_primary()?)))
            }
            None => Err(Error::ParseToken),
            _ => self.parse_primary(),
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

    pub fn parse_expr(&mut self) -> Result<Expr, Error> {
        match self.parse_attrib() {
            Err(Error::NotAttrib) => {
                self.reset();
                self.parse_math_expr().map(Expr::Math)
            }
            other => other,
        }
    }
}
