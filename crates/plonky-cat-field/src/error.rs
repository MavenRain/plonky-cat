#[derive(Debug)]
pub enum Error {
    DivisionByZero,
    InvalidFieldElement,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DivisionByZero => write!(f, "division by zero in finite field"),
            Self::InvalidFieldElement => write!(f, "value exceeds field modulus"),
        }
    }
}

impl std::error::Error for Error {}
