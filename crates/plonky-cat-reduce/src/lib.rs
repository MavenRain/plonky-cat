#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;
pub use self::error::InterleaveError;
pub use self::error::SeqError;

use std::marker::PhantomData;

pub trait ReductionFunctor {
    type Claim;
    type Witness;
    type RoundMsg;
    type Challenge;
    type BaseOpening;
    type Error;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    >;

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        challenge: Self::Challenge,
    ) -> Result<VerifierStep<Self::Claim, Self::BaseOpening>, Self::Error>;
}

pub enum ProverStep<C, W, M, O> {
    Continue(ProverContinue<C, W, M>),
    Done(ProverDone<C, W, O>),
}

pub struct ProverContinue<C, W, M> {
    claim: C,
    witness: W,
    message: M,
}

impl<C, W, M> ProverContinue<C, W, M> {
    #[must_use]
    pub fn new(claim: C, witness: W, message: M) -> Self {
        Self { claim, witness, message }
    }

    pub fn into_parts(self) -> (C, W, M) {
        (self.claim, self.witness, self.message)
    }
}

pub struct ProverDone<C, W, O> {
    claim: C,
    witness: W,
    opening: O,
}

impl<C, W, O> ProverDone<C, W, O> {
    #[must_use]
    pub fn new(claim: C, witness: W, opening: O) -> Self {
        Self { claim, witness, opening }
    }

    pub fn into_parts(self) -> (C, W, O) {
        (self.claim, self.witness, self.opening)
    }
}

pub enum VerifierStep<C, O> {
    Continue(VerifierContinue<C>),
    Done(VerifierDone<C, O>),
}

pub struct VerifierContinue<C> {
    claim: C,
}

impl<C> VerifierContinue<C> {
    #[must_use]
    pub fn new(claim: C) -> Self {
        Self { claim }
    }

    pub fn into_inner(self) -> C {
        self.claim
    }
}

pub struct VerifierDone<C, O> {
    claim: C,
    opening: O,
}

impl<C, O> VerifierDone<C, O> {
    #[must_use]
    pub fn new(claim: C, opening: O) -> Self {
        Self { claim, opening }
    }

    pub fn into_parts(self) -> (C, O) {
        (self.claim, self.opening)
    }
}

pub trait ClaimLens {
    type Whole;
    type Part;
    type Residue;
    type Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error>;
    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error>;

    fn check_join_split(whole: Self::Whole) -> Result<bool, Self::Error>
    where
        Self::Whole: Clone + PartialEq,
    {
        let snapshot = whole.clone();
        let (part, residue) = Self::split(whole)?;
        let rejoined = Self::join(part, residue)?;
        Ok(rejoined == snapshot)
    }

    fn check_split_join(
        part: Self::Part,
        residue: Self::Residue,
    ) -> Result<bool, Self::Error>
    where
        Self::Part: Clone + PartialEq,
        Self::Residue: Clone + PartialEq,
    {
        let part_snap = part.clone();
        let residue_snap = residue.clone();
        let whole = Self::join(part, residue)?;
        let (part2, residue2) = Self::split(whole)?;
        Ok(part2 == part_snap && residue2 == residue_snap)
    }
}

pub trait ClaimAdapter {
    type A: ReductionFunctor;
    type B: ReductionFunctor;

    type Shared;
    type SharedWitness;
    type SharedOpening;

    type LensA: ClaimLens<
        Whole = Self::Shared,
        Part = <Self::A as ReductionFunctor>::Claim,
    >;
    type LensB: ClaimLens<
        Whole = Self::Shared,
        Part = <Self::B as ReductionFunctor>::Claim,
    >;
    type WLensA: ClaimLens<
        Whole = Self::SharedWitness,
        Part = <Self::A as ReductionFunctor>::Witness,
    >;
    type WLensB: ClaimLens<
        Whole = Self::SharedWitness,
        Part = <Self::B as ReductionFunctor>::Witness,
    >;

    type Error;

    fn combine_openings(
        a: <Self::A as ReductionFunctor>::BaseOpening,
        b: <Self::B as ReductionFunctor>::BaseOpening,
    ) -> Result<Self::SharedOpening, Self::Error>;
}

// -- Seq --

pub trait SeqAdapter {
    type A: ReductionFunctor;
    type B: ReductionFunctor;
    type Error;

    /// Validate A's base-case opening and produce B's initial claim.
    /// This function IS the verifier's base-case check for A; if the
    /// opening is invalid, return Err.
    fn handoff_claim(
        final_a: <Self::A as ReductionFunctor>::Claim,
        opening_a: <Self::A as ReductionFunctor>::BaseOpening,
    ) -> Result<<Self::B as ReductionFunctor>::Claim, Self::Error>;

