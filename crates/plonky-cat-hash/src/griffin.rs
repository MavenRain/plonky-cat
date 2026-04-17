use plonky_cat_field::{BabyBear, Field};
use crate::Hasher;

const ROUNDS: usize = 12;
const ALPHA: u64 = 7;
const ALPHA_INV: u64 = 1_725_656_503;

/// Griffin hash for BabyBear.  State width 3, forward S-box x^7,
/// inverse S-box x^{7^{-1} mod (p-1)}, 12 rounds.
pub struct Griffin;

impl Hasher for Griffin {
    type F = BabyBear;

    fn hash_pair(left: BabyBear, right: BabyBear) -> BabyBear {
        permute([left, right, BabyBear::zero()])[0]
    }
}

fn permute(state: [BabyBear; 3]) -> [BabyBear; 3] {
    let rc = round_constants();
    (0..ROUNDS).fold(state, |s, r| round(s, [rc[r * 3], rc[r * 3 + 1], rc[r * 3 + 2]]))
}

fn round(state: [BabyBear; 3], rc: [BabyBear; 3]) -> [BabyBear; 3] {
    let y0 = (state[0] + rc[0]).pow(ALPHA);
    let y1 = (state[1] + rc[1]).pow(ALPHA_INV);
    let y2 = state[2] + rc[2];
    let l = y0 + y1 + y2;
    [y0 + l, y1 + l, y2 + l]
}

fn round_constants() -> Vec<BabyBear> {
    let seed = BabyBear::from(0x6F1F_F100u32);
    std::iter::successors(Some(seed), |prev| Some(prev.pow(ALPHA) + seed))
        .take(ROUNDS * 3)
        .collect()
}
