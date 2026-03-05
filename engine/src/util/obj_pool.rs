use std::marker::PhantomData;

// ----------------------------------------------------------------------------
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ObjId<T> {
    index: usize,
    epoch: u32,
    _marker: PhantomData<T>,
}

// ----------------------------------------------------------------------------
impl<T> Default for ObjId<T> {
    fn default() -> Self {
        Self {
            index: 0,
            epoch: 0,
            _marker: PhantomData,
        }
    }
}

// ----------------------------------------------------------------------------
impl<T> Copy for ObjId<T> {}

// ----------------------------------------------------------------------------
impl<T> Clone for ObjId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// ----------------------------------------------------------------------------
struct ObjSlot<T> {
    value: Option<T>,
    epoch: u32,
}

// ----------------------------------------------------------------------------
impl<T> Default for ObjSlot<T> {
    fn default() -> Self {
        Self {
            value: None,
            epoch: 0,
        }
    }
}

// ----------------------------------------------------------------------------
#[derive(Default)]
pub struct ObjPool<T> {
    pool: Vec<ObjSlot<T>>,
    free: Vec<usize>,
}

// ----------------------------------------------------------------------------
impl<T> ObjPool<T> {
    // ------------------------------------------------------------------------
    pub fn new() -> Self {
        Self {
            pool: Vec::new(),
            free: Vec::new(),
        }
    }

    // ------------------------------------------------------------------------
    pub fn is_empty(&self) -> bool {
        self.pool.len() == self.free.len()
    }

    // ------------------------------------------------------------------------
    pub fn insert(&mut self, value: T) -> ObjId<T> {
        let index = if let Some(i) = self.free.pop() {
            i
        } else {
            self.pool.push(ObjSlot::<T>::default());
            self.pool.len() - 1
        };

        let slot = &mut self.pool[index];
        slot.value = Some(value);

        ObjId {
            index,
            epoch: slot.epoch,
            _marker: PhantomData,
        }
    }

    // ------------------------------------------------------------------------
    pub fn remove(&mut self, key: ObjId<T>) -> Option<T> {
        let slot = self.pool.get_mut(key.index)?;

        if slot.epoch != key.epoch {
            return None;
        }

        let value = slot.value.take()?;

        slot.epoch = slot.epoch.wrapping_add(1);
        self.free.push(key.index);

        Some(value)
    }

    // ------------------------------------------------------------------------
    pub fn contains(&self, key: ObjId<T>) -> bool {
        self.get(key).is_some()
    }

    // ------------------------------------------------------------------------
    pub fn get(&self, key: ObjId<T>) -> Option<&T> {
        let slot = self.pool.get(key.index)?;
        if slot.epoch != key.epoch {
            return None;
        }
        slot.value.as_ref()
    }

    // ------------------------------------------------------------------------
    pub fn get_mut(&mut self, key: ObjId<T>) -> Option<&mut T> {
        let slot = self.pool.get_mut(key.index)?;
        if slot.epoch != key.epoch {
            return None;
        }
        slot.value.as_mut()
    }

    // ------------------------------------------------------------------------
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.pool.iter().filter_map(|s| s.value.as_ref())
    }

    // ------------------------------------------------------------------------
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.pool.iter_mut().filter_map(|s| s.value.as_mut())
    }
}

// ----------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------------
    #[test]
    fn basic_insert_get_remove() {
        let mut pool = ObjPool::default();
        assert!(pool.is_empty());

        let key = pool.insert(42);
        assert!(!pool.is_empty());

        assert_eq!(pool.get(key), Some(&42));

        *pool.get_mut(key).unwrap() = 10;
        let value = pool.remove(key);

        assert_eq!(value, Some(10));
        assert!(pool.get(key).is_none());
        assert!(pool.remove(key).is_none());
        assert!(pool.is_empty());

        let new = pool.insert(100);
        assert_eq!(new.index, key.index);
        assert_ne!(new.epoch, key.epoch);

        assert_eq!(pool.get(new), Some(&100));
        assert_eq!(pool.get(key), None);
    }

    // ------------------------------------------------------------------------
    #[test]
    fn multiple_insertions() {
        let mut pool = ObjPool::default();

        let a = pool.insert(1);
        let b = pool.insert(2);
        let c = pool.insert(3);

        assert_eq!(pool.get(a), Some(&1));
        assert_eq!(pool.get(b), Some(&2));
        assert_eq!(pool.get(c), Some(&3));

        pool.remove(a);

        let d = pool.insert(4);

        assert_eq!(pool.get(a), None);
        assert_eq!(pool.get(b), Some(&2));
        assert_eq!(pool.get(c), Some(&3));
        assert_eq!(pool.get(d), Some(&4));
    }
}
