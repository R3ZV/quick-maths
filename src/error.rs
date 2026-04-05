#[derive(Debug)]
pub enum Error {
    ParseToken,
    UnexpectedToken,
    ValOutOfBounds,
    NotAttrib,
    UndeclaredVar,
    None,
}
