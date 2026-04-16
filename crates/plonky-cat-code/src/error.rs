#[derive(Debug)]
pub enum Error {
    MessageTooLong { msg_len: usize, rate_times_n: usize },
    FftError(plonky_cat_fft::Error),
    InvalidRate,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MessageTooLong { msg_len, rate_times_n } =>
                write!(f, "message length {msg_len} exceeds code dimension {rate_times_n}"),
            Self::FftError(e) =>
                write!(f, "FFT: {e}"),
            Self::InvalidRate =>
                write!(f, "rate must be a power-of-two reciprocal"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::FftError(e) => Some(e),
            Self::MessageTooLong { .. } => None,
            Self::InvalidRate => None,
        }
    }
}

impl From<plonky_cat_fft::Error> for Error {
    fn from(e: plonky_cat_fft::Error) -> Self {
        Self::FftError(e)
    }
}
