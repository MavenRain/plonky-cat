#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;

use plonky_cat_field::Field;
use plonky_cat_hash::Hasher;
use plonky_cat_reduce::{ClaimAdapter, ClaimLens, Interleave};
use plonky_cat_fri::{Fri, FriClaim, FriOpening, FriWitness};
use plonky_cat_sumcheck::{Sumcheck, SumcheckClaim, SumcheckWitness};

// -- Shared claim: pairs a FRI codeword claim with a sumcheck claim --

#[derive(Debug, Clone)]
pub struct BaseFoldShared<F: Field> {
    fri_claim: FriClaim<F>,
    sum_claim: SumcheckClaim<F>,
}

impl<F: Field> BaseFoldShared<F> {
    #[must_use]
    pub fn new(fri_claim: FriClaim<F>, sum_claim: SumcheckClaim<F>) -> Self {
        Self { fri_claim, sum_claim }
    }

    #[must_use]
    pub fn fri_claim(&self) -> &FriClaim<F> {
        &self.fri_claim
    }

    #[must_use]
    pub fn sum_claim(&self) -> &SumcheckClaim<F> {
        &self.sum_claim
    }
}

// -- Shared witness --

#[derive(Debug, Clone)]
pub struct BaseFoldWitness<H: Hasher> {
    fri_witness: FriWitness<H>,
    sum_witness: SumcheckWitness<H::F>,
}

impl<H: Hasher> BaseFoldWitness<H> {
    #[must_use]
    pub fn new(fri_witness: FriWitness<H>, sum_witness: SumcheckWitness<H::F>) -> Self {
        Self { fri_witness, sum_witness }
    }
}

// -- Shared opening: FRI constant + sumcheck final eval --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseFoldOpening<F: Field> {
    fri_opening: FriOpening<F>,
    sum_opening: F,
}

impl<F: Field> BaseFoldOpening<F> {
    #[must_use]
    pub fn new(fri_opening: FriOpening<F>, sum_opening: F) -> Self {
        Self { fri_opening, sum_opening }
    }

    #[must_use]
    pub fn fri_opening(&self) -> &FriOpening<F> {
        &self.fri_opening
    }

    #[must_use]
    pub fn sum_opening(&self) -> F {
        self.sum_opening
    }
}

// -- Lenses: project shared state into FRI / sumcheck claims --

pub struct FriLens<F>(PhantomData<F>);

impl<F: Field> ClaimLens for FriLens<F> {
    type Whole = BaseFoldShared<F>;
    type Part = FriClaim<F>;
    type Residue = SumcheckClaim<F>;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.fri_claim, whole.sum_claim))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldShared::new(part, residue))
    }
}

pub struct SumcheckLens<F>(PhantomData<F>);

impl<F: Field> ClaimLens for SumcheckLens<F> {
    type Whole = BaseFoldShared<F>;
    type Part = SumcheckClaim<F>;
    type Residue = FriClaim<F>;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.sum_claim, whole.fri_claim))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldShared::new(residue, part))
    }
}

// -- Witness lenses --

pub struct FriWitnessLens<H>(PhantomData<H>);

impl<H: Hasher> ClaimLens for FriWitnessLens<H> {
    type Whole = BaseFoldWitness<H>;
    type Part = FriWitness<H>;
    type Residue = SumcheckWitness<H::F>;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.fri_witness, whole.sum_witness))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldWitness::new(part, residue))
    }
}

pub struct SumcheckWitnessLens<H>(PhantomData<H>);

impl<H: Hasher> ClaimLens for SumcheckWitnessLens<H> {
    type Whole = BaseFoldWitness<H>;
    type Part = SumcheckWitness<H::F>;
    type Residue = FriWitness<H>;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.sum_witness, whole.fri_witness))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldWitness::new(residue, part))
    }
}

// -- The adapter --

pub struct BaseFoldAdapter<H> {
    _marker: PhantomData<H>,
}

impl<H: Hasher> ClaimAdapter for BaseFoldAdapter<H> {
    type A = Fri<H>;
    type B = Sumcheck<H::F>;

    type Shared = BaseFoldShared<H::F>;
    type SharedWitness = BaseFoldWitness<H>;
    type SharedOpening = BaseFoldOpening<H::F>;

    type LensA = FriLens<H::F>;
    type LensB = SumcheckLens<H::F>;
    type WLensA = FriWitnessLens<H>;
    type WLensB = SumcheckWitnessLens<H>;

    type Error = Error;

    fn combine_openings(
        a: FriOpening<H::F>,
        b: H::F,
    ) -> Result<Self::SharedOpening, Self::Error> {
        Ok(BaseFoldOpening::new(a, b))
    }
}

/// BaseFold = Interleave<BaseFoldAdapter<H>>.
/// This is the v0.1 thesis: no handwritten protocol code.
pub type BaseFold<H> = Interleave<BaseFoldAdapter<H>>;
