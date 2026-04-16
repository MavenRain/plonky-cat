#[derive(Debug)]
pub enum Error<RE: std::fmt::Debug> {
    Reduction(RE),
    UnexpectedDone { round: usize },
    ProtocolNotDone { messages_consumed: usize },
}

impl<RE: std::fmt::Debug + std::fmt::Display> std::fmt::Display for Error<RE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reduction(e) => write!(f, "reduction: {e}"),
            Self::UnexpectedDone { round } =>
                write!(f, "verifier returned Done on round {round} with messages remaining"),
            Self::ProtocolNotDone { messages_consumed } =>
                write!(f, "all {messages_consumed} messages consumed but verifier did not return Done"),
        }
    }
}

impl<RE: std::fmt::Debug + std::fmt::Display> std::error::Error for Error<RE> {}
