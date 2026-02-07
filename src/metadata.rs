use crate::ID;

/// The struct holding additional information about an object.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Metadata {
    /// The reverse ID, allowing the retrieve the ID of the object from the
    /// data vector.
    pub reverse_id: ID,
    /// An identifier that is changed when the object is erased, used to
    /// ensure a handle is still valid.
    pub validity_id: ID,
}

impl Metadata {
    // Factory constructor
    pub fn new(reverse_id: ID, validity_id: ID) -> Self {
        Self {
            reverse_id,
            validity_id,
        }
    }
}

/// Default constructor
impl Default for Metadata {
    fn default() -> Self {
        Self {
            reverse_id: 0,
            validity_id: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_creation() {
        let meta = Metadata::new(42, 999);
        
        assert_eq!(meta.reverse_id, 42);
        assert_eq!(meta.validity_id, 999);
    }

    #[test]
    fn test_metadata_default() {
        let meta = Metadata::default();
        
        assert_eq!(meta.reverse_id, 0);
        assert_eq!(meta.validity_id, 0);
    }

    #[test]
    fn test_metadata_traits() {
        let m1 = Metadata::new(1, 1);
        
        let m2 = m1; 
        assert_eq!(m1.reverse_id, m2.reverse_id);
        
        assert_eq!(m1, m2);
        
        let m3 = m1.clone();
        assert_eq!(m1, m3);

        let debug_str = format!("{:?}", m1);
        assert!(debug_str.contains("reverse_id"));
    }
}
