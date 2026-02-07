pub mod handle;
pub mod metadata;
pub mod vector;

pub use crate::handle::*;
pub use crate::metadata::*;
pub use crate::vector::*;

/// Alias to differentiate betweens IDs and index.
/// An ID allows to access the data through the index vector and is associated
/// with the same object until it is erased An index is simply the current
/// position of the object in the data vector and may change with deletions.
pub type ID = usize;

#[allow(unused)]
pub const INVALID_ID: usize = usize::MAX;
