# plonky-cat v0.1 core abstraction design

Red-pen resolved 2026-04-15.  Covers `plonky-cat-reduce` only; everything else in v0.1 is downstream of the types defined here.

## 1. The thesis

FRI and sumcheck are both interactive reductions of a polynomial claim.  Each consumes a verifier challenge and produces (a) a smaller claim and (b) a prover-to-verifier message per round.  We model this as an F-coalgebra: the prover-side protocol is an anamorphism (unfold) over claims; the verifier-side protocol is a catamorphism (fold) over the transcript.

If the thesis holds, BaseFold is not a new scheme; it is literally `Interleave<BaseFoldAdapter>`, where `BaseFoldAdapter: ClaimAdapter` sets `type A = Fri<F>` and `type B = Sumcheck<F>`.  The `Interleave` combinator shares one challenge and one transcript across the two functors; the adapter is a `ClaimLens` pair projecting a shared `(Codeword, SumClaim)` state into each constituent's claim type.

The v0.1 success criterion: `examples/fib-plonk-basefold.rs` typechecks and verifies using this composition (no hand-rolled BaseFold anywhere).

## 2. `ReductionFunctor`

The central trait.  Everything else in the crate is either a combinator over it, a state container for it, or a driver that runs it.

```rust
pub trait ReductionFunctor {
    type Claim;
    type Witness;       // convention: wrap in comp_cat_rs::Secret<T> for zeroization
    type RoundMsg;
    type Challenge;
    type BaseOpening;   // data delivered at termination alongside the final claim
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
    ) -> Result<
        VerifierStep<Self::Claim, Self::BaseOpening>,
        Self::Error,
    >;
}
```

Notes:
- No `&self`, no `&mut self`.  A `ReductionFunctor` is a type-level object; all state lives in `Claim` and `Witness`.  This is the static-dispatch carve-out preserved all the way through.
- **No `is_base` predicate** (Q1 resolved).  Termination is signaled exclusively by the `Done` variant returned from `prover_step` / `verifier_step`.  The step function itself is the sole termination oracle; there is no separate predicate that could desynchronize with the step logic.
- **No `prover_base` / `verifier_base` methods.**  Base-case validation happens inside `prover_step` / `verifier_step` on the round that returns `Done`.  If the base case is invalid, the step returns `Err`, not `Done`.
- `prover_step` consumes its inputs and returns a new claim, new witness, and round message (or terminates), all by move.  Immutability via ownership transfer, in line with the delay-run rule (no interior mutation).
- `type Witness` is unconstrained at the trait level; implementers set it to `Secret<InnerWitness>` by convention (Q7 resolved).  Zeroization is automatic at drop.  The convention is documented at the crate level and enforced by review, not by a trait bound, so downstream crates can pick their preferred `Secret` source.
- `type BaseOpening` (Q2 resolved) carries the data delivered at termination beyond the final claim itself.  For FRI this is the query answers; for sumcheck it is the final polynomial evaluation; for `Interleave<Ad>` it is whatever `Ad::combine_openings` produces.  Pure-reduction schemes can set `type BaseOpening = ()`.
- Steps are pure `Result`, not `Io<Result>` (Q6 resolved).  Effects (transcript, RNG) are threaded by the `prove` / `verify` drivers.  This keeps combinators verbose-free and makes soundness reasoning local to each step.

## 3. `ProverStep` and `VerifierStep`

Sum types, with opaque variants wrapping constructor-only structs (no `pub` fields, per convention).

```rust
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
    pub fn new(claim: C, witness: W, opening: O) -> Self {
        Self { claim, witness, opening }
    }

    pub fn into_parts(self) -> (C, W, O) {
        (self.claim, self.witness, self.opening)
    }
}
```

`VerifierStep<C, O>` is the mirror, without the witness:

```rust
pub enum VerifierStep<C, O> {
    Continue(VerifierContinue<C>),
    Done(VerifierDone<C, O>),
}

pub struct VerifierContinue<C> {
    claim: C,
}

impl<C> VerifierContinue<C> {
    pub fn new(claim: C) -> Self { Self { claim } }
    pub fn into_inner(self) -> C { self.claim }
}

pub struct VerifierDone<C, O> {
    claim: C,
    opening: O,
}

impl<C, O> VerifierDone<C, O> {
    pub fn new(claim: C, opening: O) -> Self {
        Self { claim, opening }
    }

    pub fn into_parts(self) -> (C, O) {
        (self.claim, self.opening)
    }
}
```

