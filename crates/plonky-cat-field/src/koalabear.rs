use crate::error::Error;
use crate::Field;

const P: u64 = 0x7F00_0001;
const TWO_ADIC_ORDER: u32 = 24;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KoalaBear(u64);

impl KoalaBear {
    #[must_use]
    pub fn modulus() -> u64 {
        P
    }

    #[must_use]
    pub fn two_adic_order() -> u32 {
        TWO_ADIC_ORDER
    }

    #[must_use]
    pub fn two_adic_generator() -> Self {
        Self::from(3u32).pow((P - 1) / (1u64 << TWO_ADIC_ORDER))
    }

    pub fn root_of_unity(log_n: u32) -> Result<Self, Error> {
        if log_n > TWO_ADIC_ORDER {
            Err(Error::InvalidFieldElement)
        } else {
            Ok(Self::two_adic_generator().pow(1u64 << (TWO_ADIC_ORDER - log_n)))
        }
    }
}

impl Field for KoalaBear {
    fn zero() -> Self { Self(0) }
    fn one() -> Self { Self(1) }

    fn inv(self) -> Result<Self, Error> {
        if self.0 == 0 {
            Err(Error::DivisionByZero)
        } else {
            Ok(self.pow(P - 2))
        }
    }

    fn pow(self, exp: u64) -> Self {
        iterative_pow(self, exp)
    }
}

impl From<u64> for KoalaBear {
    fn from(val: u64) -> Self { Self(val % P) }
}

impl From<u32> for KoalaBear {
    fn from(val: u32) -> Self { Self(u64::from(val) % P) }
}

impl From<KoalaBear> for u64 {
    fn from(val: KoalaBear) -> Self { val.0 }
}

impl std::ops::Add for KoalaBear {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self((self.0 + rhs.0) % P) }
}

impl std::ops::Sub for KoalaBear {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self((self.0 + P - rhs.0) % P) }
}

impl std::ops::Mul for KoalaBear {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self { Self((self.0 * rhs.0) % P) }
}

impl std::ops::Neg for KoalaBear {
    type Output = Self;
    fn neg(self) -> Self { if self.0 == 0 { Self(0) } else { Self(P - self.0) } }
}

impl std::fmt::Display for KoalaBear {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn iterative_pow(base: KoalaBear, exp: u64) -> KoalaBear {
    (0..64)
        .rev()
        .fold((KoalaBear::one(), base), |(acc, b), i| {
            let squared = acc * acc;
            if (exp >> i) & 1 == 1 { (squared * b, b) } else { (squared, b) }
        })
        .0
}
