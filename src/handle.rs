use crate::index_type::IndexType;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Handle<T, I: IndexType = u32> {
    /// The ID of the object.
    pub id: I,
    /// The validity ID of the object at the time of creation. Used to check
    /// the validity of the handle.
    pub validity_id: I,
    /// Prevent type collisions so not just any type of Handle can be passed
    /// into any type of Vector.
    pub _marker: PhantomData<T>,
}

impl<T, I: IndexType> Copy for Handle<T, I> {}

impl<T, I: IndexType> Clone for Handle<T, I> {
    fn clone(&self) -> Self { *self }
}

impl<T, I: IndexType> Handle<T, I> {
    /// Creates a new handle using the default `u32` index type.
    pub fn new(id: I, validity_id: I) -> Self {
        Self {
            id,
            validity_id,
            _marker: PhantomData,
        }
    }

    /// Returns the ID of the associated object the Handle represents.
    #[must_use]
    pub fn get_id(&self) -> I {
        self.id
    }

    /// Returns the validity ID of the associated object the Handle represents. 
    #[must_use]
    pub fn get_validity_id(&self) -> I {
        self.validity_id
    }
}

// Default factory constructor
impl<T, I: IndexType> Default for Handle<T, I> {
    fn default() -> Self {
        Self {
            id: I::zero(),
            validity_id: I::zero(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_handle_creation() {
        let handle: Handle<isize> = Handle::new(10, 500);
        
        assert_eq!(handle.id, 10);
        assert_eq!(handle.validity_id, 500);
        assert_eq!(handle.get_id(), 10);
    }

    #[test]
    fn test_handle_default() {
        let handle: Handle<isize> = Handle::default();
        
        assert_eq!(handle.id, 0);
        assert_eq!(handle.validity_id, 0);
    }

    #[test]
    fn test_handle_equality() {
        let h1: Handle<isize> = Handle::new(1, 100);
        let h2: Handle<isize> = Handle::new(1, 100);
        let h3: Handle<isize> = Handle::new(1, 101);
        let h4: Handle<isize> = Handle::new(2, 100);

        assert_eq!(h1, h2,
            "Handles with same ID and Validity should be equal");
        assert_ne!(h1, h3,
            "Handles with different Validity should NOT be equal");
        assert_ne!(h1, h4,
            "Handles with different IDs should NOT be equal");
    }

    #[test]
    fn test_handle_copy_semantics() {
        let h1: Handle<isize> = Handle::new(5, 50);
        
        let h2 = h1; 
        
        assert_eq!(h1.id, 5);
        assert_eq!(h2.id, 5);
    }

    #[test]
    fn test_handle_hashing() {
        use std::collections::HashSet;
        
        let mut set = HashSet::new();
        let h1: Handle<isize> = Handle::new(1, 1);
        
        set.insert(h1);
        
        assert!(set.contains(&Handle::new(1, 1)));
        assert!(!set.contains(&Handle::new(1, 2)));
    }

    #[test]
    fn test_handle_layout_sizes() {
        // Handle<T, u8> -> 1 byte ID + 1 byte validity = 2 bytes
        assert_eq!(size_of::<Handle<i32, u8>>(), 2);
        
        // Handle<T, u16> -> 2 byte ID + 2 byte validity = 4 bytes
        assert_eq!(size_of::<Handle<i32, u16>>(), 4);
        
        // Handle<T, u32> -> 4 byte ID + 4 byte validity = 8 bytes
        assert_eq!(size_of::<Handle<i32, u32>>(), 8);
        
        // Handle<T, u64> -> 8 byte ID + 8 byte validity = 16 bytes
        assert_eq!(size_of::<Handle<i32, u64>>(), 16);
    }
}
