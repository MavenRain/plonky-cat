use plonky_cat_field::BabyBear;
use plonky_cat_fri::FriClaim;
use plonky_cat_reduce::ClaimLens;
use plonky_cat_sumcheck::SumcheckClaim;
use plonky_cat_basefold::{BaseFoldShared, FriLens, SumcheckLens};
use proptest::prelude::*;

type F = BabyBear;

fn shared_strategy() -> impl Strategy<Value = BaseFoldShared<F>> {
    ((0u64..F::modulus()), 1usize..=8, (0u64..F::modulus()), 1usize..=4)
        .prop_map(|(root_val, cw_len, sum_val, num_vars)| {
            let root = F::from(root_val);
            let claimed_sum = F::from(sum_val);
            BaseFoldShared::new(
                FriClaim::until_constant(root, 1 << cw_len.min(4)),
                SumcheckClaim::new(claimed_sum, num_vars),
            )
        })
}

proptest! {
    #[test]
    fn fri_lens_join_split(shared in shared_strategy()) {
        let result = FriLens::<F>::check_join_split(shared)
            .map_err(|e| TestCaseError::fail(format!("{e}")))?;
        prop_assert!(result, "FriLens: join . split != id");
    }

    #[test]
    fn sumcheck_lens_join_split(shared in shared_strategy()) {
        let result = SumcheckLens::<F>::check_join_split(shared)
            .map_err(|e| TestCaseError::fail(format!("{e}")))?;
        prop_assert!(result, "SumcheckLens: join . split != id");
    }

    #[test]
    fn fri_lens_split_join(
        root_val in 0u64..F::modulus(),
        cw_len_log in 1u32..=4u32,
        sum_val in 0u64..F::modulus(),
        num_vars in 1usize..=4,
    ) {
        let fri_claim = FriClaim::until_constant(F::from(root_val), 1usize << cw_len_log);
        let sum_claim = SumcheckClaim::new(F::from(sum_val), num_vars);

        let result = FriLens::<F>::check_split_join(fri_claim, sum_claim)
            .map_err(|e| TestCaseError::fail(format!("{e}")))?;
        prop_assert!(result, "FriLens: split . join != id");
    }

    #[test]
    fn sumcheck_lens_split_join(
        root_val in 0u64..F::modulus(),
        cw_len_log in 1u32..=4u32,
        sum_val in 0u64..F::modulus(),
        num_vars in 1usize..=4,
    ) {
        let sum_claim = SumcheckClaim::new(F::from(sum_val), num_vars);
        let fri_claim = FriClaim::until_constant(F::from(root_val), 1usize << cw_len_log);

        let result = SumcheckLens::<F>::check_split_join(sum_claim, fri_claim)
            .map_err(|e| TestCaseError::fail(format!("{e}")))?;
        prop_assert!(result, "SumcheckLens: split . join != id");
    }
}
