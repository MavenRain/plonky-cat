use plonky_cat_field::BinaryField8;
use plonky_cat_poly::MultilinearPoly;
use crate::SumcheckFunction;

/// Binius-style sumcheck witness over GF(2^8).
///
/// Binius operates on binary tower fields where addition is XOR.
/// The sumcheck protocol works identically to the prime-field case,
/// but the field arithmetic is binary.  The key advantage: field
/// operations are cheaper (XOR for add, carry-less multiply for mul).
///
/// This is a thin wrapper demonstrating that the `SumcheckFunction`
/// trait works uniformly over binary and prime fields.
#[derive(Debug, Clone)]
pub struct BiniusWitness {
    poly: MultilinearPoly<BinaryField8>,
}

impl BiniusWitness {
    #[must_use]
    pub fn new(poly: MultilinearPoly<BinaryField8>) -> Self {
        Self { poly }
    }
}

impl SumcheckFunction for BiniusWitness {
    type F = BinaryField8;

    fn num_vars(&self) -> usize { self.poly.num_vars() }
    fn round_poly_degree(&self) -> usize { 1 }

    fn round_poly_evals(&self) -> Vec<BinaryField8> {
        let (s0, s1) = self.poly.sumcheck_round_poly();
        vec![s0, s1]
    }

    fn fix_variable(self, val: BinaryField8) -> Self {
        Self { poly: self.poly.fix_variable(val) }
    }

    fn final_value(&self) -> Option<BinaryField8> {
        self.poly.evals().first().copied()
    }
}