## 4. `ClaimLens`: the primitive for combinators

`Seq` and `Interleave` both need to project a "whole" claim into a "part" claim, run a sub-reduction, and rejoin.  This is a lens.  Static-dispatch form, with the two lens laws as default trait methods (Q3 resolved):

```rust
pub trait ClaimLens {
    type Whole;
    type Part;
    type Residue;
    type Error;

    fn split(whole: Self::Whole) -> Result<(Self::Part, Self::Residue), Self::Error>;
    fn join(part: Self::Part, residue: Self::Residue) -> Result<Self::Whole, Self::Error>;

    /// Law 1 (join-split): `join(split(w)) = w` on any reachable `Whole`.
    /// Property tests call this on arbitrary wholes; it should return `Ok(true)`.
    fn check_join_split(whole: Self::Whole) -> Result<bool, Self::Error>
    where
        Self::Whole: Clone + PartialEq,
    {
        let snapshot = whole.clone();
        let (part, residue) = Self::split(whole)?;
        let rejoined = Self::join(part, residue)?;
        Ok(rejoined == snapshot)
    }

    /// Law 2 (split-join): `split(join(p, r)) = (p, r)` on any reachable `(Part, Residue)`.
    /// Property tests call this on arbitrary part/residue pairs.
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
```

Property tests under `tests/` drive `check_join_split` and `check_split_join` with `proptest`-style generators.  When we eventually write the Lean 4 version, the two default methods map directly onto the two Lean lemmas `join_split_id` and `split_join_id`.

Witnesses get their own parallel lens.  Rather than duplicating the trait, a "witness lens" is just `ClaimLens` with `Whole` and `Part` set to the witness types.

## 5. `ClaimAdapter`: the Interleave glue

```rust
pub trait ClaimAdapter {
    type A: ReductionFunctor;
    type B: ReductionFunctor;

    type Shared;
    type SharedWitness;
    type SharedOpening;

    type LensA:  ClaimLens<Whole = Self::Shared,        Part = <Self::A as ReductionFunctor>::Claim>;
    type LensB:  ClaimLens<Whole = Self::Shared,        Part = <Self::B as ReductionFunctor>::Claim>;
    type WLensA: ClaimLens<Whole = Self::SharedWitness, Part = <Self::A as ReductionFunctor>::Witness>;
    type WLensB: ClaimLens<Whole = Self::SharedWitness, Part = <Self::B as ReductionFunctor>::Witness>;

    type Error;

    /// Combine A's and B's base openings into the interleaved opening.
    /// Called only when both sub-functors return `Done` in the same round.
    fn combine_openings(
        a: <Self::A as ReductionFunctor>::BaseOpening,
        b: <Self::B as ReductionFunctor>::BaseOpening,
    ) -> Result<Self::SharedOpening, Self::Error>;
}
```

No termination predicates on the adapter; termination comes from the underlying A and B each returning `Done` in the same round.  If one terminates before the other, that is a `DoneDesync` error flagged at the `Interleave` layer.

The challenge-uniformity constraint (A and B share a single challenge per round) is enforced at the `Interleave` `impl`, not on the adapter trait, via the sealed-trait mechanism in §7.

## 6. `Seq` combinator

`Seq<Ad>` runs `A` to its `Done`, then hands off to `B`.

```rust
pub struct Seq<Ad> {
    _marker: PhantomData<Ad>,
}

pub trait SeqAdapter {
    type A: ReductionFunctor;
    type B: ReductionFunctor;
    type Error;

    fn handoff_claim(
        final_a: <Self::A as ReductionFunctor>::Claim,
        opening_a: <Self::A as ReductionFunctor>::BaseOpening,
    ) -> Result<<Self::B as ReductionFunctor>::Claim, Self::Error>;

    fn handoff_witness(
        final_a: <Self::A as ReductionFunctor>::Witness,
    ) -> Result<<Self::B as ReductionFunctor>::Witness, Self::Error>;
}

pub enum SeqClaim<CA, CB> {
    PhaseA(CA),
    PhaseB(CB),
}

pub enum SeqWitness<WA, WB> {
    PhaseA(WA),
    PhaseB(WB),
}

pub enum SeqRoundMsg<MA, MB> {
    PhaseA(MA),
    PhaseB(MB),
}

pub enum SeqError<EA, EB, EAd> {
    InA(EA),
    InB(EB),
    Handoff(EAd),
    PhaseDesync,
}
```

