#[derive(Debug)]
pub enum Error {
    FriError(plonky_cat_fri::Error),
    SumcheckError(plonky_cat_sumcheck::Error),
    LensMismatch,
    InconsistentShared,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FriError(e) => write!(f, "FRI: {e}"),
            Self::SumcheckError(e) => write!(f, "sumcheck: {e}"),
            Self::LensMismatch => write!(f, "lens split/join inconsistency"),
            Self::InconsistentShared =>
                write!(f, "shared state inconsistent between FRI and sumcheck"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FriError(e) => Some(e),
            Self::SumcheckError(e) => Some(e),
            Self::LensMismatch => None,
            Self::InconsistentShared => None,
        }
    }
}

impl From<plonky_cat_fri::Error> for Error {
    fn from(e: plonky_cat_fri::Error) -> Self {
        Self::FriError(e)
    }
}

impl From<plonky_cat_sumcheck::Error> for Error {
    fn from(e: plonky_cat_sumcheck::Error) -> Self {
        Self::SumcheckError(e)
    }
}
