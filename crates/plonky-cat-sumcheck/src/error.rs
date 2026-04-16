#[derive(Debug)]
pub enum Error {
    RoundSumMismatch,
    StepOnFinished,
    WitnessEmpty,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RoundSumMismatch =>
                write!(f, "round polynomial s(0) + s(1) does not match claimed sum"),
            Self::StepOnFinished =>
                write!(f, "prover_step called on fully reduced claim"),
            Self::WitnessEmpty =>
                write!(f, "witness polynomial has no evaluations"),
        }
    }
}

impl std::error::Error for Error {}
