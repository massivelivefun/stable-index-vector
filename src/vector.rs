use crate::{ID, handle::Handle, metadata::Metadata};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Vector<T> {
    /// The vector holding the actual objects.
    pub data: Vec<T>,
    /// The vector holding the associated metadata. It is accessed using the
    /// same index as for the data vector.
    pub metadata: Vec<Metadata>,
    /// The vector that stores the data index for each ID.
    pub indices: Vec<ID>,
}

/// A vector that provides stable IDs when adding objects.
/// These ID will still allow to access their associated objects even after
/// inserting or removing other objects.
/// This comes at the cost of a small overhead because of an addition indirection.
impl<T> Vector<T> {
    /// Copies the provided object at the end of the vector
    ///
    /// @param object The object to copy
    /// @return The ID to retrieve the object
    pub fn push(&mut self, object: T) -> ID {
        let id = self.get_free_slot();
        self.data.push(object);
        id
    }

    /// Removes the object from the vector
    ///
    /// @param id The ID of the object to remove
    pub fn erase_by_id(&mut self, id: ID) {
        let data_id = self.indices[id];
        let last_data_id = self.data.len() - 1;
        let last_id = self.metadata[last_data_id].reverse_id;

        self.metadata[data_id].validity_id += 1;
        self.data.swap(data_id, last_data_id);
        self.metadata.swap(data_id, last_data_id);
        self.indices.swap(id, last_id);
        self.data.pop();
    }

    /// Removes the object from the vector
    ///
    /// @param index The index in the data vector of the object to remove
    pub fn erase_by_data(&mut self, index: usize) {
        self.erase_by_id(self.metadata[index].reverse_id);
    }

    /// Removes the object referenced by the handle from the vector
    ///
    /// @param handle The handle referencing the object to remove
    pub fn erase_by_handle(&mut self, handle: &Handle<T>) {
        self.erase_by_id(handle.get_id());
    }

    /// Return the index in the data vector of the object referenced by the
    /// provided ID
    ///
    /// @param id The ID to find the data index of
    /// @return The index in the data vector assoicated with the ID
    #[must_use]
    pub fn get_data_index(&self, id: ID) -> usize {
        self.indices[id]
    }

    /// Return the number of objects in the vector
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Tells if the vector is currently empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return the vector's capacity (i.e. the number of allocated slots in
    /// the vector)
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Creates a handle pointing to the provided ID
    ///
    /// @param id The ID of the object
    /// @return A handle to the object
    pub fn create_handle(&self, id: ID) -> Option<Handle<T>> {
        if id >= self.indices.len() {
            return None;
        }
        let data_index = self.get_data_index(id);
        if data_index >= self.data.len() {
            return None;
        }
        Some(Handle {
            id,
            validity_id: self.metadata[data_index].validity_id,
            _marker: PhantomData,
        })
    }

    /// Creates a handle to an object using its position in the data vector
    ///
    /// @param index The index of the object in the data vector
    /// @return A handle to the object
    pub fn create_handle_from_data(&self, index: usize) -> Option<Handle<T>> {
        // Ensure the object is valid. If the data index is greater than the
        // current size it means that it has been swapped and removed.
        if index >= self.data.len() {
            return None;
        }
        Some(Handle {
            id: self.metadata[index].reverse_id,
            validity_id: self.metadata[index].validity_id,
            _marker: PhantomData,
        })
    }

    /// Checks if the provided object is still valid considering its last
    /// known validity ID.
    ///
    /// @param id The ID of the object
    /// @param validity_id The last known validity ID
    /// @return True if the last knownvlidity ID is equal to the current one
    #[must_use]
    pub fn is_valid(&self, id: ID, validity_id: ID) -> bool {
        validity_id == self.metadata[self.indices[id]].validity_id
    }

    /// Returns an iterator over immutable references to the elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Returns an iterator over mutable references to the elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Pre allocates @p size slots in the vector
    /// @param size The number of slots to allocate in the vector
    pub fn reserve(&mut self, size: usize) {
        self.data.reserve(size);
        self.metadata.reserve(size);
        self.indices.reserve(size);
    }

