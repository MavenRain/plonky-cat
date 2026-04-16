#[derive(Debug)]
pub enum Error {
    EmptyTrace,
    RowCountNotPowerOfTwo { len: usize },
    ColumnLengthMismatch { expected: usize, got: usize },
    InsufficientWires { gate_needs: usize, trace_has: usize },
    PolyError(plonky_cat_poly::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyTrace => write!(f, "trace has no rows"),
            Self::RowCountNotPowerOfTwo { len } =>
                write!(f, "row count {len} is not a power of two"),
            Self::ColumnLengthMismatch { expected, got } =>
                write!(f, "column length {got} does not match expected {expected}"),
            Self::InsufficientWires { gate_needs, trace_has } =>
                write!(f, "gate needs {gate_needs} wires but trace has {trace_has}"),
            Self::PolyError(e) => write!(f, "poly: {e}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PolyError(e) => Some(e),
            Self::EmptyTrace => None,
            Self::RowCountNotPowerOfTwo { .. } => None,
            Self::ColumnLengthMismatch { .. } => None,
            Self::InsufficientWires { .. } => None,
        }
    }
}

impl From<plonky_cat_poly::Error> for Error {
    fn from(e: plonky_cat_poly::Error) -> Self {
        Self::PolyError(e)
    }
}
