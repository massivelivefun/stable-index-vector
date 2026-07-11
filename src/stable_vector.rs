use crate::{
    handle::Handle,
    index_type::IndexType,
    metadata::Metadata
};
use std::array;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StableVec<T, I: IndexType = u32> {
    /// The vector holding the actual objects.
    pub data: Vec<T>,
    /// The vector holding the associated metadata. It is accessed using the
    /// same index as for the data vector.
    pub metadata: Vec<Metadata<I>>,
    /// The vector that stores the data index for each ID.
    pub indices: Vec<I>,
}

#[derive(Debug)]
pub enum StableVecError {
    CapacityExceeded,
}

/// A vector that provides stable IDs when adding objects.
/// These ID will still allow to access their associated objects even after
/// inserting or removing other objects.
/// This comes at the cost of a small overhead because of an addition
/// indirection.
impl<T, I: IndexType> StableVec<T, I> {
    /// Creates a new, empty vector. 
    /// If no index type is specified, it defaults to `u32`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Copies the provided object at the end of the vector.
    ///
    /// @param `object` - The object to copy.
    /// @return - The ID to retrieve the object.
    pub fn push(&mut self, object: T) -> I {
        self.try_push(object).expect("StableVec capacity exceeded")
    }

    pub fn try_push(&mut self, object: T) -> Result<I, StableVecError> {
        if self.data.len() >= I::max().to_usize() {
            return Err(StableVecError::CapacityExceeded);
        }
        // This is safe because we check if the number of elements is always
        // less than the I::max() before hand so we can then push without
        // breaking the StableVec's state
        unsafe {
            Ok(self.push_unchecked(object))
        }
    }

    /// Safety:
    /// The caller must guarantee that the current number of elements
    /// is strictly less than `I::max()`. If this invariant is violated,
    /// the ID will truncate, leading to silent data corruption.
    #[inline]
    pub unsafe fn push_unchecked(&mut self, object: T) -> I {
        let id = self.get_free_slot();
        self.data.push(object);
        id
    }

    /// Removes the object from the vector.
    ///
    /// @param `id` - The ID of the object to remove.
    pub fn erase_by_id(&mut self, id: I) {
        let data_id = self.indices[id.to_usize()].to_usize();
        let last_data_id = self.data.len() - 1;
        let last_id = self.metadata[last_data_id].reverse_id.to_usize();

        self.metadata[data_id].validity_id =
            self.metadata[data_id].validity_id.add(I::one());
        self.data.swap(data_id, last_data_id);
        self.metadata.swap(data_id, last_data_id);
        self.indices.swap(id.to_usize(), last_id);
        self.data.pop();
    }

    /// Removes the object from the vector.
    ///
    /// @param `index` - The index in the data vector of the object to remove.
    pub fn erase_by_data(&mut self, index: usize) {
        self.erase_by_id(self.metadata[index].reverse_id);
    }

    /// Removes the object referenced by the handle from the vector.
    ///
    /// @param `handle` - The handle referencing the object to remove
    pub fn erase_by_handle(&mut self, handle: &Handle<T, I>) {
        self.erase_by_id(handle.get_id());
    }

    /// Return the index in the data vector of the object referenced by the
    /// provided ID.
    ///
    /// @param `id` - The ID to find the data index of.
    /// @return - The index in the data vector assoicated with the ID.
    #[must_use]
    pub fn get_data_index(&self, id: I) -> usize {
        self.indices[id.to_usize()].to_usize()
    }

