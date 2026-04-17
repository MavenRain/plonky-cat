use std::marker::PhantomData;
use super::{ReductionFunctor, TranscriptSerialize};

/// Recursive composition: prove that the verifier of protocol `Inner`
/// accepts, by encoding the verifier's computation as a constraint
/// in protocol `Outer`.
///
/// `Recurse<Inner, Outer, Bridge>` is a `ReductionFunctor` that:
///   1. Runs `Inner` to produce a proof
///   2. Encodes `Inner`'s verifier as an `Outer` claim
///   3. Runs `Outer` to prove the verifier accepts
///
/// The `Bridge` trait converts `Inner`'s verifier check into `Outer`'s claim.

pub trait RecursionBridge {
    type Inner: ReductionFunctor;
    type Outer: ReductionFunctor;
    type Error;

    fn encode_verifier(
        inner_claim: <Self::Inner as ReductionFunctor>::Claim,
        inner_opening: <Self::Inner as ReductionFunctor>::BaseOpening,
    ) -> Result<
        (<Self::Outer as ReductionFunctor>::Claim, <Self::Outer as ReductionFunctor>::Witness),
        Self::Error,
    >;
}

/// Recursive proof claim: either still in the inner phase or in the outer phase.
#[derive(Debug, Clone)]
pub enum RecurseClaim<CI, CO> {
    Inner(CI),
    Outer(CO),
}

#[derive(Debug, Clone)]
pub enum RecurseWitness<WI, WO> {
    Inner(WI),
    Outer(WO),
}

#[derive(Debug, Clone)]
pub enum RecurseRoundMsg<MI, MO> {
    Inner(MI),
    Outer(MO),
}

impl<F, MI, MO> TranscriptSerialize<F> for RecurseRoundMsg<MI, MO>
where
    MI: TranscriptSerialize<F>,
    MO: TranscriptSerialize<F>,
{
    fn to_field_elements(&self) -> Vec<F> {
        match self {
            Self::Inner(m) => m.to_field_elements(),
            Self::Outer(m) => m.to_field_elements(),
        }
    }
}

#[derive(Debug)]
pub enum RecurseError<EI, EO, EB> {
    Inner(EI),
    Outer(EO),
    Bridge(EB),
    PhaseDesync,
}

impl<EI: std::fmt::Display, EO: std::fmt::Display, EB: std::fmt::Display>
    std::fmt::Display for RecurseError<EI, EO, EB>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Inner(e) => write!(f, "inner: {e}"),
            Self::Outer(e) => write!(f, "outer: {e}"),
            Self::Bridge(e) => write!(f, "bridge: {e}"),
            Self::PhaseDesync => write!(f, "recursion phase desynchronization"),
        }
    }
}

/// `Recurse<B>` is a `ReductionFunctor` implementing recursive composition.
/// It sequences `Inner` then `Outer`, with the bridge encoding the verifier.
///
/// This is structurally similar to `Seq`, but the handoff is specifically
/// the recursion bridge: encoding the inner verifier as an outer constraint.
pub struct Recurse<B> {
    _marker: PhantomData<B>,
}
