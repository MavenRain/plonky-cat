#[derive(Debug)]
pub enum Error {
    EmptyCoefficients,
    EvaluationDimensionMismatch { expected: usize, got: usize },
    IndexOutOfBounds { index: usize, len: usize },
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCoefficients => write!(f, "polynomial requires at least one coefficient"),
            Self::EvaluationDimensionMismatch { expected, got } =>
                write!(f, "multilinear evaluation expected {expected} variables, got {got}"),
            Self::IndexOutOfBounds { index, len } =>
                write!(f, "coefficient index {index} out of bounds for length {len}"),
        }
    }
}

impl std::error::Error for Error {}
