#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;

use plonky_cat_field::Field;
use plonky_cat_hash::Hasher;
use plonky_cat_reduce::{ClaimAdapter, ClaimLens, Interleave};
use plonky_cat_fri::{Fri, FriClaim, FriOpening, FriWitness};
use plonky_cat_sumcheck::{
    Sumcheck, SumcheckClaim, SumcheckFunction, SumcheckOpening,
};

// -- Shared claim --

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

// -- Shared witness, generic over sumcheck function --

#[derive(Debug, Clone)]
pub struct BaseFoldWitness<H: Hasher, W> {
    fri_witness: FriWitness<H>,
    sum_witness: W,
}

impl<H: Hasher, W> BaseFoldWitness<H, W> {
    #[must_use]
    pub fn new(fri_witness: FriWitness<H>, sum_witness: W) -> Self {
        Self { fri_witness, sum_witness }
    }
}

// -- Shared opening --

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseFoldOpening<F: Field> {
    fri_opening: FriOpening<F>,
    sum_opening: SumcheckOpening<F>,
}

impl<F: Field> BaseFoldOpening<F> {
    #[must_use]
    pub fn new(fri_opening: FriOpening<F>, sum_opening: SumcheckOpening<F>) -> Self {
        Self { fri_opening, sum_opening }
    }

    #[must_use]
    pub fn fri_opening(&self) -> &FriOpening<F> {
        &self.fri_opening
    }

    #[must_use]
    pub fn sum_opening(&self) -> &SumcheckOpening<F> {
        &self.sum_opening
    }
}

// -- Claim lenses --

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

pub struct FriWitnessLens<H, W>(PhantomData<(H, W)>);

impl<H: Hasher, W: SumcheckFunction<F = H::F>> ClaimLens for FriWitnessLens<H, W> {
    type Whole = BaseFoldWitness<H, W>;
    type Part = FriWitness<H>;
    type Residue = W;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.fri_witness, whole.sum_witness))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldWitness::new(part, residue))
    }
}

pub struct SumcheckWitnessLens<H, W>(PhantomData<(H, W)>);

impl<H: Hasher, W: SumcheckFunction<F = H::F>> ClaimLens for SumcheckWitnessLens<H, W> {
    type Whole = BaseFoldWitness<H, W>;
    type Part = W;
    type Residue = FriWitness<H>;
    type Error = Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error> {
        Ok((whole.sum_witness, whole.fri_witness))
    }

    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error> {
        Ok(BaseFoldWitness::new(residue, part))
    }
}

// -- The adapter, generic over hash and sumcheck function --

pub struct BaseFoldAdapter<H, W> {
    _marker: PhantomData<(H, W)>,
}

impl<H, W> ClaimAdapter for BaseFoldAdapter<H, W>
where
    H: Hasher,
    W: SumcheckFunction<F = H::F>,
{
    type A = Fri<H>;
    type B = Sumcheck<W>;

    type Shared = BaseFoldShared<H::F>;
    type SharedWitness = BaseFoldWitness<H, W>;
    type SharedOpening = BaseFoldOpening<H::F>;

    type LensA = FriLens<H::F>;
    type LensB = SumcheckLens<H::F>;
    type WLensA = FriWitnessLens<H, W>;
    type WLensB = SumcheckWitnessLens<H, W>;

    type Error = Error;

    fn combine_openings(
        a: FriOpening<H::F>,
        b: SumcheckOpening<H::F>,
    ) -> Result<Self::SharedOpening, Self::Error> {
        Ok(BaseFoldOpening::new(a, b))
    }
}

/// BaseFold = Interleave<BaseFoldAdapter<H, W>>.
/// Generic over hash function and sumcheck witness type.
/// For degree-1 (multilinear): `BaseFold<H, LinearWitness<F>>`
/// For degree-2 (product):     `BaseFold<H, ProductWitness<F>>`
pub type BaseFold<H, W> = Interleave<BaseFoldAdapter<H, W>>;
