# plonky-cat

A `comp-cat-rs`-based rebuild of the Plonky3 proving toolkit.  v0.1 unifies FRI and sumcheck as F-coalgebras of a shared claim functor; BaseFold is literal functor composition (`Interleave<BaseFoldAdapter>`).  See `DESIGN.md` for the full abstraction design.

## Crate layout

| Crate | Purpose |
|---|---|
| `plonky-cat-field` | Field abstractions; BabyBear, Goldilocks, Mersenne31 |
| `plonky-cat-poly` | Univariate and multilinear polynomial functors |
| `plonky-cat-fft` | FFT/NTT including four-step Bailey algorithm |
| `plonky-cat-code` | Error-correcting codes: Reed-Solomon, tensor-RS, Reed-Muller |
| `plonky-cat-hash` | Arithmetization-friendly and generic hash functions |
| `plonky-cat-merkle` | Merkle tree over `plonky-cat-hash` |
| `plonky-cat-transcript` | Fiat-Shamir transcript as a natural transformation |
| `plonky-cat-reduce` | **Core**: unified claim-reduction coalgebra (`ReductionFunctor`) |
| `plonky-cat-fri` | FRI as a `ReductionFunctor` implementation |
| `plonky-cat-sumcheck` | Sumcheck as a `ReductionFunctor` implementation |
| `plonky-cat-basefold` | BaseFold as `Interleave<BaseFoldAdapter>` |
| `plonky-cat-tensor-pcs` | Tensor polynomial commitment schemes: Ligero, Brakedown, Orion |
| `plonky-cat-plonk` | PLONK arithmetization with custom gates |
| `plonky-cat-prover` | Prover driver: anamorphism over `ReductionFunctor` |
| `plonky-cat-verifier` | Verifier driver: catamorphism over `ReductionFunctor` |

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
