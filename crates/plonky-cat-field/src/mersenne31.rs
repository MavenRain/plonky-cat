use crate::error::Error;
use crate::Field;

const P: u64 = (1 << 31) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mersenne31(u64);

impl Mersenne31 {
    #[must_use]
    pub fn modulus() -> u64 {
        P
    }
}

impl Field for Mersenne31 {
    fn zero() -> Self {
        Self(0)
    }

    fn one() -> Self {
        Self(1)
    }

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

impl From<u64> for Mersenne31 {
    fn from(val: u64) -> Self {
        Self(val % P)
    }
}

impl From<u32> for Mersenne31 {
    fn from(val: u32) -> Self {
        Self(u64::from(val) % P)
    }
}

impl From<Mersenne31> for u64 {
    fn from(val: Mersenne31) -> Self {
        val.0
    }
}

impl std::ops::Add for Mersenne31 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let sum = self.0 + rhs.0;
        Self(if sum >= P { sum - P } else { sum })
    }
}

impl std::ops::Sub for Mersenne31 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(if self.0 >= rhs.0 {
            self.0 - rhs.0
        } else {
            self.0 + P - rhs.0
        })
    }
}

impl std::ops::Mul for Mersenne31 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let product = self.0 * rhs.0;
        Self(reduce_mersenne(product))
    }
}

impl std::ops::Neg for Mersenne31 {
    type Output = Self;

    fn neg(self) -> Self {
        if self.0 == 0 { Self(0) } else { Self(P - self.0) }
    }
}

impl std::fmt::Display for Mersenne31 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn reduce_mersenne(val: u64) -> u64 {
    let lo = val & P;
    let hi = val >> 31;
    let sum = lo + hi;
    if sum >= P { sum - P } else { sum }
}

fn iterative_pow(base: Mersenne31, exp: u64) -> Mersenne31 {
    (0..64)
        .rev()
        .fold((Mersenne31::one(), base), |(acc, b), i| {
            let squared = acc * acc;
            if (exp >> i) & 1 == 1 {
                (squared * b, b)
            } else {
                (squared, b)
            }
        })
        .0
}