`Seq::prover_step` dispatches on whether the current `SeqClaim` is `PhaseA` or `PhaseB`.  When `A` returns `Done` in phase A, the adapter runs and the next call starts phase B with the handed-off claim.  No `_` wildcards; exhaust every variant.

## 7. `Interleave` combinator (the one that earns its keep)

```rust
pub struct Interleave<Ad> {
    _marker: PhantomData<Ad>,
}

mod sealed {
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

impl<Ad> ReductionFunctor for Interleave<Ad>
where
    Ad: ClaimAdapter,
    sealed::SameChallengeWitness: sealed::SameChallenge<Ad::A, Ad::B>,
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
        // 1. Split shared claim via LensA; split shared witness via WLensA.
        // 2. Call Ad::A::prover_step with the challenge; receive a ProverStep.
        // 3. Rejoin A's result into `shared'` via LensA / WLensA.
        // 4. Split `shared'` via LensB + WLensB.
        // 5. Call Ad::B::prover_step with the *same* challenge.
        // 6. Rejoin B's result into `shared''`.
        // 7. Termination rule:
        //      both A and B returned Continue  -> Continue(InterleavedMsg(msg_a, msg_b))
        //      both A and B returned Done      -> Done(combine_openings(o_a, o_b))
        //      exactly one returned Done        -> Err(DoneDesync)
        //    No `_` wildcard; match all four variant combinations explicitly.
        todo!("v0.1 spike implementation")
    }

    fn verifier_step(
        claim: Self::Claim,
        message: Self::RoundMsg,
        challenge: Self::Challenge,
    ) -> Result<
        VerifierStep<Self::Claim, Self::BaseOpening>,
        Self::Error,
    > {
        todo!("v0.1 spike implementation")
    }
}

pub struct InterleavedMsg<MA, MB> {
    msg_a: MA,
    msg_b: MB,
}

impl<MA, MB> InterleavedMsg<MA, MB> {
    pub fn new(msg_a: MA, msg_b: MB) -> Self { Self { msg_a, msg_b } }
    pub fn into_parts(self) -> (MA, MB) { (self.msg_a, self.msg_b) }
}

pub enum InterleaveError<EA, EB, EAd> {
    InA(EA),
    InB(EB),
    Adapter(EAd),
    DoneDesync,
}
```

The sealed-trait mechanism (Q4 resolved) enforces `A::Challenge = B::Challenge` at the type level.  `SameChallengeWitness` is the only type that can implement `SameChallenge<A, B>`, and it can do so only when the challenge types match.  A misapplied `Interleave` produces an error message naming `SameChallenge` directly, which is far more readable than the `From`/`Into`-based alternative.

If a future scheme needs independent challenges, introduce `InterleaveIndep<Ad>` as a sibling with its own sealed-trait or no challenge-uniformity constraint; do not weaken `Interleave`.

## 8. `Transcript` (lives in `plonky-cat-transcript`)

Summary here; full design lives in its own crate.

```rust
pub trait Transcript: Sized {
    type Challenge;
    type Error;

    fn absorb<M: AsTranscriptBytes>(self, message: M) -> Io<Self, Self::Error>;
    fn squeeze(self) -> Io<(Self, Self::Challenge), Self::Error>;
}
```

Every transcript operation returns a new `Transcript` wrapped in `Io`.  No interior mutation; each call is a move.  This is the delay-run rule applied to Fiat-Shamir state: the transcript is effectful (reads entropy, hashes) but only manifests at the outer `.run()` boundary.

## 9. `prove` and `verify` drivers

These are the anamorphism and catamorphism, respectively.  Both stay inside `Io`.

```rust
pub fn prove<R, T>(
    initial_claim: R::Claim,
    initial_witness: R::Witness,
    transcript: T,
) -> Io<Proof<R::RoundMsg, R::BaseOpening>, ProveError<R::Error, T::Error>>
where
    R: ReductionFunctor,
    T: Transcript<Challenge = R::Challenge>,
{
    // Unfold (no is_base; termination signaled by ProverStep::Done):
    //
    //   loop (claim, witness, transcript, collected_msgs) {
    //     let (transcript, challenge) = transcript.squeeze().await?;
    //     match R::prover_step(claim, witness, challenge)? {
    //       ProverStep::Continue(c) => {
    //         let (claim_next, witness_next, msg) = c.into_parts();
    //         let transcript = transcript.absorb(&msg).await?;
    //         collected_msgs.push(msg);
    //         continue with (claim_next, witness_next, transcript, collected_msgs);
    //       }
    //       ProverStep::Done(d) => {
    //         let (_final_claim, _final_witness, opening) = d.into_parts();
    //         return Proof::new(collected_msgs, opening);
    //       }
    //     }
    //   }
    //
    // Written as a successors-based fold inside Io, not a while loop.  No `scan`
    // combinator.  Match on ProverStep is two-armed and exhaustive.
    todo!("v0.1 spike implementation")
}