    /// Return the validity ID associated with the provided ID
    pub fn get_validity_id(&self, id: ID) -> ID {
        self.metadata[self.indices[id]].validity_id
    }

    /// Returns a raw pointer to the first element of the data vector
    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    /// Returns a constant reference to the data vector
    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    /// Return a reference to the data vector
    pub fn get_data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    /// Returns the ID that would be used if an object was added
    #[must_use]
    pub fn get_next_id(&self) -> ID {
        if self.metadata.len() > self.data.len() {
            return self.metadata[self.data.len()].reverse_id;
        }
        self.data.len()
    }

    /// Erase all objects and invalidates all slots
    pub fn clear(&mut self) {
        self.data.clear();

        for md in &mut self.metadata {
            md.validity_id += 1;
        }
    }

    #[must_use]
    pub fn is_valid_id(&self, id: ID) -> bool {
        id < self.indices.len()
    }

    pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
        if handle.id >= self.indices.len() {
            return None;
        }
        let data_index = self.indices[handle.id];
        let current_validity = self.metadata[data_index].validity_id;
        if handle.validity_id != current_validity {
            return None;
        }
        Some(&self.data[data_index])
    }
    
    pub fn get_mut(&mut self, handle: &Handle<T>) -> Option<&mut T> {
        if handle.id >= self.indices.len() {
            return None;
        }
        let data_index = self.indices[handle.id];
        let current_validity = self.metadata[data_index].validity_id;
        if handle.validity_id != current_validity {
            return None;
        }
        Some(&mut self.data[data_index])
    }

    /// Creates a new slot in the vector
    ///
    /// @note If a slot is available it will be reused, if not a new one will
    /// be created.
    /// @return The ID of the newly created slot.
    fn get_free_slot(&mut self) -> ID {
        let id = self.get_free_id();
        self.indices[id] = self.data.len();
        id
    }

    /// Gets a ID to a free slot.
    ///
    /// @note If an ID is available it will be reused, if not a new one will be
    /// created.
    /// @return An ID of a free slot.
    fn get_free_id(&mut self) -> ID {
        // This means that we have available slots
        if self.metadata.len() > self.data.len() {
            // Update the validity ID
            self.metadata[self.data.len()].validity_id += 1;
            return self.metadata[self.data.len()].reverse_id;
        }
        // A new slot has to be created
        let new_id = self.data.len();
        self.metadata.push(Metadata::new(new_id, 0));
        self.indices.push(new_id);
        new_id
    }
}

impl<T> Index<usize> for Vector<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let data_index = self.indices[index];
        &self.data[data_index]
    }
}

impl<T> IndexMut<usize> for Vector<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let data_index = self.indices[index];
        &mut self.data[data_index]
    }
}