    fn handoff_witness(
        final_a: <Self::A as ReductionFunctor>::Witness,
    ) -> Result<<Self::B as ReductionFunctor>::Witness, Self::Error>;
}

pub struct Seq<Ad> {
    _marker: PhantomData<Ad>,
}

pub enum SeqClaim<CA, CB> {
    PhaseA(CA),
    PhaseB(CB),
}

pub enum SeqWitness<WA, WB> {
    PhaseA(WA),
    PhaseB(WB),
}

pub enum SeqRoundMsg<MA, MB, OA> {
    PhaseA(MA),
    Transition(OA),
    PhaseB(MB),
}

impl<Ad: SeqAdapter> ReductionFunctor for Seq<Ad>
where
    <Ad::A as ReductionFunctor>::BaseOpening: Clone,
    Ad::B: ReductionFunctor<Challenge = <Ad::A as ReductionFunctor>::Challenge>,
{
    type Claim = SeqClaim<
        <Ad::A as ReductionFunctor>::Claim,
        <Ad::B as ReductionFunctor>::Claim,
    >;
    type Witness = SeqWitness<
        <Ad::A as ReductionFunctor>::Witness,
        <Ad::B as ReductionFunctor>::Witness,
    >;
    type RoundMsg = SeqRoundMsg<
        <Ad::A as ReductionFunctor>::RoundMsg,
        <Ad::B as ReductionFunctor>::RoundMsg,
        <Ad::A as ReductionFunctor>::BaseOpening,
    >;
    type Challenge = <Ad::A as ReductionFunctor>::Challenge;
    type BaseOpening = <Ad::B as ReductionFunctor>::BaseOpening;
    type Error = SeqError<
        <Ad::A as ReductionFunctor>::Error,
        <Ad::B as ReductionFunctor>::Error,
        Ad::Error,
    >;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    > {
        match (claim, witness) {
            (SeqClaim::PhaseA(ca), SeqWitness::PhaseA(wa)) => {
                Ad::A::prover_step(ca, wa, challenge)
                    .map_err(SeqError::InA)
                    .and_then(|step| match step {
                        ProverStep::Continue(c) => {
                            let (ca2, wa2, ma) = c.into_parts();
                            Ok(ProverStep::Continue(ProverContinue::new(
                                SeqClaim::PhaseA(ca2),
                                SeqWitness::PhaseA(wa2),
                                SeqRoundMsg::PhaseA(ma),
                            )))
                        }
                        ProverStep::Done(d) => {
                            let (ca_final, wa_final, oa) = d.into_parts();
                            let oa_msg = oa.clone();
                            let cb = Ad::handoff_claim(ca_final, oa)
                                .map_err(SeqError::Handoff)?;
                            let wb = Ad::handoff_witness(wa_final)
                                .map_err(SeqError::Handoff)?;
                            Ok(ProverStep::Continue(ProverContinue::new(
                                SeqClaim::PhaseB(cb),
                                SeqWitness::PhaseB(wb),
                                SeqRoundMsg::Transition(oa_msg),
                            )))
                        }
                    })
            }
            (SeqClaim::PhaseB(cb), SeqWitness::PhaseB(wb)) => {
                Ad::B::prover_step(cb, wb, challenge)
                    .map_err(SeqError::InB)
                    .map(|step| match step {
                        ProverStep::Continue(c) => {
                            let (cb2, wb2, mb) = c.into_parts();
                            ProverStep::Continue(ProverContinue::new(
                                SeqClaim::PhaseB(cb2),
                                SeqWitness::PhaseB(wb2),
                                SeqRoundMsg::PhaseB(mb),
                            ))
                        }
                        ProverStep::Done(d) => {
                            let (cb_final, wb_final, ob) = d.into_parts();
                            ProverStep::Done(ProverDone::new(
                                SeqClaim::PhaseB(cb_final),
                                SeqWitness::PhaseB(wb_final),
                                ob,
                            ))
                        }
                    })
            }
            (SeqClaim::PhaseA(_), SeqWitness::PhaseB(_)) => Err(SeqError::PhaseDesync),
            (SeqClaim::PhaseB(_), SeqWitness::PhaseA(_)) => Err(SeqError::PhaseDesync),
        }
    }

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        challenge: Self::Challenge,
    ) -> Result<VerifierStep<Self::Claim, Self::BaseOpening>, Self::Error> {
        match (claim, message) {
            (SeqClaim::PhaseA(ca), SeqRoundMsg::PhaseA(ma)) => {
                Ad::A::verifier_step(ca, ma, challenge)
                    .map_err(SeqError::InA)
                    .and_then(|step| match step {
                        VerifierStep::Continue(c) => Ok(VerifierStep::Continue(
                            VerifierContinue::new(SeqClaim::PhaseA(c.into_inner())),
                        )),
                        VerifierStep::Done(_) => Err(SeqError::PhaseDesync),
                    })
            }
            (SeqClaim::PhaseA(ca), SeqRoundMsg::Transition(oa)) => {
                Ad::handoff_claim(ca, oa)
                    .map_err(SeqError::Handoff)
                    .map(|cb| VerifierStep::Continue(
                        VerifierContinue::new(SeqClaim::PhaseB(cb)),
                    ))
            }
            (SeqClaim::PhaseB(cb), SeqRoundMsg::PhaseB(mb)) => {
                Ad::B::verifier_step(cb, mb, challenge)
                    .map_err(SeqError::InB)
                    .map(|step| match step {
                        VerifierStep::Continue(c) => VerifierStep::Continue(
                            VerifierContinue::new(SeqClaim::PhaseB(c.into_inner())),
                        ),
                        VerifierStep::Done(d) => {
                            let (cb_final, ob) = d.into_parts();
                            VerifierStep::Done(VerifierDone::new(
                                SeqClaim::PhaseB(cb_final),
                                ob,
                            ))
                        }
                    })
            }
            (SeqClaim::PhaseA(_), SeqRoundMsg::PhaseB(_)) => Err(SeqError::PhaseDesync),
            (SeqClaim::PhaseB(_), SeqRoundMsg::PhaseA(_)) => Err(SeqError::PhaseDesync),
            (SeqClaim::PhaseB(_), SeqRoundMsg::Transition(_)) => Err(SeqError::PhaseDesync),
        }
    }
}

