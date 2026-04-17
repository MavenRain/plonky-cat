use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;
use crate::error::Error;
use crate::{SumcheckClaim, SumcheckFunction};

/// LogUp: a lookup argument via sumcheck.
///
/// Proves that every element in `values` appears in `table` by reducing
/// to the identity:
///   sum_{i} 1/(beta - values[i]) = sum_{j} m_j/(beta - table[j])
///
/// where m_j is the multiplicity of table[j] in values, and beta is a
/// random challenge from the verifier.
///
/// The sumcheck verifies this identity by checking that the "difference
/// polynomial" sums to zero over the boolean hypercube:
///   sum_{x in {0,1}^k} [f(x) - g(x)] = 0
///
/// where f encodes the LHS and g encodes the RHS.

#[derive(Debug, Clone)]
pub struct LogUpWitness<F: Field> {
    numerators: MultilinearPoly<F>,
    denominators: MultilinearPoly<F>,
}

impl<F: Field> LogUpWitness<F> {
    pub fn new(
        numerators: MultilinearPoly<F>,
        denominators: MultilinearPoly<F>,
    ) -> Self {
        Self { numerators, denominators }
    }
}

impl<F: Field> SumcheckFunction for LogUpWitness<F> {
    type F = F;

    fn num_vars(&self) -> usize { self.numerators.num_vars() }

    fn round_poly_degree(&self) -> usize { 2 }

    fn round_poly_evals(&self) -> Vec<F> {
        let half = self.numerators.num_evals() / 2;
        let n_evals = self.numerators.evals();
        let d_evals = self.denominators.evals();

        (0..3)
            .map(|t| {
                (0..half).fold(F::zero(), |acc, i| {
                    let n_val = eval_at(n_evals[2 * i], n_evals[2 * i + 1], t);
                    let d_val = eval_at(d_evals[2 * i], d_evals[2 * i + 1], t);
                    acc + n_val * d_val
                })
            })
            .collect()
    }

    fn fix_variable(self, val: F) -> Self {
        Self {
            numerators: self.numerators.fix_variable(val),
            denominators: self.denominators.fix_variable(val),
        }
    }

    fn final_value(&self) -> Option<F> {
        self.numerators.evals().first()
            .zip(self.denominators.evals().first())
            .map(|(n, d)| *n * *d)
    }
}

/// Build a LogUp sumcheck claim from values and table.
/// `beta` is the random challenge binding the lookup argument.
pub fn logup_claim<F: Field>(
    values: &[F],
    table: &[F],
    multiplicities: &[F],
    beta: F,
) -> Result<(SumcheckClaim<F>, LogUpWitness<F>), Error> {
    let k = values.len();
    if !k.is_power_of_two() {
        Err(Error::WitnessEmpty)
    } else {
        let log_k = k.trailing_zeros();

        let lhs_nums: Vec<F> = values.iter().map(|_| F::one()).collect();
        let lhs_denoms: Vec<F> = values.iter().map(|v| beta - *v).collect();

        let rhs_nums: Vec<F> = multiplicities.to_vec();
        let rhs_denoms: Vec<F> = table.iter().map(|t| beta - *t).collect();

        let diff_nums: Vec<F> = lhs_nums.into_iter()
            .zip(rhs_nums)
            .map(|(l, r)| l - r)
            .collect();
        let diff_denoms: Vec<F> = lhs_denoms.into_iter()
            .zip(rhs_denoms)
            .map(|(l, r)| l * r)
            .collect();

        let claimed_sum = diff_nums.iter()
            .zip(diff_denoms.iter())
            .fold(F::zero(), |acc, (n, d)| acc + *n * *d);

        let num_vars: usize = log_k.try_into().map_err(|_| Error::WitnessEmpty)?;
        let num_poly = MultilinearPoly::from_evals(num_vars, diff_nums)
            .map_err(|_| Error::WitnessEmpty)?;
        let den_poly = MultilinearPoly::from_evals(num_vars, diff_denoms)
            .map_err(|_| Error::WitnessEmpty)?;

        Ok((
            SumcheckClaim::new(claimed_sum, num_vars),
            LogUpWitness::new(num_poly, den_poly),
        ))
    }
}

fn eval_at<F: Field>(e0: F, e1: F, t: usize) -> F {
    (0..t).fold(e0, |_acc, _| e1 + (e1 - e0) * field_from_usize::<F>(t))
}

fn field_from_usize<F: Field>(n: usize) -> F {
    (0..n).fold(F::zero(), |acc, _| acc + F::one())
}
