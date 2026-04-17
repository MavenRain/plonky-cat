use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;
use crate::error::Error;

/// Customizable Constraint System: generalizes R1CS, PLONK, and AIR.
///
/// A CCS instance is defined by:
///   - `t` constraint matrices M_0, ..., M_{t-1}, each n x m
///   - `q` multisets S_0, ..., S_{q-1} of indices into the matrices
///   - `q` coefficients c_0, ..., c_{q-1}
///
/// The constraint is: sum_{i=0}^{q-1} c_i * hadamard_product_{j in S_i} (M_j * z) = 0
///
/// where z is the witness vector and hadamard_product is element-wise multiplication.
///
/// R1CS is CCS with t=3, q=2, S_0={0,1}, S_1={2}, c_0=1, c_1=-1:
///   M_0 * z . M_1 * z - M_2 * z = 0  (i.e., A*z . B*z = C*z)
///
/// PLONK is CCS with selector columns as sparse matrices.

#[derive(Debug, Clone)]
pub struct CcsInstance<F> {
    num_constraints: usize,
    num_variables: usize,
    matrices: Vec<SparseMatrix<F>>,
    multisets: Vec<Vec<usize>>,
    coefficients: Vec<F>,
}

#[derive(Debug, Clone)]
pub struct SparseMatrix<F> {
    num_rows: usize,
    num_cols: usize,
    entries: Vec<(usize, usize, F)>,
}

impl<F: Field> SparseMatrix<F> {
    #[must_use]
    pub fn new(num_rows: usize, num_cols: usize, entries: Vec<(usize, usize, F)>) -> Self {
        Self { num_rows, num_cols, entries }
    }

    #[must_use]
    pub fn num_rows(&self) -> usize { self.num_rows }

    #[must_use]
    pub fn num_cols(&self) -> usize { self.num_cols }

    #[must_use]
    pub fn mul_vec(&self, z: &[F]) -> Vec<F> {
        let init = vec![F::zero(); self.num_rows];
        self.entries.iter().fold(init, |acc, (row, col, val)| {
            let updated = acc[*row] + *val * z[*col];
            acc.into_iter()
                .enumerate()
                .map(|(i, v)| if i == *row { updated } else { v })
                .collect()
        })
    }
}

impl<F: Field> CcsInstance<F> {
    pub fn new(
        num_constraints: usize,
        num_variables: usize,
        matrices: Vec<SparseMatrix<F>>,
        multisets: Vec<Vec<usize>>,
        coefficients: Vec<F>,
    ) -> Result<Self, Error> {
        if multisets.len() != coefficients.len() {
            Err(Error::ColumnLengthMismatch {
                expected: multisets.len(),
                got: coefficients.len(),
            })
        } else {
            Ok(Self { num_constraints, num_variables, matrices, multisets, coefficients })
        }
    }

    #[must_use]
    pub fn num_constraints(&self) -> usize { self.num_constraints }

    #[must_use]
    pub fn num_variables(&self) -> usize { self.num_variables }

    /// Check whether witness z satisfies the CCS constraints.
    #[must_use]
    pub fn is_satisfied(&self, z: &[F]) -> bool {
        let mz: Vec<Vec<F>> = self.matrices.iter()
            .map(|m| m.mul_vec(z))
            .collect();

        let result = self.multisets.iter()
            .zip(self.coefficients.iter())
            .fold(vec![F::zero(); self.num_constraints], |acc, (multiset, coeff)| {
                let hadamard = multiset.iter().fold(
                    vec![F::one(); self.num_constraints],
                    |prod, &idx| {
                        prod.into_iter()
                            .zip(mz[idx].iter())
                            .map(|(p, v)| p * *v)
                            .collect()
                    },
                );
                acc.into_iter()
                    .zip(hadamard.iter())
                    .map(|(a, h)| a + *coeff * *h)
                    .collect()
            });

        result.iter().all(|v| *v == F::zero())
    }

    /// Convert the CCS constraint evaluation to a multilinear polynomial
    /// for sumcheck.  Requires num_constraints to be a power of two.
    pub fn constraint_poly(&self, z: &[F]) -> Result<MultilinearPoly<F>, Error> {
        let mz: Vec<Vec<F>> = self.matrices.iter()
            .map(|m| m.mul_vec(z))
            .collect();

        let evals: Vec<F> = (0..self.num_constraints)
            .map(|row| {
                self.multisets.iter()
                    .zip(self.coefficients.iter())
                    .fold(F::zero(), |acc, (multiset, coeff)| {
                        let hadamard = multiset.iter()
                            .fold(F::one(), |prod, &idx| prod * mz[idx][row]);
                        acc + *coeff * hadamard
                    })
            })
            .collect();

        let log_n = self.num_constraints.trailing_zeros();
        MultilinearPoly::from_evals(
            log_n.try_into().map_err(|_| Error::EmptyTrace)?,
            evals,
        ).map_err(Error::from)
    }
}