    /// Return the number of objects in the vector.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns True if the vector is currently empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return the vector's capacity (i.e. the number of allocated slots in
    /// the vector).
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Creates a handle pointing to the provided ID.
    ///
    /// @param `id` - The ID of the object.
    /// @return - A handle to the object.
    pub fn create_handle(&self, id: I) -> Option<Handle<T, I>> {
        if id.to_usize() >= self.indices.len() {
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

    /// Creates a handle to an object using its position in the data vector.
    ///
    /// @param `index` - The index of the object in the data vector.
    /// @return - A handle to the object.
    pub fn create_handle_from_data(&self, index: usize)
        -> Option<Handle<T, I>>
    {
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

    /// Returns the underlying data as a contiguous slice.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns the underlying data as a contiguous mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Checks if the provided object is still valid considering its last
    /// known validity ID.
    ///
    /// @param `id` - The ID of the object.
    /// @param `validity_id` - The last known validity ID.
    /// @return - True if the last known validity ID is equal to the current
    /// one.
    #[must_use]
    pub fn is_valid(&self, id: I, validity_id: I) -> bool {
        validity_id ==
            self.metadata[self.indices[id.to_usize()].to_usize()].validity_id
    }

    /// Returns an iterator over immutable references to the elements.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Returns an iterator over mutable references to the elements.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Pre allocates @p size slots in the vector.
    /// @param size - The number of slots to allocate in the vector.
    pub fn reserve(&mut self, size: usize) {
        self.data.reserve(size);
        self.metadata.reserve(size);
        self.indices.reserve(size);
    }

    /// Return the validity ID associated with the provided ID.
    pub fn get_validity_id(&self, id: I) -> I {
        self.metadata[self.indices[id.to_usize()].to_usize()].validity_id
    }

    /// Returns an optional to the first element of the data vector.
    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    /// Consumes the Vector to return the underlying data.
    pub fn data(self) -> Vec<T> {
        self.data
    }

    /// Returns a constant reference to the data vector.
    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }

    /// Return a reference to the data vector.
    pub fn get_data_mut(&mut self) -> &mut Vec<T> {
        &mut self.data
    }

    /// Returns the ID that would be used if an object was added.
    #[must_use]
    pub fn get_next_id(&self) -> I {
        if self.metadata.len() > self.data.len() {
            return self.metadata[self.data.len()].reverse_id;
        }
        I::from_usize(self.data.len())
    }

    /// Erase all objects and invalidates all slots.
    pub fn clear(&mut self) {
        self.data.clear();

        for md in &mut self.metadata {
            md.validity_id = md.validity_id.add(I::one());
        }
    }

    #[must_use]
    pub fn is_valid_id(&self, id: I) -> bool {
        id.to_usize() < self.indices.len()
    }

    /// Make sure the Handles originate from the callee vector.
    pub fn get(&self, handle: &Handle<T, I>) -> Option<&T> {
        if handle.id.to_usize() >= self.indices.len() {
            return None;
        }
        let data_index = self.indices[handle.id.to_usize()].to_usize();
        let current_validity = self.metadata[data_index].validity_id;
        if handle.validity_id != current_validity {
            return None;
        }
        Some(&self.data[data_index])
    }
    
    /// Make sure the Handles originate from the callee vector.
    pub fn get_mut(&mut self, handle: &Handle<T, I>) -> Option<&mut T> {
        if handle.id.to_usize() >= self.indices.len() {
            return None;
        }
        let data_index = self.indices[handle.id.to_usize()].to_usize();
        let current_validity = self.metadata[data_index].validity_id;
        if handle.validity_id != current_validity {
            return None;
        }
        Some(&mut self.data[data_index])
    }

    /// Safely retrieves mutable references to a known number of distinct
    /// objects simultaneously.
    /// Make sure the Handles originate from the callee vector.
    /// Returns None if any handle is invalid, out of bounds, or if there
    /// are duplicates.
    pub fn get_many_mut<const N: usize>(
        &mut self,
        handles: [&Handle<T, I>; N],
    ) -> Option<[&mut T; N]> {
        let mut indices = [0; N];
        for i in 0..N {
            let h = handles[i];
            
            if h.id.to_usize() >= self.indices.len() {
                return None;
            }
            
            let data_idx = self.indices[h.id.to_usize()].to_usize();
            
            if data_idx >= self.data.len() {
                return None;
            }
            
            if h.validity_id != self.metadata[data_idx].validity_id {
                return None;
            }
            
            indices[i] = data_idx;
        }

        for i in 0..N {
            for j in (i + 1)..N {
                if indices[i] == indices[j] {
                    return None;
                }
            }
        }

        // SAFETY: 
        // - We verified `data_idx < self.data.len()` for all indices.
        // - We verified all indices are strictly unique, guaranteeing no
        //   mutable aliasing.
        // - The lifetime of the returned references is bound to `&mut self`.
        let ptr = self.data.as_mut_ptr();
        
        let refs = array::from_fn(|i| {
            let data_idx = indices[i];
            unsafe { &mut *ptr.add(data_idx) }
        });

        Some(refs)
    }

    /// Retrieves a reference without bounds checking or verifying validity
    /// IDs.
    /// 
    /// Safety:
    /// The caller must guarantee that the handle is currently valid.
    /// The caller must guarantee that the handle originates from the vector.
    pub unsafe fn get_unchecked(&self, handle: &Handle<T, I>) -> &T {
        unsafe {
            let data_index =
                (*self.indices.get_unchecked(handle.id.to_usize()))
                    .to_usize();
            self.data.get_unchecked(data_index)
        }
    }

    /// Retrieves a reference without bounds checking or verifying validity
    /// IDs.
    /// 
    /// Safety:
    /// The caller must guarantee that the handle is currently valid.
    /// The caller must guarantee that the handle originates from the vector.
    pub unsafe fn get_unchecked_mut(&mut self, handle: &Handle<T, I>)
        -> &mut T
    {
        unsafe { 
            let data_index =
                (*self.indices.get_unchecked(handle.id.to_usize()))
                    .to_usize();
            self.data.get_unchecked_mut(data_index)
        }
    }

    /// Creates a new slot in the vector.
    ///
    /// @note If a slot is available it will be reused, if not a new one will
    /// be created.
    /// @return - The ID of the newly created slot.
    fn get_free_slot(&mut self) -> I {
        let id = self.get_free_id();
        self.indices[id.to_usize()] = I::from_usize(self.data.len());
        id
    }

    /// Gets a ID to a free slot.
    ///
    /// @note - If an ID is available it will be reused, if not a new one will
    /// be created.
    /// @return - An ID of a free slot.
    fn get_free_id(&mut self) -> I {
        if self.data.len() >= I::max().to_usize() {
            panic!("StableVec capacity exceeded for IndexType!");
        }
        // This means that we have available slots
        if self.metadata.len() > self.data.len() {
            // Update the validity ID
            self.metadata[self.data.len()].validity_id =
                self.metadata[self.data.len()].validity_id.add(I::one());
            return self.metadata[self.data.len()].reverse_id;
        }
        // A new slot has to be created
        let new_id = I::from_usize(self.data.len());
        self.metadata.push(Metadata::new(new_id, I::zero()));
        self.indices.push(new_id);
        new_id
    }

    /// Retains only the elements specified by the predicate.
    /// Passes the Handle and a mutable reference to the element.
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&Handle<T, I>, &mut T) -> bool,
    {
        let mut i = self.data.len();
        
        // Iterate backwards so that swap-and-pop doesn't shift
        // un-evaluated elements into our current index.
        while i > 0 {
            i -= 1;
            
            let handle = Handle {
                id: self.metadata[i].reverse_id,
                validity_id: self.metadata[i].validity_id,
                _marker: PhantomData,
            };

            let keep = f(&handle, &mut self.data[i]);

            if !keep {
                self.erase_by_data(i);
            }
        }
    }
}

impl<T, I: IndexType> Index<usize> for StableVec<T, I> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let data_index = self.indices[index].to_usize();
        &self.data[data_index]
    }
}