pub fn verify<R, T>(
    initial_claim: R::Claim,
    proof: Proof<R::RoundMsg, R::BaseOpening>,
    transcript: T,
) -> Io<Verdict, VerifyError<R::Error, T::Error>>
where
    R: ReductionFunctor,
    T: Transcript<Challenge = R::Challenge>,
{
    // Mirror fold over the proof stream, reading messages and the final opening
    // and checking that verifier_step returns Done on the last message.
    todo!("v0.1 spike implementation")
}
```

`Proof<RoundMsg, BaseOpening>` (Q5 resolved) is a flat `Vec<RoundMsg>` plus the terminal `BaseOpening`, nothing fancier.  `RoundMsg` for `Interleave<Ad>` is `InterleavedMsg<MA, MB>`, so individual entries are structured; the proof spine itself is flat.  Tree-structured proofs keyed by combinator shape can come in v0.2 if serialization pressure demands it.

## 10. Worked example: BaseFold

Before v0.1 code lands, the test of the abstraction is whether we can sketch BaseFold in types without hand-waving.

```rust
// In plonky-cat-basefold.

use comp_cat_rs::Secret;

pub struct BaseFoldAdapter<F: Field> {
    _marker: PhantomData<F>,
}

pub struct BaseFoldShared<F: Field> {
    codeword_claim: CodewordClaim<F>,  // from plonky-cat-fri
    sum_claim:      SumClaim<F>,       // from plonky-cat-sumcheck
    // Invariant (enforced by constructor):
    //   rs_encode(sum_claim.poly_commitment) == codeword_claim.word_commitment
}

impl<F: Field> BaseFoldShared<F> {
    pub fn new(
        codeword_claim: CodewordClaim<F>,
        sum_claim: SumClaim<F>,
    ) -> Result<Self, BaseFoldAdapterError> {
        // Check the RS-encode consistency invariant here.
        Ok(Self { codeword_claim, sum_claim })
    }
}

pub struct BaseFoldWitness<F: Field> {
    inner: Secret<BaseFoldWitnessInner<F>>,
}

struct BaseFoldWitnessInner<F: Field> {
    poly:     MultilinearPoly<F>,
    codeword: Codeword<F>,
}

pub struct BaseFoldOpening<F: Field> {
    fri_queries: FriQueryAnswers<F>,
    sum_final:   F,
}

impl<F: Field> BaseFoldOpening<F> {
    pub fn new(fri_queries: FriQueryAnswers<F>, sum_final: F) -> Self {
        Self { fri_queries, sum_final }
    }
    pub fn into_parts(self) -> (FriQueryAnswers<F>, F) {
        (self.fri_queries, self.sum_final)
    }
}

impl<F: Field> ClaimAdapter for BaseFoldAdapter<F> {
    type A = Fri<F>;
    type B = Sumcheck<F>;

    type Shared        = BaseFoldShared<F>;
    type SharedWitness = BaseFoldWitness<F>;
    type SharedOpening = BaseFoldOpening<F>;

    type LensA  = FriLens<F>;
    type LensB  = SumcheckLens<F>;
    type WLensA = FriWitnessLens<F>;
    type WLensB = SumcheckWitnessLens<F>;

    type Error = BaseFoldAdapterError;

    fn combine_openings(
        a: FriQueryAnswers<F>,
        b: F,
    ) -> Result<BaseFoldOpening<F>, BaseFoldAdapterError> {
        Ok(BaseFoldOpening::new(a, b))
    }
}

