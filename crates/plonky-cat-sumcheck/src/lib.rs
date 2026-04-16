#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;

use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;
use plonky_cat_reduce::{
    ProverContinue, ProverDone, ProverStep,
    ReductionFunctor, TranscriptSerialize,
    VerifierContinue, VerifierDone, VerifierStep,
};

// -- Claim --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumcheckClaim<F> {
    claimed_sum: F,
    num_vars_remaining: usize,
}

impl<F: Field> SumcheckClaim<F> {
    #[must_use]
    pub fn new(claimed_sum: F, num_vars_remaining: usize) -> Self {
        Self { claimed_sum, num_vars_remaining }
    }

    #[must_use]
    pub fn claimed_sum(&self) -> F {
        self.claimed_sum
    }

    #[must_use]
    pub fn num_vars_remaining(&self) -> usize {
        self.num_vars_remaining
    }
}

// -- SumcheckFunction: abstract witness --

pub trait SumcheckFunction: Sized + Clone + std::fmt::Debug {
    type F: Field;

    fn num_vars(&self) -> usize;
    fn round_poly_degree(&self) -> usize;
    fn round_poly_evals(&self) -> Vec<Self::F>;
    fn fix_variable(self, val: Self::F) -> Self;
    fn final_value(&self) -> Option<Self::F>;
}

// -- Round message: evaluations at 0, 1, ..., d --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumcheckRoundMsg<F> {
    evals: Vec<F>,
}

impl<F: Field> SumcheckRoundMsg<F> {
    #[must_use]
    pub fn new(evals: Vec<F>) -> Self {
        Self { evals }
    }

    #[must_use]
    pub fn evals(&self) -> &[F] {
        &self.evals
    }

    #[must_use]
    pub fn degree(&self) -> usize {
        self.evals.len().saturating_sub(1)
    }

    #[must_use]
    pub fn claimed_sum(&self) -> F {
        let e0 = self.evals.first().copied().unwrap_or_else(F::zero);
        let e1 = self.evals.get(1).copied().unwrap_or_else(F::zero);
        e0 + e1
    }

    #[must_use]
    pub fn evaluate_at(&self, point: F) -> F {
        lagrange_interpolate(&self.evals, point)
    }
}

impl<F: Field> TranscriptSerialize<F> for SumcheckRoundMsg<F> {
    fn to_field_elements(&self) -> Vec<F> {
        self.evals.clone()
    }
}

// -- Opening --

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SumcheckOpening<F> {
    value: F,
}

impl<F: Field> SumcheckOpening<F> {
    #[must_use]
    pub fn new(value: F) -> Self {
        Self { value }
    }

    #[must_use]
    pub fn value(&self) -> F {
        self.value
    }
}

impl<F: Field> TranscriptSerialize<F> for SumcheckOpening<F> {
    fn to_field_elements(&self) -> Vec<F> {
        vec![self.value]
    }
}

// -- Concrete witness: degree-1 (single multilinear) --

#[derive(Debug, Clone)]
pub struct LinearWitness<F: Field> {
    poly: MultilinearPoly<F>,
}

impl<F: Field> LinearWitness<F> {
    #[must_use]
    pub fn new(poly: MultilinearPoly<F>) -> Self {
        Self { poly }
    }

    pub fn into_poly(self) -> MultilinearPoly<F> {
        self.poly
    }
}

impl<F: Field> SumcheckFunction for LinearWitness<F> {
    type F = F;

    fn num_vars(&self) -> usize { self.poly.num_vars() }
    fn round_poly_degree(&self) -> usize { 1 }

    fn round_poly_evals(&self) -> Vec<F> {
        let (s0, s1) = self.poly.sumcheck_round_poly();
        vec![s0, s1]
    }

    fn fix_variable(self, val: F) -> Self {
        Self { poly: self.poly.fix_variable(val) }
    }

    fn final_value(&self) -> Option<F> {
        self.poly.evals().first().copied()
    }
}

// -- Concrete witness: degree-2 (product of two multilinears) --

#[derive(Debug, Clone)]
pub struct ProductWitness<F: Field> {
    a: MultilinearPoly<F>,
    b: MultilinearPoly<F>,
}

impl<F: Field> ProductWitness<F> {
    #[must_use]
    pub fn new(a: MultilinearPoly<F>, b: MultilinearPoly<F>) -> Self {
        Self { a, b }
    }
}

impl<F: Field> SumcheckFunction for ProductWitness<F> {
    type F = F;

    fn num_vars(&self) -> usize { self.a.num_vars() }
    fn round_poly_degree(&self) -> usize { 2 }

    fn round_poly_evals(&self) -> Vec<F> {
        let half = self.a.num_evals() / 2;
        let a_evals = self.a.evals();
        let b_evals = self.b.evals();

        let s0 = (0..half).fold(F::zero(), |acc, i| acc + a_evals[2 * i] * b_evals[2 * i]);
        let s1 = (0..half).fold(F::zero(), |acc, i| acc + a_evals[2 * i + 1] * b_evals[2 * i + 1]);
        let s2 = (0..half).fold(F::zero(), |acc, i| {
            let a2 = a_evals[2 * i + 1].double() - a_evals[2 * i];
            let b2 = b_evals[2 * i + 1].double() - b_evals[2 * i];
            acc + a2 * b2
        });

        vec![s0, s1, s2]
    }

    fn fix_variable(self, val: F) -> Self {
        Self {
            a: self.a.fix_variable(val),
            b: self.b.fix_variable(val),
        }
    }

