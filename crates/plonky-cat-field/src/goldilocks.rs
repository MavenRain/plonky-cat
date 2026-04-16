use crate::error::Error;
use crate::Field;

const P: u128 = 18_446_744_069_414_584_321;
const ORDER_MINUS_2: u64 = 18_446_744_069_414_584_319;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Goldilocks(u128);

impl Goldilocks {
    #[must_use]
    pub fn modulus() -> u128 {
        P
    }
}

impl Field for Goldilocks {
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
            Ok(self.pow(ORDER_MINUS_2))
        }
    }

    fn pow(self, exp: u64) -> Self {
        iterative_pow(self, exp)
    }
}

impl From<u64> for Goldilocks {
    fn from(val: u64) -> Self {
        Self(u128::from(val) % P)
    }
}

impl From<u32> for Goldilocks {
    fn from(val: u32) -> Self {
        Self(u128::from(val) % P)
    }
}

impl From<Goldilocks> for u128 {
    fn from(val: Goldilocks) -> Self {
        val.0
    }
}

impl std::ops::Add for Goldilocks {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let sum = self.0 + rhs.0;
        Self(if sum >= P { sum - P } else { sum })
    }
}

impl std::ops::Sub for Goldilocks {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(if self.0 >= rhs.0 {
            self.0 - rhs.0
        } else {
            self.0 + P - rhs.0
        })
    }
}

impl std::ops::Mul for Goldilocks {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self((self.0 * rhs.0) % P)
    }
}

impl std::ops::Neg for Goldilocks {
    type Output = Self;

    fn neg(self) -> Self {
        if self.0 == 0 { Self(0) } else { Self(P - self.0) }
    }
}

impl std::fmt::Display for Goldilocks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn iterative_pow(base: Goldilocks, exp: u64) -> Goldilocks {
    (0..64)
        .rev()
        .fold((Goldilocks::one(), base), |(acc, b), i| {
            let squared = acc * acc;
            if (exp >> i) & 1 == 1 {
                (squared * b, b)
            } else {
                (squared, b)
            }
        })
        .0
}
