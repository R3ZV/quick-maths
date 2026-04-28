use std::collections::HashMap;
use std::fmt;

use crate::error::Error;
use crate::common::Value;
use crate::parser::{BinaryOp, UnaryOp};

pub enum Expr {
    Math(MathExpr),
    Assign(String, MathExpr),
}

impl Expr {
    pub fn eval(self, pgm_state: &mut HashMap<String, Value>) -> Result<Value, Error> {
        match self {
            Expr::Math(m_expr) => m_expr.eval(pgm_state),
            Expr::Assign(var, m_expr) => {
                let right = m_expr.eval(pgm_state)?;
                pgm_state.insert(var, right);
                Ok(right)
            }
        }
    }
}

pub enum MathExpr {
    Val(Value),
    Var(String),
    BinOp(BinaryOp, Box<MathExpr>, Box<MathExpr>),
    UnaryOp(UnaryOp, Box<MathExpr>),
}

// impl fmt::Display for MathExpr {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         match self {
//             MathExpr::Val(val) => write!(f, "{}", val),
//             MathExpr::Var(var) => write!(f, "{}", var),
//             MathExpr::BinOp(op, e1, e2) => {
//                 write!(f, "{} ({}, {})", op.into(), e1, e2)
//             }
//             MathExpr::UnaryOp(op, e) => write!(f, "{}{}", op.into(), e),
//         }
//     }
// }

impl MathExpr {
    pub fn eval(&self, pgm_state: &HashMap<String, Value>) -> Result<Value, Error> {
        match self {
            MathExpr::Val(val) => Ok(*val),
            MathExpr::Var(var) => pgm_state.get(var).copied().ok_or(Error::UndeclaredVar),
            MathExpr::BinOp(op, e1, e2) => {
                let v1 = e1.eval(pgm_state)?;
                let v2 = e2.eval(pgm_state)?;
                op.apply(v1, v2)
            }
            MathExpr::UnaryOp(op, e) => op.apply(e.eval(pgm_state)?),
        }
    }
}