impl<T, I: IndexType> IndexMut<usize> for StableVec<T, I> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let data_index = self.indices[index].to_usize();
        &mut self.data[data_index]
    }
}

impl<'a, T, I: IndexType> IntoIterator for &'a StableVec<T, I> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T, I: IndexType> IntoIterator for &'a mut StableVec<T, I> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

impl<T, I: IndexType> IntoIterator for StableVec<T, I> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T, I: IndexType> Default for StableVec<T, I> {
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
    use std::mem::size_of;

    #[test]
    fn test_basic_push_and_get() {
        let mut vec: StableVec<i32> = StableVec::new();
        
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
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(100);
        let handle = vec.create_handle(id).unwrap();

        assert!(vec.get(&handle).is_some());

        vec.erase_by_handle(&handle);

        assert_eq!(vec.get(&handle), None);
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn test_reuse_slots_and_stale_handles() {
        let mut vec: StableVec<i32> = StableVec::new();

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
        let mut vec: StableVec<i32> = StableVec::new();

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
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(5);
        let handle = vec.create_handle(id).unwrap();

        if let Some(val) = vec.get_mut(&handle) {
            *val = 10;
        }

        assert_eq!(vec.get(&handle), Some(&10));
    }

    #[test]
    fn test_clear() {
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(1);
        let handle = vec.create_handle(id).unwrap();

        vec.clear();

        assert!(vec.is_empty());
        assert_eq!(vec.get(&handle), None);
    }

    #[test]
    fn test_push_and_access() {
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(42);
        let handle = vec.create_handle(id).unwrap();

        assert_eq!(vec.len(), 1);
        assert_eq!(vec.get(&handle), Some(&42));
    }

    #[test]
    fn test_erase_logic() {
        let mut vec: StableVec<i32> = StableVec::new();
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
        let mut vec: StableVec<i32> = StableVec::new();

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
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(1);
        let handle = vec.create_handle(id).unwrap();

        vec.clear();

        assert!(vec.is_empty());
        assert_eq!(vec.get(&handle), None);
    }

    #[test]
    fn test_get_mut() {
        let mut vec: StableVec<i32> = StableVec::new();
        let id = vec.push(5);
        let handle = vec.create_handle(id).unwrap();

        if let Some(val) = vec.get_mut(&handle) {
            *val = 99;
        }

        assert_eq!(vec.get(&handle), Some(&99));
    }
    
    #[test]
    fn test_invalid_handle_creation() {
        let vec: StableVec<i32> = StableVec::new();
        let result = vec.create_handle(999);
        assert!(result.is_none());
    }

    #[test]
    fn test_iterators() {
        let mut vec: StableVec<i32> = StableVec::new();
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

    #[test]
    fn test_stable_vec_layout_sizes() {
        // On a 64-bit system, a Vec is 24 bytes (pointer + len + capacity).
        // StableVec has 3 Vecs: data (Vec<T>), metadata (Vec<Metadata<I>>),
        // indices (Vec<I>).
        // 3 * 24 bytes = 72 bytes. 
        
        let size_u8 = size_of::<StableVec<i32, u8>>();
        let size_u32 = size_of::<StableVec<i32, u32>>();
        let size_u64 = size_of::<StableVec<i32, u64>>();
        
        assert_eq!(size_u8, 72);
        assert_eq!(size_u32, 72);
        assert_eq!(size_u64, 72);
    }
}
