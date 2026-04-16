#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;

use plonky_cat_field::Field;
use plonky_cat_poly::MultilinearPoly;
use plonky_cat_reduce::{
    ProverContinue, ProverDone, ProverStep,
    ReductionFunctor,
    VerifierContinue, VerifierDone, VerifierStep,
};

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

#[derive(Debug, Clone)]
pub struct SumcheckWitness<F> {
    poly: MultilinearPoly<F>,
}

impl<F: Field> SumcheckWitness<F> {
    #[must_use]
    pub fn new(poly: MultilinearPoly<F>) -> Self {
        Self { poly }
    }

    pub fn into_poly(self) -> MultilinearPoly<F> {
        self.poly
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumcheckRoundMsg<F> {
    sum_lo: F,
    sum_hi: F,
}

impl<F: Field> SumcheckRoundMsg<F> {
    #[must_use]
    pub fn new(sum_lo: F, sum_hi: F) -> Self {
        Self { sum_lo, sum_hi }
    }

    #[must_use]
    pub fn sum_lo(&self) -> F {
        self.sum_lo
    }

    #[must_use]
    pub fn sum_hi(&self) -> F {
        self.sum_hi
    }

    #[must_use]
    pub fn evaluate_at(&self, point: F) -> F {
        self.sum_lo + (self.sum_hi - self.sum_lo) * point
    }

    #[must_use]
    pub fn claimed_sum(&self) -> F {
        self.sum_lo + self.sum_hi
    }
}

pub struct Sumcheck<F> {
    _marker: PhantomData<F>,
}

impl<F: Field> ReductionFunctor for Sumcheck<F> {
    type Claim = SumcheckClaim<F>;
    type Witness = SumcheckWitness<F>;
    type RoundMsg = SumcheckRoundMsg<F>;
    type Challenge = F;
    type BaseOpening = F;
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
            witness.poly.evals()
                .first()
                .copied()
                .ok_or(Error::WitnessEmpty)
                .map(|opening| ProverStep::Done(ProverDone::new(claim, witness, opening)))
        } else {
            let (sum_lo, sum_hi) = witness.poly.sumcheck_round_poly();
            let msg = SumcheckRoundMsg::new(sum_lo, sum_hi);
            let new_poly = witness.poly.fix_variable(challenge);
            let new_sum = msg.evaluate_at(challenge);

            Ok(ProverStep::Continue(ProverContinue::new(
                SumcheckClaim::new(new_sum, claim.num_vars_remaining - 1),
                SumcheckWitness::new(new_poly),
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
                        new_sum,
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
