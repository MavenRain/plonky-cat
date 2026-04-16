# BaseFold soundness sketch: `Interleave<BaseFoldAdapter>`

Prose sketch, not a machine-checked proof.  Companion to `plonky-cat-design.md`; all trait and type names match §2, §5, §7, §10 of that document.

## 1. Setup

Let $F$ be a finite field of prime (or prime-power) order, and let $\mathrm{RS}[F, n, \rho]$ denote the Reed-Solomon code of length $n$, rate $\rho$, and evaluation domain $D \subseteq F$.  Let $k = \log_2 n_{\mathrm{msg}}$ (where $n_{\mathrm{msg}} = \rho n$) be the number of variables of the multilinear polynomial at play.  We consider a multilinear polynomial $P : F^k \to F$ and a claimed sum $S = \sum_{x \in \{0,1\}^k} P(x) \in F$.

A BaseFold prover commits to a Merkle root of the codeword $w = \mathrm{rs\_encode}(P) \in F^n$ and proves jointly:

1. $w$ is close (up to the list-decoding radius of $\mathrm{RS}[F, n, \rho]$) to a codeword in $\mathrm{RS}[F, n, \rho]$; and
2. The unique codeword witnessed in (1) is $\mathrm{rs\_encode}$ of a multilinear $P$ whose sum over $\{0,1\}^k$ equals $S$.

In the plonky-cat vocabulary (§2, §5, §10):

```rust
type Fri<F>: ReductionFunctor;
type Sumcheck<F>: ReductionFunctor;

type Shared = BaseFoldShared<F>;  // pairs CodewordClaim + SumClaim
type SharedWitness = BaseFoldWitness<F>;  // Secret<(poly, codeword)>
type SharedOpening = BaseFoldOpening<F>;  // (fri_queries, sum_final)

type BaseFold<F> = Interleave<BaseFoldAdapter<F>>;
```

The `ClaimAdapter::LensA` and `ClaimAdapter::LensB` project `BaseFoldShared` into `Fri<F>::Claim` and `Sumcheck<F>::Claim` respectively; `ClaimAdapter::WLensA` and `ClaimAdapter::WLensB` do the same for witnesses.  The `BaseFoldShared` constructor enforces the invariant $\mathrm{rs\_encode}(P) = w$ at the point `Shared` is assembled; this is the structural hook on which the soundness argument rests.

The prover's `Witness` is `Secret<(MultilinearPoly<F>, Codeword<F>)>` per convention (§2, Q7); the verifier holds only the Merkle root of $w$, the claimed sum $S$, and the transcript state.

## 2. The round structure

Per §7, one round of `Interleave<BaseFoldAdapter>::prover_step` proceeds as follows.  Let the current shared claim be $C_i \in$ `BaseFoldShared<F>`, the witness be $W_i$, and the challenge be $r_i \in F$ squeezed from the transcript by the driver.

1. The driver calls `Interleave::prover_step(C_i, W_i, r_i)`.
2. `LensA::split(C_i)` produces `(fri_claim_i, residue_i)`.
3. `WLensA::split(W_i)` produces `(fri_witness_i, w_residue_i)`.
4. `Fri::prover_step(fri_claim_i, fri_witness_i, r_i)` returns a `ProverStep` carrying a new `fri_claim_{i+1}`, new `fri_witness_{i+1}`, and FRI round message `msg^{\mathrm{FRI}}_i`.
5. `LensA::join` and `WLensA::join` rebuild an intermediate `Shared'`.
6. `LensB::split(Shared')` produces `(sum_claim_i, residue'_i)`.
7. `Sumcheck::prover_step(sum_claim_i, sum_witness_i, r_i)` is called with the *same* challenge $r_i$, yielding `sum_claim_{i+1}`, `sum_witness_{i+1}`, and sumcheck round message $s_i \in F[X]$.
8. `LensB::join` rebuilds $C_{i+1}$; the combinator emits `InterleavedMsg(msg^{\mathrm{FRI}}_i, s_i)`.

The single shared $r_i$ is the categorical content of `Interleave`.  It is the type-level reason `sealed::SameChallenge` (§7, Q4) is load-bearing: without it, nothing forces `Fri::Challenge = Sumcheck::Challenge`, and the per-round commutativity lemma below would have nothing to say.  The sealed trait converts a semantic precondition of the BaseFold paper into a type-system obligation that fails at `impl`-resolution time if violated.

After the last round, both `Fri::prover_step` and `Sumcheck::prover_step` return `ProverStep::Done`.  If exactly one returns `Done` while the other returns `Continue`, the combinator returns `InterleaveError::DoneDesync`.  If both return `Done`, `BaseFoldAdapter::combine_openings` packages the terminal FRI query answers with the final sumcheck evaluation into a `BaseFoldOpening`, and the combinator returns `ProverStep::Done` to the driver.

