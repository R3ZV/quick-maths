#[derive(Debug)]
pub enum Error {
    Overflow,
    DivisionByZero,
    ParseToken,
    UnexpectedToken,
    ValOutOfBounds,
    NotAssign,
    UndeclaredVar,
    InvalidBinOp,
    InvalidUnaryOp,
    TypeMismatch,
}
