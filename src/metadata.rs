use crate::index_type::IndexType;

/// The struct holding additional information about an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Metadata<I: IndexType> {
    /// The reverse ID, allowing the retrieve the ID of the object from the
    /// data vector.
    pub reverse_id: I,
    /// An identifier that is changed when the object is erased, used to
    /// ensure a handle is still valid.
    pub validity_id: I,
}

impl<I: IndexType> Metadata<I> {
    // Factory constructor
    pub fn new(reverse_id: I, validity_id: I) -> Self {
        Self { reverse_id, validity_id }
    }
}

/// Default constructor
impl<I: IndexType> Default for Metadata<I> {
    fn default() -> Self {
        Self { reverse_id: I::zero(), validity_id: I::zero() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = Metadata::new(42u32, 999u32);
        
        assert_eq!(meta.reverse_id, 42);
        assert_eq!(meta.validity_id, 999);
    }

    #[test]
    fn test_metadata_default() {
        let meta = Metadata::<u32>::default();
        
        assert_eq!(meta.reverse_id, 0);
        assert_eq!(meta.validity_id, 0);
    }

    #[test]
    fn test_metadata_traits() {
        let m1 = Metadata::new(1u32, 1u32);
        
        let m2 = m1; 
        assert_eq!(m1.reverse_id, m2.reverse_id);
        
        assert_eq!(m1, m2);
        
        let m3 = m1.clone();
        assert_eq!(m1, m3);

        let debug_str = format!("{:?}", m1);
        assert!(debug_str.contains("reverse_id"));
    }
}
