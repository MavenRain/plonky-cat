#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;

use plonky_cat_field::Field;
use plonky_cat_hash::Hasher;
use plonky_cat_merkle::MerkleTree;
use plonky_cat_reduce::{
    ProverContinue, ProverDone, ProverStep, TranscriptSerialize,
    ReductionFunctor,
    VerifierContinue, VerifierDone, VerifierStep,
};

// -- Claims and witnesses --

#[derive(Debug, Clone)]
pub struct FriClaim<F> {
    merkle_root: F,
    codeword_len: usize,
}

impl<F: Field> FriClaim<F> {
    #[must_use]
    pub fn new(merkle_root: F, codeword_len: usize) -> Self {
        Self { merkle_root, codeword_len }
    }

    #[must_use]
    pub fn merkle_root(&self) -> F {
        self.merkle_root
    }

    #[must_use]
    pub fn codeword_len(&self) -> usize {
        self.codeword_len
    }
}

#[derive(Debug, Clone)]
pub struct FriWitness<H: Hasher> {
    codeword: Vec<H::F>,
    tree: MerkleTree<H>,
}

impl<H: Hasher> FriWitness<H> {
    pub fn build(codeword: Vec<H::F>) -> Result<Self, Error> {
        match () {
            () if codeword.is_empty() => Err(Error::CodewordEmpty),
            () if !codeword.len().is_power_of_two() =>
                Err(Error::CodewordLengthNotPowerOfTwo { len: codeword.len() }),
            () => {
                let tree = MerkleTree::<H>::build(codeword.clone())?;
                Ok(Self { codeword, tree })
            }
        }
    }

    pub fn merkle_root(&self) -> Result<H::F, Error> {
        self.tree.root().map_err(Error::from)
    }

    #[must_use]
    pub fn codeword(&self) -> &[H::F] {
        &self.codeword
    }
}

// -- Round message: the folded codeword's Merkle root --

#[derive(Debug, Clone)]
pub struct FriRoundMsg<F> {
    folded_root: F,
}

impl<F: Field> FriRoundMsg<F> {
    #[must_use]
    pub fn new(folded_root: F) -> Self {
        Self { folded_root }
    }

    #[must_use]
    pub fn folded_root(&self) -> F {
        self.folded_root
    }
}

impl<F: Field> TranscriptSerialize<F> for FriRoundMsg<F> {
    fn to_field_elements(&self) -> Vec<F> {
        vec![self.folded_root]
    }
}

// -- Base opening: the final (constant) codeword value --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FriOpening<F> {
    constant_value: F,
}

impl<F: Field> FriOpening<F> {
    #[must_use]
    pub fn new(constant_value: F) -> Self {
        Self { constant_value }
    }

    pub fn into_constant_value(self) -> F {
        self.constant_value
    }

    #[must_use]
    pub fn constant_value(&self) -> F {
        self.constant_value
    }
}

impl<F: Field> TranscriptSerialize<F> for FriOpening<F> {
    fn to_field_elements(&self) -> Vec<F> {
        vec![self.constant_value]
    }
}

// -- FRI folding: halve the codeword with challenge r --
// fold(w, r)[i] = (w[2i] + w[2i+1]) / 2 + r * (w[2i] - w[2i+1]) / 2
// Simplified: fold(w, r)[i] = w[2i] * (1 + r) / 2 + w[2i+1] * (1 - r) / 2
// Even simpler for v0.1: fold(w, r)[i] = w[2i] + r * w[2i+1]

fn fold_codeword<F: Field>(codeword: &[F], challenge: F) -> Vec<F> {
    let half = codeword.len() / 2;
    (0..half)
        .map(|i| codeword[2 * i] + challenge * codeword[2 * i + 1])
        .collect()
}

// -- FRI as ReductionFunctor --

pub struct Fri<H> {
    _marker: PhantomData<H>,
}

impl<H: Hasher> ReductionFunctor for Fri<H>
where
    H::F: Field,
{
    type Claim = FriClaim<H::F>;
    type Witness = FriWitness<H>;
    type RoundMsg = FriRoundMsg<H::F>;
    type Challenge = H::F;
    type BaseOpening = FriOpening<H::F>;
    type Error = Error;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    > {
        if claim.codeword_len <= 1 {
            witness.codeword.first()
                .copied()
                .ok_or(Error::CodewordEmpty)
                .map(|val| ProverStep::Done(ProverDone::new(
                    claim,
                    witness,
                    FriOpening::new(val),
                )))
        } else {
            let folded = fold_codeword(&witness.codeword, challenge);
            let new_witness = FriWitness::<H>::build(folded)?;
            let new_root = new_witness.tree.root()?;
            let msg = FriRoundMsg::new(new_root);

            Ok(ProverStep::Continue(ProverContinue::new(
                FriClaim::new(new_root, claim.codeword_len / 2),
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
        if claim.codeword_len <= 1 {
            Err(Error::StepOnFinished)
        } else {
            let new_len = claim.codeword_len / 2;

            if new_len <= 1 {
                Ok(VerifierStep::Done(VerifierDone::new(
                    FriClaim::new(message.folded_root, new_len),
                    FriOpening::new(message.folded_root),
                )))
            } else {
                Ok(VerifierStep::Continue(VerifierContinue::new(
                    FriClaim::new(message.folded_root, new_len),
                )))
            }
        }
    }
}
