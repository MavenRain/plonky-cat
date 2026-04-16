#[derive(Debug)]
pub enum Error {
    EmptyLeaves,
    NotPowerOfTwo { len: usize },
    LeafIndexOutOfBounds { index: usize, num_leaves: usize },
    EmptyTree,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyLeaves => write!(f, "cannot build Merkle tree from empty leaves"),
            Self::NotPowerOfTwo { len } =>
                write!(f, "leaf count {len} is not a power of two"),
            Self::LeafIndexOutOfBounds { index, num_leaves } =>
                write!(f, "leaf index {index} out of bounds for {num_leaves} leaves"),
            Self::EmptyTree =>
                write!(f, "tree has no layers"),
        }
    }
}

impl std::error::Error for Error {}
