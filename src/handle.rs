use crate::ID;
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Handle<T> {
    /// The ID of the object.
    pub id: ID,
    /// The validity ID of the object at the time of creation. Used to check
    /// the validity of the handle.
    pub validity_id: ID,
    /// Prevent type collisions so not just any type of Handle can be passed
    /// into any type of Vector.
    pub _marker: PhantomData<T>,
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self { *self }
}

impl<T> Handle<T> {
    /// Factory constructor
    pub fn new(id: ID, validity_id: ID) -> Self {
        Self {
            id,
            validity_id,
            _marker: PhantomData,
        }
    }

    /// Returns the ID of the associated object
    #[must_use]
    pub fn get_id(&self) -> usize {
        self.id
    }
}

// Default factory constructor
impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self {   
            id: 0,
            validity_id: 0,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
