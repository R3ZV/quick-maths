#[derive(Debug)]
pub enum Error {
    ParseToken,
    UnexpectedToken,
    ValOutOfBounds,
    NotAssign,
    UndeclaredVar,
    InvalidBinOp,
    InvalidUnaryOp,
    None,
}
