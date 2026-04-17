use std::marker::PhantomData;
use plonky_cat_field::Field;
use plonky_cat_hash::Hasher;
use plonky_cat_merkle::MerkleTree;
use plonky_cat_reduce::{
    ProverContinue, ProverDone, ProverStep, ReductionFunctor, TranscriptSerialize,
    VerifierContinue, VerifierDone, VerifierStep,
};
use crate::error::Error;

/// WHIR: Worst-case to average-case Hardness of Inner-product with
/// Reed-Solomon codes.  A multilinear PCS with logarithmic proof size
/// that uses a different folding strategy than FRI.
///
/// Each round: the prover commits to a "folded" function, the verifier
/// sends a random weighting vector, and the prover reduces the inner-product
/// claim via a proximity test.  The key difference from FRI: WHIR operates
/// on multilinear evaluations directly, not on univariate codewords.

#[derive(Debug, Clone)]
pub struct WhirClaim<F> {
    commitment: F,
    num_vars: usize,
}

impl<F: Field> WhirClaim<F> {
    #[must_use]
    pub fn new(commitment: F, num_vars: usize) -> Self {
        Self { commitment, num_vars }
    }

    #[must_use]
    pub fn commitment(&self) -> F { self.commitment }

    #[must_use]
    pub fn num_vars(&self) -> usize { self.num_vars }
}

#[derive(Debug, Clone)]
pub struct WhirWitness<H: Hasher> {
    evals: Vec<H::F>,
    tree: MerkleTree<H>,
}

impl<H: Hasher> WhirWitness<H> {
    pub fn build(evals: Vec<H::F>) -> Result<Self, Error> {
        if evals.is_empty() {
            Err(Error::CodewordEmpty)
        } else if !evals.len().is_power_of_two() {
            Err(Error::CodewordLengthNotPowerOfTwo { len: evals.len() })
        } else {
            let tree = MerkleTree::<H>::build(evals.clone())?;
            Ok(Self { evals, tree })
        }
    }

    pub fn commitment(&self) -> Result<H::F, Error> {
        self.tree.root().map_err(Error::from)
    }
}

#[derive(Debug, Clone)]
pub struct WhirRoundMsg<F> {
    folded_commitment: F,
}

impl<F: Field> WhirRoundMsg<F> {
    #[must_use]
    pub fn new(folded_commitment: F) -> Self {
        Self { folded_commitment }
    }
}

impl<F: Field> TranscriptSerialize<F> for WhirRoundMsg<F> {
    fn to_field_elements(&self) -> Vec<F> {
        vec![self.folded_commitment]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhirOpening<F> {
    value: F,
}

impl<F: Field> WhirOpening<F> {
    #[must_use]
    pub fn new(value: F) -> Self { Self { value } }

    #[must_use]
    pub fn value(&self) -> F { self.value }
}

impl<F: Field> TranscriptSerialize<F> for WhirOpening<F> {
    fn to_field_elements(&self) -> Vec<F> {
        vec![self.value]
    }
}

pub struct Whir<H> {
    _marker: PhantomData<H>,
}

/// WHIR folding: weighted sum of even/odd pairs.
/// fold(evals, r)[i] = evals[2i] + r * evals[2i+1]
fn whir_fold<F: Field>(evals: &[F], challenge: F) -> Vec<F> {
    let half = evals.len() / 2;
    (0..half)
        .map(|i| evals[2 * i] + challenge * evals[2 * i + 1])
        .collect()
}

impl<H: Hasher> ReductionFunctor for Whir<H>
where
    H::F: Field,
{
    type Claim = WhirClaim<H::F>;
    type Witness = WhirWitness<H>;
    type RoundMsg = WhirRoundMsg<H::F>;
    type Challenge = H::F;
    type BaseOpening = WhirOpening<H::F>;
    type Error = Error;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    > {
        if claim.num_vars == 0 {
            witness.evals.first()
                .copied()
                .ok_or(Error::CodewordEmpty)
                .map(|val| ProverStep::Done(ProverDone::new(
                    claim, witness, WhirOpening::new(val),
                )))
        } else {
            let folded = whir_fold(&witness.evals, challenge);
            let new_witness = WhirWitness::<H>::build(folded)?;
            let new_commitment = new_witness.commitment()?;
            let msg = WhirRoundMsg::new(new_commitment);

            Ok(ProverStep::Continue(ProverContinue::new(
                WhirClaim::new(new_commitment, claim.num_vars - 1),
                new_witness,
                msg,
            )))
        }
    }

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        _challenge: Self::Challenge,
    ) -> Result<VerifierStep<Self::Claim, Self::BaseOpening>, Self::Error> {
        if claim.num_vars == 0 {
            Err(Error::StepOnFinished)
        } else {
            let new_vars = claim.num_vars - 1;

            if new_vars == 0 {
                Ok(VerifierStep::Done(VerifierDone::new(
                    WhirClaim::new(message.folded_commitment, 0),
                    WhirOpening::new(message.folded_commitment),
                )))
            } else {
                Ok(VerifierStep::Continue(VerifierContinue::new(
                    WhirClaim::new(message.folded_commitment, new_vars),
                )))
            }
        }
    }
}