pub type BaseFold<F> = Interleave<BaseFoldAdapter<F>>;
```

If this typechecks, `examples/fib-plonk-basefold.rs` drives `prove::<BaseFold<Goldilocks>, _>(...)` and we're done.  If it does not typecheck, the abstraction needs revision; that is precisely the signal we want out of v0.1.

The claim invariant on `BaseFoldShared` (that `rs_encode(poly) == word`) is where BaseFold's soundness proof lives.  The type cannot enforce it structurally, but the constructor can reject any `BaseFoldShared` where it fails; combined with the `ClaimLens` default-method laws, this gives us "well-typed implies sound for one round" as a property test target.

## 11. Error strategy

Per-crate hand-rolled enums, as locked.  `plonky-cat-reduce` exposes:

```rust
pub enum Error {
    StepOnFinishedClaim,
    ChallengeUnused,
    MessageConsistencyFailure,
}
```

Combinators wrap their components' errors explicitly rather than `From`-flattening:

```rust
pub enum SeqError<EA, EB, EAd>        { /* as §6 */ }
pub enum InterleaveError<EA, EB, EAd> { /* as §7 */ }
```

`plonky-cat-prover::Error` is the thin workspace-facing wrapper with `From` impls from each constituent error.  Users of the `prove` / `verify` functions see only the prover error type; internal errors are carried through.

## 12. Resolved decisions (2026-04-15)

All seven red-pen questions resolved.

1. **`is_base` killed.**  Termination signaled exclusively by `ProverStep::Done` / `VerifierStep::Done`.  The step function is the sole oracle.  No separate `prover_base` / `verifier_base` methods either; base-case validation happens inside the step that returns `Done`.

2. **`BaseOpening` associated type added.**  Both step functions return it via the `Done` variant.  In `Interleave<Ad>`, the adapter combines A's and B's openings via `ClaimAdapter::combine_openings`.  Pure-reduction schemes set `type BaseOpening = ()`.

3. **`ClaimLens` laws as default trait methods.**  `check_join_split` and `check_split_join` live on the trait with `Clone + PartialEq` bounds on the relevant associated types.  Property tests drive them from `tests/`.  Direct Lean 4 lemma correspondence when we formalize.

4. **`SameChallenge` sealed trait.**  Interleave's challenge-uniformity constraint is enforced by `sealed::SameChallengeWitness: sealed::SameChallenge<Ad::A, Ad::B>`, not a `where A::Challenge = B::Challenge` clause.  Error messages name `SameChallenge` directly when misapplied.

5. **`Proof<RoundMsg, BaseOpening>` is flat.**  Flat `Vec<RoundMsg>` plus a terminal `BaseOpening`.  Tree-structured proofs keyed by combinator shape deferred to v0.2 if needed.

6. **Steps are pure.**  `prover_step` and `verifier_step` return `Result`, not `Io<Result>`.  Effects (transcript, RNG) threaded by the `prove` / `verify` drivers, not by individual steps.

7. **Witnesses wrap in `Secret<T>`.**  By convention, not by trait bound.  Each implementer sets `type Witness = Secret<InnerWitness>`.  Zeroization automatic at drop.  The `Witness` associated type is left unconstrained so downstream crates can pick their own `Secret` source (`comp-cat-rs` re-export preferred).

## 13. What this document commits to

Fixed for v0.1:

- `ReductionFunctor` as the sole protocol-shape trait, with the six associated types above.
- Anamorphism/catamorphism framing for prove/verify.
- Termination exclusively via the `Done` variant; no parallel predicate.
- `ClaimLens` as the sole compositionality primitive, with laws as default trait methods.
- `Seq` and `Interleave` as the only combinators.  No `Par`, no `Choose`, no `Race`.  Those go in v0.2 if ever.
- BaseFold implemented solely as `type BaseFold<F> = Interleave<BaseFoldAdapter<F>>`, with zero handwritten protocol code.
- Per-crate errors, thin workspace wrapper at `plonky-cat-prover::Error`.
- Witnesses wrapped in `Secret<T>` by convention; steps pure; `SameChallenge` enforced by sealed trait.

Everything below that level (concrete FRI parameters, concrete sumcheck polynomial shapes, transcript hash choice) is free to change later without disturbing the trait.