## 3. The invariant and why it is preserved

The semantic invariant carried by `BaseFoldShared<F>` is:

$$ \mathrm{rs\_encode}(P) \;=\; w, $$

where $P$ is the multilinear polynomial the sumcheck side reasons about and $w$ is the codeword the FRI side reasons about.  We need this invariant to be preserved round to round, i.e., if it holds for $(P_i, w_i)$ then for the same challenge $r_i$ it holds for $(P_{i+1}, w_{i+1})$ where $P_{i+1} = \mathrm{substitute}_{r_i}(P_i)$ (sumcheck substitutes the next variable with $r_i$) and $w_{i+1} = \mathrm{fold}_{r_i}(w_i)$ (FRI folds the codeword with $r_i$).

The BaseFold correctness lemma states that these operations commute with encoding:

$$ \mathrm{fold}_{r} \circ \mathrm{rs\_encode} \;=\; \mathrm{rs\_encode} \circ \mathrm{substitute}_{r}. $$

This is the one piece of the soundness argument that is BaseFold-specific rather than combinator-generic; everything else in this sketch is about wiring.  Given the lemma, the invariant is preserved for the *same* $r$, and challenge-sharing becomes essential, not incidental.

The `ClaimLens` default methods `check_join_split` and `check_split_join` (§4, Q3) give us structural preservation of the invariant across the combinator boundary.  Property tests drive these methods under arbitrary `BaseFoldShared` values; any implementation of `LensA` or `LensB` that drops or corrupts state flunks the round-trip laws.  Combined with the per-round commutativity lemma above, this gives "well-typed implies sound for one round" as a property-test target and a Lean lemma target (see §6).

## 4. Soundness error composition

Let $\varepsilon_{\mathrm{FRI}}(n, \rho, d)$ be the IOPP soundness error for FRI over $\mathrm{RS}[F, n, \rho]$ at proximity parameter $d$, and let $\varepsilon_{\mathrm{SC}}(k, |F|)$ be the sumcheck soundness error for a $k$-variate multilinear with per-round polynomial degree bounded by $d_{\max}$ (typically $\varepsilon_{\mathrm{SC}} \le k \, d_{\max} / |F|$).  The informal composition bound for `BaseFold<F>` is:

```text
ε_BaseFold(n, ρ, k, F)  ≤  ε_FRI(n, ρ, d)  +  ε_SC(k, |F|)  -  Δ_reuse
```

where $\Delta_{\mathrm{reuse}} \ge 0$ is the correction from challenge reuse.  The correction is non-negative (the shared-challenge protocol is at least as sound as the independent-challenge protocol) because, under the random-oracle model for Fiat-Shamir, reusing one challenge queries the oracle fewer times than drawing two independent challenges; a bound on the adversary's query budget translates directly into a smaller error.  The BaseFold paper gives the precise form of $\Delta_{\mathrm{reuse}}$; we treat it informally here.

The additive decomposition is the right first approximation because one round of the interleaved reduction constitutes one FRI round *and* one sumcheck round, and the conditional probability of the verifier accepting a false claim factorizes (under independence assumptions that the RO model grants us) into the individual acceptance probabilities.  The correction term hides where that factorization is imperfect: specifically, in the event where a cheating prover exploits correlations between the same $r_i$'s effect on FRI folding and on sumcheck substitution.  The BaseFold paper argues this correlation cannot help the prover, so the correction is in the defender's favor.

For concrete parameters, the exact bound needs a careful treatment of the FRI list-decoding radius.  This sketch assumes the "unique decoding" regime; the Johnson-bound regime tightens the FRI term slightly at the cost of a more delicate argument.  That distinction is orthogonal to the combinator-level reasoning and is owned entirely by `plonky-cat-fri`.

## 5. Where the `Interleave` type machinery buys soundness

Each type-level device in §7 maps onto a specific piece of the soundness argument:

- **`sealed::SameChallenge<Ad::A, Ad::B>`** is exactly the "same $r$" precondition of the BaseFold correctness lemma.  Without it, the commutativity $\mathrm{fold}_r \circ \mathrm{rs\_encode} = \mathrm{rs\_encode} \circ \mathrm{substitute}_r$ would have no hook: FRI could fold with $r_i^{\mathrm{FRI}}$ while sumcheck substitutes with $r_i^{\mathrm{SC}}$, and the invariant would desynchronize by round 2.  The sealed trait is the type-system analogue of this side condition.

- **`ClaimLens::check_join_split` / `check_split_join`** (Q3, §4) preserve the `BaseFoldShared` invariant across split/rejoin boundaries.  In the Lean 4 translation, these correspond exactly to the two lens lemmas; in property tests, they guard against adapter implementations that silently drop one half of `Shared`.

