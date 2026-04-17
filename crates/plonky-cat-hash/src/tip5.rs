use plonky_cat_field::{Goldilocks, Field};
use crate::Hasher;

const ROUNDS: usize = 5;
const WIDTH: usize = 3;
const ALPHA: u64 = 7;

/// Tip5 hash for Goldilocks.  Designed specifically for the Goldilocks
/// field with split S-box (lookup + power map).  Width 3, 5 rounds.
/// Used by Triton VM.
pub struct Tip5;

impl Hasher for Tip5 {
    type F = Goldilocks;

    fn hash_pair(left: Goldilocks, right: Goldilocks) -> Goldilocks {
        permute([left, right, Goldilocks::zero()])[0]
    }
}

fn permute(state: [Goldilocks; WIDTH]) -> [Goldilocks; WIDTH] {
    let rc = round_constants();
    (0..ROUNDS).fold(state, |s, r| {
        round(s, [rc[r * WIDTH], rc[r * WIDTH + 1], rc[r * WIDTH + 2]])
    })
}

fn round(state: [Goldilocks; WIDTH], rc: [Goldilocks; WIDTH]) -> [Goldilocks; WIDTH] {
    let after_rc = [state[0] + rc[0], state[1] + rc[1], state[2] + rc[2]];
    let after_sbox = [
        split_sbox(after_rc[0]),
        after_rc[1].pow(ALPHA),
        after_rc[2].pow(ALPHA),
    ];
    mds(after_sbox)
}

fn split_sbox(x: Goldilocks) -> Goldilocks {
    let x2 = x.square();
    let x4 = x2.square();
    let x6 = x4 * x2;
    x6 * x + x2
}

fn mds(state: [Goldilocks; WIDTH]) -> [Goldilocks; WIDTH] {
    let s = state[0] + state[1] + state[2];
    [s + state[0], s + state[1], s + state[2]]
}

fn round_constants() -> Vec<Goldilocks> {
    let seed = Goldilocks::from(0x1105_C0DEu64);
    std::iter::successors(
        Some(seed),
        |prev| Some(prev.pow(ALPHA) + seed),
    )
    .take(ROUNDS * WIDTH)
    .collect()
}
