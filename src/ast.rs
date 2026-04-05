use std::collections::HashMap;
use std::fmt;

use crate::error::Error;
use crate::lexer::MathOperator;

pub enum Expr {
    Math(MathExpr),
    Attrib(String, MathExpr),
}

impl Expr {
    pub fn eval(self, pgm_state: &mut HashMap<String, i32>) -> Result<i32, Error> {
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

pub enum MathExpr {
    Val(i32),
    Var(String),
    BinOp(MathOperator, Box<MathExpr>, Box<MathExpr>),
    UnMinus(Box<MathExpr>),
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
    pub fn eval(&self, pgm_state: &HashMap<String, i32>) -> Result<i32, Error> {
        match self {
            MathExpr::Val(val) => Ok(*val),
            MathExpr::Var(var) => pgm_state.get(var).copied().ok_or(Error::UndeclaredVar),
            MathExpr::BinOp(op, e1, e2) => {
                let v1 = e1.eval(pgm_state)?;
                let v2 = e2.eval(pgm_state)?;
                Ok(op.apply(v1, v2))
            }
            MathExpr::UnMinus(e) => Ok(-e.eval(pgm_state)?),
        }
    }
}
