use plonky_cat_field::{BabyBear, Field, Mersenne31, KoalaBear};
use proptest::prelude::*;

fn nonzero_babybear() -> impl Strategy<Value = BabyBear> {
    (1u64..BabyBear::modulus()).prop_map(BabyBear::from)
}

fn nonzero_mersenne31() -> impl Strategy<Value = Mersenne31> {
    (1u64..Mersenne31::modulus()).prop_map(Mersenne31::from)
}

fn nonzero_koalabear() -> impl Strategy<Value = KoalaBear> {
    (1u64..KoalaBear::modulus()).prop_map(KoalaBear::from)
}

macro_rules! inverse_tests {
    ($name:ident, $strategy:expr) => {
        mod $name {
            use super::*;

            proptest! {
                #[test]
                fn mul_inverse_is_one(a in $strategy) {
                    let inv = a.inv().map_err(|e| TestCaseError::fail(format!("{e}")))?;
                    prop_assert_eq!(a * inv, Field::one());
                }

                #[test]
                fn inv_of_inv(a in $strategy) {
                    let inv = a.inv().map_err(|e| TestCaseError::fail(format!("{e}")))?;
                    let inv_inv = inv.inv().map_err(|e| TestCaseError::fail(format!("{e}")))?;
                    prop_assert_eq!(inv_inv, a);
                }

                #[test]
                fn pow_matches_repeated_mul(a in $strategy) {
                    let a3 = a.pow(3);
                    prop_assert_eq!(a3, a * a * a);
                }
            }
        }
    };
}

inverse_tests!(babybear_inv, nonzero_babybear());
inverse_tests!(mersenne31_inv, nonzero_mersenne31());
inverse_tests!(koalabear_inv, nonzero_koalabear());
