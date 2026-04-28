use std::fmt;
#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Value {
    Int(i32),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(val) => write!(f, "int::{}", val),
            Value::Bool(val) => write!(f, "bool::{}", val),
        }
    }
}