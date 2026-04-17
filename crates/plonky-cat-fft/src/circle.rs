use plonky_cat_field::{Mersenne31, Field};
use crate::error::Error;

/// Circle group point over Mersenne31: (x, y) where x^2 + y^2 = 1.
/// The circle group C(F_p) has order p + 1 = 2^31, which is a power of two,
/// making it ideal for FFT despite Mersenne31 lacking multiplicative roots
/// of unity of high 2-adic order.
///
/// This is the foundation of circle-STARKs (Starknet/Stwo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CirclePoint {
    x: Mersenne31,
    y: Mersenne31,
}

impl CirclePoint {
    #[must_use]
    pub fn new(x: Mersenne31, y: Mersenne31) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn identity() -> Self {
        Self { x: Mersenne31::one(), y: Mersenne31::zero() }
    }

    #[must_use]
    pub fn x(&self) -> Mersenne31 { self.x }

    #[must_use]
    pub fn y(&self) -> Mersenne31 { self.y }

    #[must_use]
    pub fn is_on_circle(&self) -> bool {
        self.x.square() + self.y.square() == Mersenne31::one()
    }

    /// Circle group operation: (x1, y1) * (x2, y2) = (x1*x2 - y1*y2, x1*y2 + y1*x2).
    /// This is complex multiplication restricted to the unit circle.
    #[must_use]
    pub fn mul(self, rhs: Self) -> Self {
        Self {
            x: self.x * rhs.x - self.y * rhs.y,
            y: self.x * rhs.y + self.y * rhs.x,
        }
    }

    /// Repeated squaring on the circle group.
    #[must_use]
    pub fn pow(self, exp: u64) -> Self {
        (0..64)
            .rev()
            .fold((Self::identity(), self), |(acc, b), i| {
                let squared = acc.mul(acc);
                if (exp >> i) & 1 == 1 { (squared.mul(b), b) } else { (squared, b) }
            })
            .0
    }

    /// Generator of the circle group of order 2^31.
    /// g = (x, y) where x^2 + y^2 = 1 and g has order 2^31.
    #[must_use]
    pub fn generator() -> Self {
        Self::new(Mersenne31::from(2u32), Mersenne31::from(1268011823u32))
    }

    /// Subgroup generator of order 2^log_n.
    pub fn subgroup_generator(log_n: u32) -> Result<Self, Error> {
        if log_n > 31 {
            Err(Error::LengthNotPowerOfTwo { len: 1 << log_n.min(31) })
        } else {
            Ok(Self::generator().pow(1u64 << (31 - log_n)))
        }
    }
}

/// Circle-domain evaluation: evaluate a polynomial at circle-group points.
/// The "circle FFT" evaluates a polynomial P at the points
/// {g^0, g^1, ..., g^{n-1}} on the circle, where g is the subgroup generator.
///
/// For v0.3, this is a direct evaluation (O(n^2)); the fast circle FFT
/// (O(n log n)) uses a butterfly structure over the circle group.
pub fn circle_evaluate(
    coeffs: &[Mersenne31],
    log_n: u32,
) -> Result<Vec<Mersenne31>, Error> {
    if coeffs.is_empty() {
        Err(Error::EmptyInput)
    } else {
        let g = CirclePoint::subgroup_generator(log_n)?;
        let n = 1usize << log_n;
        let domain: Vec<CirclePoint> = std::iter::successors(
            Some(CirclePoint::identity()),
            |prev| Some(prev.mul(g)),
        )
        .take(n)
        .collect();

        Ok(domain.iter()
            .map(|pt| {
                coeffs.iter()
                    .enumerate()
                    .fold(Mersenne31::zero(), |acc, (i, c)| {
                        acc + *c * pt.pow(u64::try_from(i).unwrap_or(0)).x()
                    })
            })
            .collect())
    }
}
