#![forbid(unsafe_code)]

mod error;
pub use self::error::Error;

use plonky_cat_field::Field;
use plonky_cat_hash::Hasher;

pub struct MerkleTree<H: Hasher> {
    layers: Vec<Vec<H::F>>,
}

impl<H: Hasher> Clone for MerkleTree<H> {
    fn clone(&self) -> Self {
        Self { layers: self.layers.clone() }
    }
}

impl<H: Hasher> std::fmt::Debug for MerkleTree<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MerkleTree")
            .field("depth", &self.layers.len().saturating_sub(1))
            .field("num_leaves", &self.layers.first().map_or(0, Vec::len))
            .finish()
    }
}

impl<H: Hasher> MerkleTree<H> {
    pub fn build(leaves: Vec<H::F>) -> Result<Self, Error> {
        match () {
            () if leaves.is_empty() => Err(Error::EmptyLeaves),
            () if !leaves.len().is_power_of_two() =>
                Err(Error::NotPowerOfTwo { len: leaves.len() }),
            () => {
                let layers: Vec<Vec<H::F>> = std::iter::successors(Some(leaves), |prev| {
                    if prev.len() <= 1 {
                        None
                    } else {
                        Some(
                            prev.chunks(2)
                                .map(|pair| H::hash_pair(pair[0], pair[1]))
                                .collect(),
                        )
                    }
                })
                .collect();

                Ok(Self { layers })
            }
        }
    }

    pub fn root(&self) -> Result<H::F, Error> {
        self.layers
            .last()
            .and_then(|layer| layer.first().copied())
            .ok_or(Error::EmptyTree)
    }

    #[must_use]
    pub fn num_leaves(&self) -> usize {
        self.layers.first().map_or(0, Vec::len)
    }

    #[must_use]
    pub fn depth(&self) -> usize {
        self.layers.len().saturating_sub(1)
    }

    pub fn leaf(&self, index: usize) -> Result<H::F, Error> {
        self.layers
            .first()
            .and_then(|leaves| leaves.get(index).copied())
            .ok_or(Error::LeafIndexOutOfBounds {
                index,
                num_leaves: self.num_leaves(),
            })
    }

    pub fn auth_path(&self, index: usize) -> Result<AuthPath<H::F>, Error> {
        if index >= self.num_leaves() {
            Err(Error::LeafIndexOutOfBounds {
                index,
                num_leaves: self.num_leaves(),
            })
        } else {
            let siblings = (0..self.depth())
                .map(|d| {
                    let sibling_idx = (index >> d) ^ 1;
                    self.layers[d][sibling_idx]
                })
                .collect();

            Ok(AuthPath::new(siblings, index))
        }
    }
}

pub struct AuthPath<F> {
    siblings: Vec<F>,
    leaf_index: usize,
}

impl<F: Field> AuthPath<F> {
    #[must_use]
    pub fn new(siblings: Vec<F>, leaf_index: usize) -> Self {
        Self { siblings, leaf_index }
    }

    #[must_use]
    pub fn siblings(&self) -> &[F] {
        &self.siblings
    }

    #[must_use]
    pub fn leaf_index(&self) -> usize {
        self.leaf_index
    }

    #[must_use]
    pub fn verify<H: Hasher<F = F>>(&self, leaf: F, root: F) -> bool {
        let computed = self.siblings.iter().enumerate().fold(leaf, |cur, (d, sib)| {
            if (self.leaf_index >> d) & 1 == 0 {
                H::hash_pair(cur, *sib)
            } else {
                H::hash_pair(*sib, cur)
            }
        });
        computed == root
    }

    #[must_use]
    pub fn into_siblings(self) -> Vec<F> {
        self.siblings
    }
}
