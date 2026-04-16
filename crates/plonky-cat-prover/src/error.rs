#[derive(Debug)]
pub enum Error<RE: std::fmt::Debug> {
    Reduction(RE),
    MaxRoundsExceeded { limit: usize },
}

impl<RE: std::fmt::Debug + std::fmt::Display> std::fmt::Display for Error<RE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reduction(e) => write!(f, "reduction: {e}"),
            Self::MaxRoundsExceeded { limit } =>
                write!(f, "protocol exceeded {limit} rounds without terminating"),
        }
    }
}

impl<RE: std::fmt::Debug + std::fmt::Display> std::error::Error for Error<RE> {}
