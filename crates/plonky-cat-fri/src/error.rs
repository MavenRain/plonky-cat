#[derive(Debug)]
pub enum Error {
    StepOnFinished,
    CodewordLengthNotPowerOfTwo { len: usize },
    CodewordEmpty,
    MerkleError(plonky_cat_merkle::Error),
    FoldingMismatch,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StepOnFinished =>
                write!(f, "FRI step called on fully reduced codeword"),
            Self::CodewordLengthNotPowerOfTwo { len } =>
                write!(f, "codeword length {len} is not a power of two"),
            Self::CodewordEmpty =>
                write!(f, "codeword is empty"),
            Self::MerkleError(e) =>
                write!(f, "merkle: {e}"),
            Self::FoldingMismatch =>
                write!(f, "FRI folding produced inconsistent codeword"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MerkleError(e) => Some(e),
            Self::StepOnFinished => None,
            Self::CodewordLengthNotPowerOfTwo { .. } => None,
            Self::CodewordEmpty => None,
            Self::FoldingMismatch => None,
        }
    }
}

impl From<plonky_cat_merkle::Error> for Error {
    fn from(e: plonky_cat_merkle::Error) -> Self {
        Self::MerkleError(e)
    }
}