    fn final_value(&self) -> Option<F> {
        self.a.evals().first()
            .zip(self.b.evals().first())
            .map(|(a, b)| *a * *b)
    }
}

// -- Concrete witness: sum of products (general degree) --

#[derive(Debug, Clone)]
pub struct SumOfProductsWitness<F: Field> {
    terms: Vec<Vec<MultilinearPoly<F>>>,
}

impl<F: Field> SumOfProductsWitness<F> {
    #[must_use]
    pub fn new(terms: Vec<Vec<MultilinearPoly<F>>>) -> Self {
        Self { terms }
    }
}

impl<F: Field> SumcheckFunction for SumOfProductsWitness<F> {
    type F = F;

    fn num_vars(&self) -> usize {
        self.terms.first()
            .and_then(|t| t.first())
            .map_or(0, MultilinearPoly::num_vars)
    }

    fn round_poly_degree(&self) -> usize {
        self.terms.iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
    }

    fn round_poly_evals(&self) -> Vec<F> {
        let degree = self.round_poly_degree();
        let num_eval_points = degree + 1;

        (0..num_eval_points)
            .map(|t| {
                self.terms.iter().fold(F::zero(), |acc, factors| {
                    let half = factors.first().map_or(0, |p| p.num_evals() / 2);
                    let term_sum = (0..half).fold(F::zero(), |inner_acc, i| {
                        let product = factors.iter().fold(F::one(), |prod, poly| {
                            let evals = poly.evals();
                            let val = eval_at_point(evals[2 * i], evals[2 * i + 1], t);
                            prod * val
                        });
                        inner_acc + product
                    });
                    acc + term_sum
                })
            })
            .collect()
    }

    fn fix_variable(self, val: F) -> Self {
        Self {
            terms: self.terms.into_iter()
                .map(|factors| factors.into_iter()
                    .map(|p| p.fix_variable(val))
                    .collect())
                .collect(),
        }
    }

    fn final_value(&self) -> Option<F> {
        Some(self.terms.iter().fold(F::zero(), |acc, factors| {
            let product = factors.iter().fold(F::one(), |prod, p| {
                p.evals().first().copied().map_or(F::one(), |v| prod * v)
            });
            acc + product
        }))
    }
}

// -- Sumcheck as ReductionFunctor, generic over witness function --

pub struct Sumcheck<W> {
    _marker: PhantomData<W>,
}

impl<W: SumcheckFunction> ReductionFunctor for Sumcheck<W> {
    type Claim = SumcheckClaim<W::F>;
    type Witness = W;
    type RoundMsg = SumcheckRoundMsg<W::F>;
    type Challenge = W::F;
    type BaseOpening = SumcheckOpening<W::F>;
    type Error = Error;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    > {
        if claim.num_vars_remaining == 0 {
            witness.final_value()
                .ok_or(Error::WitnessEmpty)
                .map(|val| ProverStep::Done(ProverDone::new(claim, witness, SumcheckOpening::new(val))))
        } else {
            let evals = witness.round_poly_evals();
            let msg = SumcheckRoundMsg::new(evals);
            let new_poly = witness.fix_variable(challenge);
            let new_sum = msg.evaluate_at(challenge);

            Ok(ProverStep::Continue(ProverContinue::new(
                SumcheckClaim::new(new_sum, claim.num_vars_remaining - 1),
                new_poly,
                msg,
            )))
        }
    }

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        challenge: Self::Challenge,
    ) -> Result<VerifierStep<Self::Claim, Self::BaseOpening>, Self::Error> {
        match () {
            () if claim.num_vars_remaining == 0 => Err(Error::StepOnFinished),
            () if message.claimed_sum() != claim.claimed_sum => Err(Error::RoundSumMismatch),
            () => {
                let new_sum = message.evaluate_at(challenge);
                let new_vars = claim.num_vars_remaining - 1;

                if new_vars == 0 {
                    Ok(VerifierStep::Done(VerifierDone::new(
                        SumcheckClaim::new(new_sum, 0),
                        SumcheckOpening::new(new_sum),
                    )))
                } else {
                    Ok(VerifierStep::Continue(VerifierContinue::new(
                        SumcheckClaim::new(new_sum, new_vars),
                    )))
                }
            }
        }
    }
}

// -- Backward-compatible type alias --

/// `SumcheckWitness<F>` is a backward-compatible alias for `LinearWitness<F>`.
pub type SumcheckWitness<F> = LinearWitness<F>;

// -- Lagrange interpolation over integer points 0, 1, ..., d --

fn lagrange_interpolate<F: Field>(evals: &[F], point: F) -> F {
    let d = evals.len();
    (0..d)
        .map(|i| {
            let basis = (0..d)
                .filter(|j| *j != i)
                .fold(F::one(), |acc, j| {
                    let j_field = field_from_usize::<F>(j);
                    let i_field = field_from_usize::<F>(i);
                    acc * (point - j_field) * (i_field - j_field).inv().unwrap_or_else(|_| F::one())
                });
            evals[i] * basis
        })
        .fold(F::zero(), |acc, term| acc + term)
}

fn field_from_usize<F: Field>(n: usize) -> F {
    (0..n).fold(F::zero(), |acc, _| acc + F::one())
}

fn eval_at_point<F: Field>(e0: F, e1: F, t: usize) -> F {
    let t_field = field_from_usize::<F>(t);
    e0 + (e1 - e0) * t_field
}
