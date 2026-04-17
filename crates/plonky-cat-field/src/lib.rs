#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

mod babybear;
mod binary_tower;
mod goldilocks;
mod koalabear;
mod mersenne31;

pub use self::babybear::BabyBear;
pub use self::binary_tower::BinaryField8;
pub use self::goldilocks::Goldilocks;
pub use self::koalabear::KoalaBear;
pub use self::mersenne31::Mersenne31;

pub trait Field:
    Sized
    + Clone
    + Copy
    + PartialEq
    + Eq
    + std::fmt::Debug
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Neg<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
    fn inv(self) -> Result<Self, Error>;
    fn pow(self, exp: u64) -> Self;

    fn double(self) -> Self {
        self + self
    }

    fn square(self) -> Self {
        self * self
    }
}
