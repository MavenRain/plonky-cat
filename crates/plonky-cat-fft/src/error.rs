#[derive(Debug)]
pub enum Error {
    LengthNotPowerOfTwo { len: usize },
    EmptyInput,
    InverseOfZero,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LengthNotPowerOfTwo { len } =>
                write!(f, "FFT input length {len} is not a power of two"),
            Self::EmptyInput =>
                write!(f, "FFT input is empty"),
            Self::InverseOfZero =>
                write!(f, "cannot invert zero for inverse FFT scaling"),
        }
    }
}

impl std::error::Error for Error {}
