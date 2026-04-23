#[derive(Debug)]
pub enum Error {
    ParseToken,
    UnexpectedToken,
    ValOutOfBounds,
    NotAttrib,
    NotComp,
    UndeclaredVar,
    InvalidOp,
    None,
}
