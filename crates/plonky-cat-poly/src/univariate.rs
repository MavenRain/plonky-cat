use plonky_cat_field::Field;
use crate::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnivariatePoly<F> {
    coeffs: Vec<F>,
}

impl<F: Field> UnivariatePoly<F> {
    pub fn from_coeffs(coeffs: Vec<F>) -> Result<Self, Error> {
        if coeffs.is_empty() {
            Err(Error::EmptyCoefficients)
        } else {
            Ok(Self { coeffs })
        }
    }

    #[must_use]
    pub fn zero_poly() -> Self {
        Self { coeffs: vec![F::zero()] }
    }

    #[must_use]
    pub fn constant(val: F) -> Self {
        Self { coeffs: vec![val] }
    }

    #[must_use]
    pub fn degree(&self) -> usize {
        self.coeffs.iter()
            .rposition(|c| *c != F::zero())
            .unwrap_or(0)
    }

    #[must_use]
    pub fn num_coeffs(&self) -> usize {
        self.coeffs.len()
    }

    pub fn coeff(&self, index: usize) -> Result<F, Error> {
        self.coeffs.get(index)
            .copied()
            .ok_or(Error::IndexOutOfBounds { index, len: self.coeffs.len() })
    }

    #[must_use]
    pub fn evaluate(&self, point: F) -> F {
        self.coeffs.iter()
            .rev()
            .fold(F::zero(), |acc, c| acc * point + *c)
    }

    #[must_use]
    pub fn into_coeffs(self) -> Vec<F> {
        self.coeffs
    }
}

impl<F: Field> std::ops::Add for UnivariatePoly<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let max_len = self.coeffs.len().max(rhs.coeffs.len());
        let coeffs = (0..max_len)
            .map(|i| {
                let a = self.coeffs.get(i).copied().unwrap_or_else(F::zero);
                let b = rhs.coeffs.get(i).copied().unwrap_or_else(F::zero);
                a + b
            })
            .collect();
        Self { coeffs }
    }
}

impl<F: Field> std::ops::Sub for UnivariatePoly<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let max_len = self.coeffs.len().max(rhs.coeffs.len());
        let coeffs = (0..max_len)
            .map(|i| {
                let a = self.coeffs.get(i).copied().unwrap_or_else(F::zero);
                let b = rhs.coeffs.get(i).copied().unwrap_or_else(F::zero);
                a - b
            })
            .collect();
        Self { coeffs }
    }
}

impl<F: Field> std::ops::Neg for UnivariatePoly<F> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            coeffs: self.coeffs.into_iter().map(std::ops::Neg::neg).collect(),
        }
    }
}

impl<F: Field> std::ops::Mul for UnivariatePoly<F> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        if self.coeffs.is_empty() || rhs.coeffs.is_empty() {
            Self::zero_poly()
        } else {
        let result_len = self.coeffs.len() + rhs.coeffs.len() - 1;
        let coeffs = (0..result_len)
            .map(|k| {
                let lo = k.saturating_sub(rhs.coeffs.len() - 1);
                let hi = k.min(self.coeffs.len() - 1);
                (lo..=hi).fold(F::zero(), |acc, i| {
                    acc + self.coeffs[i] * rhs.coeffs[k - i]
                })
            })
            .collect();
        Self { coeffs }
        }
    }
}

impl<F: Field + std::fmt::Display> std::fmt::Display for UnivariatePoly<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "UnivariatePoly(deg={})", self.degree())
    }
}
