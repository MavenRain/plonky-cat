use crate::error::Error;
use crate::Field;

/// GF(2^8) with irreducible polynomial x^8 + x^4 + x^3 + x + 1 (0x11B).
/// The AES field; used here as the base case for binary tower construction.
/// Binius builds on GF(2^{2^k}) towers; this demonstrates the arithmetic.
///
/// Addition/subtraction: XOR.  Multiplication: polynomial product mod 0x11B.
/// All elements fit in a u8.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BinaryField8(u16);

const IRREDUCIBLE: u16 = 0x11B;

impl BinaryField8 {
    #[must_use]
    pub fn from_byte(val: u8) -> Self {
        Self(u16::from(val))
    }

    #[must_use]
    pub fn to_byte(self) -> u8 {
        u8::try_from(self.0 & 0xFF).unwrap_or(0)
    }
}

impl Field for BinaryField8 {
    fn zero() -> Self { Self(0) }
    fn one() -> Self { Self(1) }

    fn inv(self) -> Result<Self, Error> {
        if self.0 == 0 {
            Err(Error::DivisionByZero)
        } else {
            Ok(self.pow(254))
        }
    }

    fn pow(self, exp: u64) -> Self {
        (0..64)
            .rev()
            .fold((Self::one(), self), |(acc, b), i| {
                let squared = acc * acc;
                if (exp >> i) & 1 == 1 { (squared * b, b) } else { (squared, b) }
            })
            .0
    }
}

impl From<u64> for BinaryField8 {
    fn from(val: u64) -> Self { Self(u16::try_from(val & 0xFF).unwrap_or(0)) }
}

impl From<u32> for BinaryField8 {
    fn from(val: u32) -> Self { Self(u16::try_from(val & 0xFF).unwrap_or(0)) }
}

impl std::ops::Add for BinaryField8 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self { Self(self.0 ^ rhs.0) }
}

impl std::ops::Sub for BinaryField8 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self { Self(self.0 ^ rhs.0) }
}

impl std::ops::Mul for BinaryField8 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self(gf8_mul(self.0, rhs.0))
    }
}

impl std::ops::Neg for BinaryField8 {
    type Output = Self;
    fn neg(self) -> Self { self }
}

impl std::fmt::Display for BinaryField8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:02x}", self.0)
    }
}

fn gf8_mul(a: u16, b: u16) -> u16 {
    let product = carryless_mul_u8(a, b);
    reduce(product)
}

fn carryless_mul_u8(a: u16, b: u16) -> u16 {
    (0..8).fold(0u16, |acc, i| {
        if (b >> i) & 1 == 1 {
            acc ^ (a << i)
        } else {
            acc
        }
    })
}

fn reduce(val: u16) -> u16 {
    (8..16).rev().fold(val, |v, i| {
        if (v >> i) & 1 == 1 {
            v ^ (IRREDUCIBLE << (i - 8))
        } else {
            v
        }
    })
}
