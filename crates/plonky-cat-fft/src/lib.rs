#![forbid(unsafe_code)]

pub mod circle;
mod error;

pub use self::circle::CirclePoint;
pub use self::error::Error;

use plonky_cat_field::Field;

/// Radix-2 DIT NTT.  Evaluates a polynomial (given as coefficients) at all
/// powers of `omega`: [P(omega^0), P(omega^1), ..., P(omega^{n-1})].
///
/// `omega` must be a principal n-th root of unity, and `coeffs.len()` must be
/// a power of two.  Purely functional (no mutation); produces a new Vec.
pub fn ntt<F: Field>(coeffs: &[F], omega: F) -> Result<Vec<F>, Error> {
    match () {
        () if coeffs.is_empty() => Err(Error::EmptyInput),
        () if !coeffs.len().is_power_of_two() =>
            Err(Error::LengthNotPowerOfTwo { len: coeffs.len() }),
        () => Ok(ntt_inner(coeffs, omega)),
    }
}

fn ntt_inner<F: Field>(coeffs: &[F], omega: F) -> Vec<F> {
    if coeffs.len() == 1 {
        vec![coeffs[0]]
    } else {
        let evens: Vec<F> = coeffs.iter().step_by(2).copied().collect();
        let odds: Vec<F> = coeffs.iter().skip(1).step_by(2).copied().collect();

        let omega_sq = omega.square();
        let evens_ntt = ntt_inner(&evens, omega_sq);
        let odds_ntt = ntt_inner(&odds, omega_sq);

        let half = coeffs.len() / 2;
        let twiddles: Vec<F> = std::iter::successors(Some(F::one()), |w| Some(*w * omega))
            .take(half)
            .collect();

        let first_half = evens_ntt.iter()
            .zip(odds_ntt.iter())
            .zip(twiddles.iter())
            .map(|((e, o), w)| *e + *w * *o);

        let second_half = evens_ntt.iter()
            .zip(odds_ntt.iter())
            .zip(twiddles.iter())
            .map(|((e, o), w)| *e - *w * *o);

        first_half.chain(second_half).collect()
    }
}

/// Inverse NTT: recover coefficients from evaluations.
/// Caller provides `omega_inv` (inverse of the root) and `n_inv` (inverse of
/// the domain size in the field) to avoid integer-to-field conversion here.
pub fn intt<F: Field>(evals: &[F], omega_inv: F, n_inv: F) -> Result<Vec<F>, Error> {
    match () {
        () if evals.is_empty() => Err(Error::EmptyInput),
        () if !evals.len().is_power_of_two() =>
            Err(Error::LengthNotPowerOfTwo { len: evals.len() }),
        () => {
            let raw = ntt_inner(evals, omega_inv);
            Ok(raw.into_iter().map(|v| v * n_inv).collect())
        }
    }
}
