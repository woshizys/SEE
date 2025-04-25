use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;

pub type DefaultHasher = std::collections::hash_map::RandomState;

/// Struct used to hold a reference to a key.
#[derive(Clone)]
pub struct KeyRef<K> {
    pub(crate) k: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) { unsafe { (*self.k).hash(state) } }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool { unsafe { (*self.k).eq(&*other.k) } }
}

impl<K: Eq> Eq for KeyRef<K> {}

impl<K> Borrow<K> for KeyRef<K> {
    fn borrow(&self) -> &K { unsafe { &*self.k } }
}

impl Borrow<str> for KeyRef<String> {
    fn borrow(&self) -> &str { unsafe { &*self.k } }
}

impl<T: ?Sized> Borrow<T> for KeyRef<Box<T>> {
    fn borrow(&self) -> &T { unsafe { &*self.k } }
}

impl<T> Borrow<[T]> for KeyRef<Vec<T>> {
    fn borrow(&self) -> &[T] { unsafe { &*self.k } }
}
pub trait Cache<K, V, S = DefaultHasher>
where
    K: Hash + Eq,
{
    /// Returns the number of key-value pairs that are currently in the the cache.
    fn len(&self) -> usize;

    /// Returns the maximum number of key-value pairs the cache can hold.
    fn cap(&self) -> NonZeroUsize;

    /// Returns a bool indicating whether the cache is empty or not.
    fn is_empty(&self) -> bool;

    /// Puts a key-value pair into cache. If the key already exists in the cache, then it updates
    /// the key's value and returns the old value. Otherwise, `None` is returned.
    fn put(&mut self, k: K, v: V) -> Option<V>;

    /// Pushes a key-value pair into the cache. If an entry with key `k` already exists in
    /// the cache or another cache entry is removed (due to the capacity), then it returns
    /// the old entry's key-value pair. Otherwise, returns `None`.
    fn push(&mut self, k: K, v: V) -> Option<(K, V)>;

    /// Returns a reference to the value of the key in the cache or `None` if it is not
    /// present in the cache.
    fn get<'a, Q>(&'a mut self, k: &Q) -> Option<&'a V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Returns a mutable reference to the value of the key in the cache or `None` if it
    /// is not present in the cache.
    fn get_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Returns a reference to the value of the key in the cache if it is
    /// present in the cache.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a reference is returned.
    fn get_or_insert<F>(&'_ mut self, k: K, f: F) -> &'_ V
    where
        F: FnOnce() -> V;

    /// Returns a mutable reference to the value of the key in the cache if it is
    /// present in the cache.
    /// If the key does not exist the provided `FnOnce` is used to populate
    /// the list and a mutable reference is returned.
    fn get_or_insert_mut<F>(&'_ mut self, k: K, f: F) -> &'_ mut V
    where
        F: FnOnce() -> V;

    /// Returns a reference to the value corresponding to the key in the cache or `None` if it is
    /// not present in the cache. Unlike `get`, `peek` does not update the Cache list so the key's
    /// position will be unchanged.
    fn peek<'a, Q>(&'a mut self, k: &Q) -> Option<&'a V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Returns a mutable reference to the value corresponding to the key in the cache or `None`
    /// if it is not present in the cache. Unlike `get_mut`, `peek_mut` does not update the Cache
    /// list so the key's position will be unchanged.
    fn peek_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Returns the value corresponding to the least recently used item or `None` if the
    /// cache is empty. Like `peek`, `peek_last` does not update the Cache list so the item's
    /// position will be unchanged.
    fn peek_last(&'_ mut self) -> Option<(&'_ K, &'_ V)>;

    /// Returns a bool indicating whether the given key is in the cache. Does not update the
    /// Cache.
    fn contains<Q>(&self, k: &Q) -> bool
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Removes and returns the value corresponding to the key from the cache or
    /// `None` if it does not exist.
    fn pop<Q>(&mut self, k: &Q) -> Option<V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Removes and returns the key and the value corresponding to the key from the cache or
    /// `None` if it does not exist.
    fn pop_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Removes and returns the key and value corresponding to the least recently
    /// used item or `None` if the cache is empty.
    fn pop_last(&mut self) -> Option<(K, V)>;

    /// Marks the key as the last eliminated one.
    fn promote<Q>(&mut self, k: &Q)
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Marks the key as the first eliminated one.
    fn demote<Q>(&mut self, k: &Q)
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized;

    /// Resizes the cache. If the new capacity is smaller than the size of the current
    /// cache any entries past the new capacity are discarded.
    fn resize(&mut self, cap: NonZeroUsize);

    /// Clears the contents of the cache.
    fn clear(&mut self);
}
