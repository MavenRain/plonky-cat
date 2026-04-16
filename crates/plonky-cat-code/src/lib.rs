#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_field::Field;
use plonky_cat_fft::ntt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RsParams {
    log_n: u32,
    log_rate_inv: u32,
}

impl RsParams {
    pub fn new(log_n: u32, log_rate_inv: u32) -> Result<Self, Error> {
        if log_rate_inv == 0 {
            Err(Error::InvalidRate)
        } else {
            Ok(Self { log_n, log_rate_inv })
        }
    }

    #[must_use]
    pub fn codeword_len(&self) -> usize {
        1 << self.log_n
    }

    #[must_use]
    pub fn message_len(&self) -> usize {
        1 << (self.log_n - self.log_rate_inv)
    }

    #[must_use]
    pub fn log_n(&self) -> u32 {
        self.log_n
    }

    #[must_use]
    pub fn log_rate_inv(&self) -> u32 {
        self.log_rate_inv
    }
}

pub fn rs_encode<F: Field>(
    message: &[F],
    omega: F,
    params: RsParams,
) -> Result<Vec<F>, Error> {
    let n = params.codeword_len();
    let k = params.message_len();

    if message.len() > k {
        Err(Error::MessageTooLong {
            msg_len: message.len(),
            rate_times_n: k,
        })
    } else {
        let padded: Vec<F> = message.iter()
            .copied()
            .chain(std::iter::repeat(F::zero()).take(n - message.len()))
            .collect();

        ntt(&padded, omega).map_err(Error::from)
    }
}