impl<'a, T> IntoIterator for &'a Vector<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Vector<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<T> IntoIterator for Vector<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T> Default for Vector<T> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            metadata: Vec::new(),
            indices: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_and_get() {
        let mut vec = Vector::default();
        
        let id1 = vec.push(10);
        let id2 = vec.push(20);

        let h1 = vec.create_handle(id1).unwrap();
        let h2 = vec.create_handle(id2).unwrap();

        assert_eq!(vec.get(&h1), Some(&10));
        assert_eq!(vec.get(&h2), Some(&20));
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_handle_invalidation_after_erase() {
        let mut vec = Vector::default();
        let id = vec.push(100);
        let handle = vec.create_handle(id).unwrap();

        assert!(vec.get(&handle).is_some());

        vec.erase_by_handle(&handle);

        assert_eq!(vec.get(&handle), None);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_reuse_slots_and_stale_handles() {
        let mut vec = Vector::default();

        let id_a = vec.push(10); // 'A'
        let handle_a = vec.create_handle(id_a).unwrap();

        vec.erase_by_handle(&handle_a);

        let id_b = vec.push(20);
        let handle_b = vec.create_handle(id_b).unwrap();

        assert_eq!(vec.get(&handle_a), None, "Old handle accessed new data!");
        
        assert_eq!(vec.get(&handle_b), Some(&20));
    }

    #[test]
    fn test_swap_behavior() {
        let mut vec = Vector::default();

        let id1 = vec.push(1);
        let id2 = vec.push(2);
        let id3 = vec.push(3);

        let h1 = vec.create_handle(id1).unwrap();
        let h2 = vec.create_handle(id2).unwrap();
        let h3 = vec.create_handle(id3).unwrap();

        vec.erase_by_handle(&h1);
        assert_eq!(vec.len(), 2);

        assert_eq!(vec.get(&h1), None);
        assert_eq!(vec.get(&h2), Some(&2));
        assert_eq!(vec.get(&h3), Some(&3));
    }

    #[test]
    fn test_mutable_access() {
        let mut vec = Vector::default();
        let id = vec.push(5);
        let handle = vec.create_handle(id).unwrap();

        if let Some(val) = vec.get_mut(&handle) {
            *val = 10;
        }

        assert_eq!(vec.get(&handle), Some(&10));
    }

    #[test]
    fn test_clear() {
        let mut vec = Vector::default();
        let id = vec.push(1);
        let handle = vec.create_handle(id).unwrap();

        vec.clear();

        assert!(vec.is_empty());
        assert_eq!(vec.get(&handle), None);
    }

    #[test]
    fn test_push_and_access() {
        let mut vec = Vector::default();
        let id = vec.push(42);
        let handle = vec.create_handle(id).unwrap();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(&handle), Some(&42));
    }

    #[test]
    fn test_erase_logic() {
        let mut vec = Vector::default();
        let id_a = vec.push(10);
        let id_b = vec.push(20);
        let id_c = vec.push(30);

        let h_a = vec.create_handle(id_a).unwrap();
        let h_b = vec.create_handle(id_b).unwrap();
        let h_c = vec.create_handle(id_c).unwrap();

        vec.erase_by_handle(&h_a);

        assert_eq!(vec.len(), 2);

        assert_eq!(vec.get(&h_a), None);
        assert_eq!(vec.get(&h_b), Some(&20));
        assert_eq!(vec.get(&h_c), Some(&30));
    }

    #[test]
    fn test_stale_handle_protection() {
        let mut vec = Vector::default();

        let id = vec.push(100);
        let handle_old = vec.create_handle(id).unwrap();

        vec.erase_by_handle(&handle_old);

        let id_new = vec.push(200);
        let handle_new = vec.create_handle(id_new).unwrap();

        assert_eq!(vec.get(&handle_old), None);
        assert_eq!(vec.get(&handle_new), Some(&200));
    }

    #[test]
    fn test_clear_invalidates_handles() {
        let mut vec = Vector::default();
        let id = vec.push(1);
        let handle = vec.create_handle(id).unwrap();

        vec.clear();

        assert!(vec.is_empty());
        assert_eq!(vec.get(&handle), None);
    }

    #[test]
    fn test_get_mut() {
        let mut vec = Vector::default();
        let id = vec.push(5);
        let handle = vec.create_handle(id).unwrap();

        if let Some(val) = vec.get_mut(&handle) {
            *val = 99;
        }

        assert_eq!(vec.get(&handle), Some(&99));
    }
    
    #[test]
    fn test_invalid_handle_creation() {
        let vec: Vector<i32> = Vector::default();
        let result = vec.create_handle(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_iterators() {
        let mut vec = Vector::default();
        vec.push(10);
        vec.push(20);
        vec.push(30);

        let mut sum = 0;
        for x in &vec {
            sum += *x;
        }
        assert_eq!(sum, 60);

        for x in &mut vec {
            *x *= 2;
        }

        let first = vec.iter().next();
        assert_eq!(first, Some(&20));

        let mut collected = Vec::new();
        for x in vec {
            collected.push(x);
        }
        
        assert_eq!(collected, vec![20, 40, 60]);
    }
}
