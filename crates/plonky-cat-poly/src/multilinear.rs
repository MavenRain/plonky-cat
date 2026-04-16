use plonky_cat_field::Field;
use crate::error::Error;

/// Multilinear polynomial in evaluation form over the boolean hypercube.
/// For k variables, stores 2^k evaluations: evals[i] = P(b_0, ..., b_{k-1})
/// where i = b_0 + 2*b_1 + ... + 2^{k-1}*b_{k-1}.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultilinearPoly<F> {
    num_vars: usize,
    evals: Vec<F>,
}

impl<F: Field> MultilinearPoly<F> {
    pub fn from_evals(num_vars: usize, evals: Vec<F>) -> Result<Self, Error> {
        let expected = 1_usize << num_vars;
        if evals.len() != expected {
            Err(Error::EvaluationDimensionMismatch {
                expected,
                got: evals.len(),
            })
        } else {
            Ok(Self { num_vars, evals })
        }
    }

    #[must_use]
    pub fn num_vars(&self) -> usize {
        self.num_vars
    }

    #[must_use]
    pub fn num_evals(&self) -> usize {
        self.evals.len()
    }

    #[must_use]
    pub fn evals(&self) -> &[F] {
        &self.evals
    }

    #[must_use]
    pub fn into_evals(self) -> Vec<F> {
        self.evals
    }

    /// Evaluate at a point in F^k by successive linear interpolation.
    pub fn evaluate(&self, point: &[F]) -> Result<F, Error> {
        if point.len() != self.num_vars {
            Err(Error::EvaluationDimensionMismatch {
                expected: self.num_vars,
                got: point.len(),
            })
        } else {
            Ok(point.iter().fold(self.evals.clone(), |table, r| {
                let half = table.len() / 2;
                (0..half)
                    .map(|i| table[2 * i] + (table[2 * i + 1] - table[2 * i]) * *r)
                    .collect()
            })
            .into_iter()
            .next()
            .unwrap_or_else(F::zero))
        }
    }

    /// Fix the first variable to `val`, reducing from k to k-1 variables.
    /// This is the core operation of the sumcheck protocol.
    #[must_use]
    pub fn fix_variable(self, val: F) -> Self {
        let half = self.evals.len() / 2;
        let evals = (0..half)
            .map(|i| self.evals[2 * i] + (self.evals[2 * i + 1] - self.evals[2 * i]) * val)
            .collect();
        Self {
            num_vars: self.num_vars.saturating_sub(1),
            evals,
        }
    }

    /// Sum over the boolean hypercube: Sigma_{x in {0,1}^k} P(x).
    #[must_use]
    pub fn sum_over_hypercube(&self) -> F {
        self.evals.iter().fold(F::zero(), |acc, e| acc + *e)
    }

    /// Compute the round polynomial for sumcheck: s_j(X) = Sigma_{x_{j+1},...,x_k in {0,1}^{k-j}}
    /// P(r_1,...,r_{j-1}, X, x_{j+1},...,x_k).
    /// Returns a univariate of degree 1 (since P is multilinear).
    #[must_use]
    pub fn sumcheck_round_poly(&self) -> (F, F) {
        let half = self.evals.len() / 2;
        let (sum_lo, sum_hi) = (0..half).fold(
            (F::zero(), F::zero()),
            |(acc_lo, acc_hi), i| (acc_lo + self.evals[2 * i], acc_hi + self.evals[2 * i + 1]),
        );
        (sum_lo, sum_hi)
    }
}

impl<F: Field> std::ops::Add for MultilinearPoly<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let evals = self.evals.into_iter()
            .zip(rhs.evals)
            .map(|(a, b)| a + b)
            .collect();
        Self {
            num_vars: self.num_vars,
            evals,
        }
    }
}

impl<F: Field> std::ops::Neg for MultilinearPoly<F> {
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            num_vars: self.num_vars,
            evals: self.evals.into_iter().map(std::ops::Neg::neg).collect(),
        }
    }
}

impl<F: Field + std::fmt::Display> std::fmt::Display for MultilinearPoly<F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultilinearPoly(vars={}, evals={})", self.num_vars, self.evals.len())
    }
}
