use plonky_cat_field::{BabyBear, Field};
use crate::Hasher;

const WIDTH: usize = 3;
const FULL_ROUNDS: usize = 8;
const PARTIAL_ROUNDS: usize = 22;
const ALPHA: u64 = 7;

pub struct Poseidon2;

impl Hasher for Poseidon2 {
    type F = BabyBear;

    fn hash_pair(left: BabyBear, right: BabyBear) -> BabyBear {
        let state = [left, right, BabyBear::zero()];
        permute(state)[0]
    }
}

fn permute(state: [BabyBear; WIDTH]) -> [BabyBear; WIDTH] {
    let rc = round_constants();
    let full_half = FULL_ROUNDS / 2;

    let after_first_full = (0..full_half).fold(state, |s, r| {
        full_round(s, [rc[r * 3], rc[r * 3 + 1], rc[r * 3 + 2]])
    });

    let rc_offset = full_half * 3;
    let after_partial = (0..PARTIAL_ROUNDS).fold(after_first_full, |s, r| {
        partial_round(s, rc[rc_offset + r])
    });

    let rc_offset2 = rc_offset + PARTIAL_ROUNDS;
    (0..full_half).fold(after_partial, |s, r| {
        full_round(s, [rc[rc_offset2 + r * 3], rc[rc_offset2 + r * 3 + 1], rc[rc_offset2 + r * 3 + 2]])
    })
}

fn full_round(state: [BabyBear; WIDTH], rc: [BabyBear; WIDTH]) -> [BabyBear; WIDTH] {
    let after_add = [state[0] + rc[0], state[1] + rc[1], state[2] + rc[2]];
    let after_sbox = [sbox(after_add[0]), sbox(after_add[1]), sbox(after_add[2])];
    mds(after_sbox)
}

fn partial_round(state: [BabyBear; WIDTH], rc: BabyBear) -> [BabyBear; WIDTH] {
    let after_add = [state[0] + rc, state[1], state[2]];
    let after_sbox = [sbox(after_add[0]), after_add[1], after_add[2]];
    internal_mix(after_sbox)
}

fn sbox(x: BabyBear) -> BabyBear {
    x.pow(ALPHA)
}

fn mds(state: [BabyBear; WIDTH]) -> [BabyBear; WIDTH] {
    let s = state[0] + state[1] + state[2];
    [
        s + state[0],
        s + state[1],
        s + state[2],
    ]
}

fn internal_mix(state: [BabyBear; WIDTH]) -> [BabyBear; WIDTH] {
    let s = state[0] + state[1] + state[2];
    [
        s + state[0].double(),
        s + state[1] * BabyBear::from(3u32),
        s + state[2] * BabyBear::from(4u32),
    ]
}

fn round_constants() -> Vec<BabyBear> {
    let count = FULL_ROUNDS * WIDTH + PARTIAL_ROUNDS;
    let seed = BabyBear::from(0x5EED_C0DEu32);
    std::iter::successors(Some(seed), |prev| Some(prev.pow(ALPHA) + seed))
        .take(count)
        .collect()
}