// -- Interleave + sealed SameChallenge --

pub mod sealed {
    use super::ReductionFunctor;

    pub trait SameChallenge<A: ReductionFunctor, B: ReductionFunctor> {}

    pub struct SameChallengeWitness;

    impl<A, B> SameChallenge<A, B> for SameChallengeWitness
    where
        A: ReductionFunctor,
        B: ReductionFunctor<Challenge = <A as ReductionFunctor>::Challenge>,
    {
    }
}

pub struct Interleave<Ad> {
    _marker: PhantomData<Ad>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterleavedMsg<MA, MB> {
    msg_a: MA,
    msg_b: MB,
}

impl<MA, MB> InterleavedMsg<MA, MB> {
    #[must_use]
    pub fn new(msg_a: MA, msg_b: MB) -> Self {
        Self { msg_a, msg_b }
    }

    pub fn into_parts(self) -> (MA, MB) {
        (self.msg_a, self.msg_b)
    }
}

impl<Ad> ReductionFunctor for Interleave<Ad>
where
    Ad: ClaimAdapter,
    <Ad::A as ReductionFunctor>::Challenge: Clone,
    Ad::B: ReductionFunctor<Challenge = <Ad::A as ReductionFunctor>::Challenge>,
    <Ad::LensA as ClaimLens>::Error: Into<Ad::Error>,
    <Ad::LensB as ClaimLens>::Error: Into<Ad::Error>,
    <Ad::WLensA as ClaimLens>::Error: Into<Ad::Error>,
    <Ad::WLensB as ClaimLens>::Error: Into<Ad::Error>,
{
    type Claim = Ad::Shared;
    type Witness = Ad::SharedWitness;
    type RoundMsg = InterleavedMsg<
        <Ad::A as ReductionFunctor>::RoundMsg,
        <Ad::B as ReductionFunctor>::RoundMsg,
    >;
    type Challenge = <Ad::A as ReductionFunctor>::Challenge;
    type BaseOpening = Ad::SharedOpening;
    type Error = InterleaveError<
        <Ad::A as ReductionFunctor>::Error,
        <Ad::B as ReductionFunctor>::Error,
        Ad::Error,
    >;

    fn prover_step(
        claim: Self::Claim,
        witness: Self::Witness,
        challenge: Self::Challenge,
    ) -> Result<
        ProverStep<Self::Claim, Self::Witness, Self::RoundMsg, Self::BaseOpening>,
        Self::Error,
    > {
        let (ca, res_a) = Ad::LensA::split(claim)
            .map_err(|e| InterleaveError::Adapter(e.into()))?;
        let (wa, wres_a) = Ad::WLensA::split(witness)
            .map_err(|e| InterleaveError::Adapter(e.into()))?;

        let step_a = Ad::A::prover_step(ca, wa, challenge.clone())
            .map_err(InterleaveError::InA)?;

        match step_a {
            ProverStep::Continue(cont_a) => {
                let (ca2, wa2, msg_a) = cont_a.into_parts();
                let mid = Ad::LensA::join(ca2, res_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;
                let wmid = Ad::WLensA::join(wa2, wres_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let (cb, res_b) = Ad::LensB::split(mid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;
                let (wb, wres_b) = Ad::WLensB::split(wmid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let step_b = Ad::B::prover_step(cb, wb, challenge)
                    .map_err(InterleaveError::InB)?;

                match step_b {
                    ProverStep::Continue(cont_b) => {
                        let (cb2, wb2, msg_b) = cont_b.into_parts();
                        let final_claim = Ad::LensB::join(cb2, res_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        let final_witness = Ad::WLensB::join(wb2, wres_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        Ok(ProverStep::Continue(ProverContinue::new(
                            final_claim,
                            final_witness,
                            InterleavedMsg::new(msg_a, msg_b),
                        )))
                    }
                    ProverStep::Done(_) => Err(InterleaveError::DoneDesync),
                }
            }
            ProverStep::Done(done_a) => {
                let (ca_final, wa_final, oa) = done_a.into_parts();
                let mid = Ad::LensA::join(ca_final, res_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;
                let wmid = Ad::WLensA::join(wa_final, wres_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let (cb, res_b) = Ad::LensB::split(mid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;
                let (wb, wres_b) = Ad::WLensB::split(wmid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let step_b = Ad::B::prover_step(cb, wb, challenge)
                    .map_err(InterleaveError::InB)?;

                match step_b {
                    ProverStep::Continue(_) => Err(InterleaveError::DoneDesync),
                    ProverStep::Done(done_b) => {
                        let (cb_final, wb_final, ob) = done_b.into_parts();
                        let final_claim = Ad::LensB::join(cb_final, res_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        let final_witness = Ad::WLensB::join(wb_final, wres_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        let opening = Ad::combine_openings(oa, ob)
                            .map_err(InterleaveError::Adapter)?;
                        Ok(ProverStep::Done(ProverDone::new(
                            final_claim,
                            final_witness,
                            opening,
                        )))
                    }
                }
            }
        }
    }

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        challenge: Self::Challenge,
    ) -> Result<VerifierStep<Self::Claim, Self::BaseOpening>, Self::Error> {
        let (msg_a, msg_b) = message.into_parts();

        let (ca, res_a) = Ad::LensA::split(claim)
            .map_err(|e| InterleaveError::Adapter(e.into()))?;

        let step_a = Ad::A::verifier_step(ca, msg_a, challenge.clone())
            .map_err(InterleaveError::InA)?;

        match step_a {
            VerifierStep::Continue(cont_a) => {
                let mid = Ad::LensA::join(cont_a.into_inner(), res_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let (cb, res_b) = Ad::LensB::split(mid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let step_b = Ad::B::verifier_step(cb, msg_b, challenge)
                    .map_err(InterleaveError::InB)?;

                match step_b {
                    VerifierStep::Continue(cont_b) => {
                        let final_claim = Ad::LensB::join(cont_b.into_inner(), res_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        Ok(VerifierStep::Continue(VerifierContinue::new(final_claim)))
                    }
                    VerifierStep::Done(_) => Err(InterleaveError::DoneDesync),
                }
            }
            VerifierStep::Done(done_a) => {
                let (ca_final, oa) = done_a.into_parts();
                let mid = Ad::LensA::join(ca_final, res_a)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let (cb, res_b) = Ad::LensB::split(mid)
                    .map_err(|e| InterleaveError::Adapter(e.into()))?;

                let step_b = Ad::B::verifier_step(cb, msg_b, challenge)
                    .map_err(InterleaveError::InB)?;

                match step_b {
                    VerifierStep::Continue(_) => Err(InterleaveError::DoneDesync),
                    VerifierStep::Done(done_b) => {
                        let (cb_final, ob) = done_b.into_parts();
                        let final_claim = Ad::LensB::join(cb_final, res_b)
                            .map_err(|e| InterleaveError::Adapter(e.into()))?;
                        let opening = Ad::combine_openings(oa, ob)
                            .map_err(InterleaveError::Adapter)?;
                        Ok(VerifierStep::Done(VerifierDone::new(final_claim, opening)))
                    }
                }
            }
        }
    }
}
