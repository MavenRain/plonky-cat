#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use std::marker::PhantomData;
use plonky_cat_field::Field;

pub trait Hasher {
    type F: Field;

    fn hash_pair(left: Self::F, right: Self::F) -> Self::F;
}

/// Algebraic hash for v0.1 structural testing.  NOT cryptographically secure.
/// Uses x^5 permutation which is invertible over fields where gcd(5, p-1) = 1.
pub struct AlgebraicHash<F> {
    _marker: PhantomData<F>,
}

impl<F: Field> Hasher for AlgebraicHash<F> {
    type F = F;

    fn hash_pair(left: F, right: F) -> F {
        let sum = left + right;
        sum.pow(5) + left * right
    }
}
