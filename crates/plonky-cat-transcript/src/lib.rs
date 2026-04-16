#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_field::Field;

/// Fiat-Shamir transcript producing field-element challenges.
/// v0.1: pure (no Io wrapping); Io integration deferred to prover/verifier drivers.
pub trait Transcript: Sized {
    type F: Field;

    fn new() -> Self;
    fn absorb(self, val: Self::F) -> Self;
    fn squeeze(self) -> (Self, Self::F);
}

/// Algebraic sponge transcript for v0.1 structural testing.
/// NOT cryptographically secure.  Uses x^5 permutation for absorption and squeezing.
pub struct AlgebraicTranscript<F> {
    state: F,
}

impl<F: Field> Transcript for AlgebraicTranscript<F> {
    type F = F;

    fn new() -> Self {
        Self { state: F::zero() }
    }

    fn absorb(self, val: F) -> Self {
        let mixed = (self.state + val).pow(5) + self.state;
        Self { state: mixed }
    }

    fn squeeze(self) -> (Self, F) {
        let challenge = self.state.pow(5) + self.state.double();
        let next = challenge + self.state;
        (Self { state: next }, challenge)
    }
}