- **`ClaimAdapter::combine_openings`** enforces that the two terminal openings are paired before `Done` is returned.  This is where the final BaseFold consistency check lives: the verifier matches the last FRI query answers against the last sumcheck evaluation, and rejects if they disagree.  Because `combine_openings` is a trait method, the combinator cannot emit `Done` without running it; there is no code path that skips the check.

- **Absence of a parallel `is_base` predicate** (Q1) eliminates the class of bugs where prover and verifier disagree on termination.  A protocol that decides termination via two sources of truth, a predicate and a `Done` variant, can desync; we have exactly one, so by construction the two parties see the same termination rule.

- **Per-crate hand-rolled errors with explicit `InA` / `InB` / `Adapter` / `DoneDesync` variants** (§11) force the combinator to surface failure modes explicitly rather than flatten them into a single error.  Soundness analysis relies on knowing *which* sub-protocol failed; flattened errors lose that information and make contest debugging harder.

## 6. What a Lean 4 formalization would look like

Translating this sketch into Lean 4 under the user's conventions (kan-tactics only, reservoir-library structure, no exceptions, no standard Mathlib tactics) follows a predictable path.

Define `ReductionFunctor` as a structure in Lean, parameterized by the six types from §2.  Define `ClaimLens` as a structure with `split` and `join` fields and two `Prop`-valued law fields corresponding directly to `check_join_split` and `check_split_join`.  Define `ClaimAdapter` as a structure bundling two `ClaimLens` instances and a `combine_openings` field.  Define `Interleave` as a function from `ClaimAdapter` to `ReductionFunctor`, with the same-challenge constraint expressed as a hypothesis on the Lean definition.

State the BaseFold composition theorem as:

> If `Fri F` is sound with error $\varepsilon_{\mathrm{FRI}}$, `Sumcheck F` is sound with error $\varepsilon_{\mathrm{SC}}$, and the per-round commutativity lemma `fold_r ∘ rs_encode = rs_encode ∘ substitute_r` holds, then `Interleave (BaseFoldAdapter F)` is sound with error at most $\varepsilon_{\mathrm{FRI}} + \varepsilon_{\mathrm{SC}}$.

Prove the theorem by induction on rounds, using the lens laws to discharge the split/rejoin obligations, using the commutativity lemma to discharge the invariant-preservation obligation, and using the Fiat-Shamir oracle bound to discharge the soundness composition.  All proof steps via kan-tactics; no `simp`, no `rw`, no `exact`, no `omega`.

The per-round commutativity lemma $\mathrm{fold}_r \circ \mathrm{rs\_encode} = \mathrm{rs\_encode} \circ \mathrm{substitute}_r$ is the piece that requires BaseFold-specific math, not combinator-level reasoning.  It belongs in a separate file under `plonky-cat-basefold/lean/` (or wherever the Lean reservoir library lives).  Everything else in the formalization is generic wiring.

The two default-method lens laws in Rust correspond to the two Lean lemmas one-to-one, which is the payoff for having chosen default trait methods over external property tests.

## 7. Caveats and non-proofs

This document is explicitly a sketch.  It does not establish:

- **Machine-checked soundness.**  The argument above is prose; no proof assistant has verified it.  Section 6 describes what formalization would look like; none has been done.

- **Exact soundness constants.**  The bound $\varepsilon_{\mathrm{FRI}} + \varepsilon_{\mathrm{SC}} - \Delta_{\mathrm{reuse}}$ is informal.  The real bound needs a careful treatment of the FRI list-decoding radius (Johnson vs. unique-decoding regime), the exact per-round degree of sumcheck univariates, and the concrete Fiat-Shamir oracle query budget.  The BaseFold paper handles these carefully; this sketch defers to it.

- **Knowledge soundness.**  Only (plain) soundness is addressed.  Knowledge soundness requires an extractor, which is standard for FRI and sumcheck individually but whose composition under `Interleave` needs a separate argument.

- **Non-idealized Fiat-Shamir.**  The random-oracle model is assumed.  A standard-model Fiat-Shamir analysis is out of scope here.

- **Invariant enforcement at the boundary.**  The argument assumes `BaseFoldShared::new` rejects any input where $\mathrm{rs\_encode}(P) \neq w$.  This shifts part of the obligation to the caller of that constructor; the plonky-cat-basefold crate owns that check, and any alternative constructor (e.g., an `unchecked` variant added later) would break the soundness guarantee.

- **Interleave for non-BaseFold adapters.**  Nothing in this sketch says `Interleave` is sound for arbitrary `ClaimAdapter`s.  The argument relies on the BaseFold-specific commutativity lemma in §3; other adapters need their own lemma and their own sketch.

These caveats are the load-bearing reasons v0.1 treats this document as a sketch rather than a proof.  Converting it into a machine-checked Lean 4 proof is deferred to v0.2 at the earliest, and gates on the per-round commutativity lemma being formalized first.
