use plonky_cat_field::{BabyBear, BinaryField8, Field, Goldilocks, KoalaBear, Mersenne31};
use proptest::prelude::*;

fn babybear_strategy() -> impl Strategy<Value = BabyBear> {
    (0u64..BabyBear::modulus()).prop_map(BabyBear::from)
}

fn mersenne31_strategy() -> impl Strategy<Value = Mersenne31> {
    (0u64..Mersenne31::modulus()).prop_map(Mersenne31::from)
}

fn koalabear_strategy() -> impl Strategy<Value = KoalaBear> {
    (0u64..KoalaBear::modulus()).prop_map(KoalaBear::from)
}

fn goldilocks_strategy() -> impl Strategy<Value = Goldilocks> {
    any::<u64>().prop_map(Goldilocks::from)
}

fn binary8_strategy() -> impl Strategy<Value = BinaryField8> {
    (0u32..256).prop_map(BinaryField8::from)
}

macro_rules! field_axiom_tests {
    ($name:ident, $strategy:expr) => {
        mod $name {
            use super::*;

            proptest! {
                #[test]
                fn add_commutative(a in $strategy, b in $strategy) {
                    prop_assert_eq!(a + b, b + a);
                }

                #[test]
                fn add_associative(a in $strategy, b in $strategy, c in $strategy) {
                    prop_assert_eq!((a + b) + c, a + (b + c));
                }

                #[test]
                fn add_identity(a in $strategy) {
                    let z = <_ as Field>::zero();
                    prop_assert_eq!(a + z, a);
                }

                #[test]
                fn add_inverse(a in $strategy) {
                    prop_assert_eq!(a + (-a), Field::zero());
                }

                #[test]
                fn mul_commutative(a in $strategy, b in $strategy) {
                    prop_assert_eq!(a * b, b * a);
                }

                #[test]
                fn mul_associative(a in $strategy, b in $strategy, c in $strategy) {
                    prop_assert_eq!((a * b) * c, a * (b * c));
                }

                #[test]
                fn mul_identity(a in $strategy) {
                    let one = <_ as Field>::one();
                    prop_assert_eq!(a * one, a);
                }

                #[test]
                fn distributive(a in $strategy, b in $strategy, c in $strategy) {
                    prop_assert_eq!(a * (b + c), a * b + a * c);
                }

                #[test]
                fn mul_zero(a in $strategy) {
                    let z = <_ as Field>::zero();
                    prop_assert_eq!(a * z, z);
                }

                #[test]
                fn double_is_add_self(a in $strategy) {
                    prop_assert_eq!(a.double(), a + a);
                }

                #[test]
                fn square_is_mul_self(a in $strategy) {
                    prop_assert_eq!(a.square(), a * a);
                }

                #[test]
                fn neg_neg_is_identity(a in $strategy) {
                    prop_assert_eq!(-(-a), a);
                }
            }
        }
    };
}

field_axiom_tests!(babybear_axioms, babybear_strategy());
field_axiom_tests!(mersenne31_axioms, mersenne31_strategy());
field_axiom_tests!(koalabear_axioms, koalabear_strategy());
field_axiom_tests!(goldilocks_axioms, goldilocks_strategy());
field_axiom_tests!(binary8_axioms, binary8_strategy());
