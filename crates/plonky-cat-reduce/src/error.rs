#[derive(Debug)]
pub enum Error {
    StepOnFinishedClaim,
    ChallengeUnused,
    MessageConsistencyFailure,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StepOnFinishedClaim => write!(f, "step called on finished claim"),
            Self::ChallengeUnused => write!(f, "challenge was unused"),
            Self::MessageConsistencyFailure => write!(f, "message consistency failure"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum SeqError<EA, EB, EAd> {
    InA(EA),
    InB(EB),
    Handoff(EAd),
    PhaseDesync,
}

impl<EA, EB, EAd> std::fmt::Display for SeqError<EA, EB, EAd>
where
    EA: std::fmt::Display,
    EB: std::fmt::Display,
    EAd: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InA(e) => write!(f, "phase A: {e}"),
            Self::InB(e) => write!(f, "phase B: {e}"),
            Self::Handoff(e) => write!(f, "handoff: {e}"),
            Self::PhaseDesync => write!(f, "phase desynchronization"),
        }
    }
}

impl<EA, EB, EAd> std::error::Error for SeqError<EA, EB, EAd>
where
    EA: std::fmt::Debug + std::fmt::Display,
    EB: std::fmt::Debug + std::fmt::Display,
    EAd: std::fmt::Debug + std::fmt::Display,
{
}

#[derive(Debug)]
pub enum InterleaveError<EA, EB, EAd> {
    InA(EA),
    InB(EB),
    Adapter(EAd),
    DoneDesync,
}

impl<EA, EB, EAd> std::fmt::Display for InterleaveError<EA, EB, EAd>
where
    EA: std::fmt::Display,
    EB: std::fmt::Display,
    EAd: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InA(e) => write!(f, "functor A: {e}"),
            Self::InB(e) => write!(f, "functor B: {e}"),
            Self::Adapter(e) => write!(f, "adapter: {e}"),
            Self::DoneDesync => write!(f, "done desynchronization"),
        }
    }
}

impl<EA, EB, EAd> std::error::Error for InterleaveError<EA, EB, EAd>
where
    EA: std::fmt::Debug + std::fmt::Display,
    EB: std::fmt::Debug + std::fmt::Display,
    EAd: std::fmt::Debug + std::fmt::Display,
{
}
