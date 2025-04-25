use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::ptr::{null_mut, NonNull};
use std::{fmt, mem};

use crate::lru::cache::{self, Cache, KeyRef};

type Replace<K, V> = (Option<(K, V)>, NonNull<LRUEntry<K, V>>);

/// LRUEntry used to hold a key value pair. Also contains
/// references to previous and next entries so we can
/// maintain the entries in a linked list ordered by their use.
struct LRUEntry<K, V> {
    key: mem::MaybeUninit<K>,
    value: mem::MaybeUninit<V>,
    prev: *mut LRUEntry<K, V>,
    next: *mut LRUEntry<K, V>,
}

impl<K, V> LRUEntry<K, V> {
    fn new(key: K, val: V) -> Self {
        LRUEntry {
            key: mem::MaybeUninit::new(key),
            value: mem::MaybeUninit::new(val),
            prev: null_mut(),
            next: null_mut(),
        }
    }

    fn new_sigil() -> Self {
        LRUEntry {
            key: mem::MaybeUninit::uninit(),
            value: mem::MaybeUninit::uninit(),
            prev: null_mut(),
            next: null_mut(),
        }
    }
}

/// An iterator over the entries of a `LRUCache`.
pub struct Iter<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *const LRUEntry<K, V>,
    end: *const LRUEntry<K, V>,

    phantom_data: PhantomData<&'a K>,
}

impl<'a, K: 'a, V: 'a> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.ptr).key.as_ptr()) };
        let val = unsafe { &(*(*self.ptr).value.as_ptr()) };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key, val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.end).key.as_ptr()) };
        let val = unsafe { &(*(*self.end).value.as_ptr()) };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key, val))
    }
}

impl<K, V> ExactSizeIterator for Iter<'_, K, V> {}
impl<K, V> FusedIterator for Iter<'_, K, V> {}

impl<K, V> Clone for Iter<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            len: self.len,
            ptr: self.ptr,
            end: self.end,
            phantom_data: PhantomData,
        }
    }
}

unsafe impl<K: Send, V: Send> Send for Iter<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for Iter<'_, K, V> {}

/// An iterator over mutable entries of a `LRUCache`.
pub struct IterMut<'a, K: 'a, V: 'a> {
    len: usize,

    ptr: *mut LRUEntry<K, V>,
    end: *mut LRUEntry<K, V>,

    phantom_data: PhantomData<&'a K>,
}

impl<'a, K: 'a, V: 'a> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.ptr).key.as_ptr()) };
        let val = unsafe { &mut (*(*self.ptr).value.as_mut_ptr()) };

        self.len -= 1;
        self.ptr = unsafe { (*self.ptr).next };

        Some((key, val))
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (self.len, Some(self.len)) }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }
}

impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        }

        let key = unsafe { &(*(*self.end).key.as_ptr()) };
        let val = unsafe { &mut (*(*self.end).value.as_mut_ptr()) };

        self.len -= 1;
        self.end = unsafe { (*self.end).prev };

        Some((key, val))
    }
}

impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {}
impl<K, V> FusedIterator for IterMut<'_, K, V> {}

unsafe impl<K: Send, V: Send> Send for IterMut<'_, K, V> {}
unsafe impl<K: Sync, V: Sync> Sync for IterMut<'_, K, V> {}

/// An iterator that moves out of a `LRUCache`.
pub struct IntoIter<K, V>
where
    K: Hash + Eq,
{
    cache: LRUCache<K, V>,
}

impl<K, V> Iterator for IntoIter<K, V>
where
    K: Hash + Eq,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> { self.cache.pop_last() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.cache.len();
        (len, Some(len))
    }

    fn count(self) -> usize { self.cache.len() }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> where K: Hash + Eq {}
impl<K, V> FusedIterator for IntoIter<K, V> where K: Hash + Eq {}

/// A LRU cache.
/// This is a single level thread unsafe LRU implementation.
#[derive(Clone)]
pub struct LRUCache<K, V, S = cache::DefaultHasher> {
    // map is used to speed up LRU access.
    map: HashMap<KeyRef<K>, NonNull<LRUEntry<K, V>>, S>,
    // cap is used to specific LRU cache capacity.
    cap: NonZeroUsize,

    // head and tail are sigil nodes to facilitate inserting entries
    head: *mut LRUEntry<K, V>,
    tail: *mut LRUEntry<K, V>,
}

impl<K, V, S> LRUCache<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Creates a new LRU Cache with the given capacity.
    fn construct(cap: NonZeroUsize, map: HashMap<KeyRef<K>, NonNull<LRUEntry<K, V>>, S>) -> Self {
        let cache = LRUCache {
            map,
            cap,
            head: Box::into_raw(Box::new(LRUEntry::new_sigil())),
            tail: Box::into_raw(Box::new(LRUEntry::new_sigil())),
        };

        unsafe {
            (*cache.head).next = cache.tail;
            (*cache.tail).prev = cache.head;
        }

        cache
    }

    /// Detach specific `node`.
    fn detach(&mut self, node: *mut LRUEntry<K, V>) {
        unsafe {
            (*(*node).prev).next = (*node).next;
            (*(*node).next).prev = (*node).prev;
        }
    }

    /// Attaches `node` after the sigil `self.head` node.
    fn attach(&mut self, node: *mut LRUEntry<K, V>) {
        unsafe {
            (*node).next = (*self.head).next;
            (*node).prev = self.head;
            (*self.head).next = node;
            (*(*node).next).prev = node;
        }
    }

    fn detach_last(&mut self) -> Option<Box<LRUEntry<K, V>>> {
        let prev = unsafe { (*self.tail).prev };

        if prev != self.head {
            let old_key = KeyRef {
                k: unsafe { &(*(*prev).key.as_ptr()) },
            };
            let old_node = self.map.remove(&old_key).unwrap();

            let node_ptr: *mut LRUEntry<K, V> = old_node.as_ptr();
            self.detach(node_ptr);

            Some(unsafe { Box::from_raw(node_ptr) })
        } else {
            None
        }
    }

    fn attach_last(&mut self, node: *mut LRUEntry<K, V>) {
        unsafe {
            (*node).next = self.tail;
            (*node).prev = (*self.tail).prev;
            (*self.tail).prev = node;
            (*(*node).prev).next = node;
        }
    }

    // Used internally to swap out a node if the cache is full or to create a new node if space
    // is available. Shared between `put`, `push`, `get_or_insert`, and `get_or_insert_mut`.
    fn replace_or_create_node(&mut self, k: K, v: V) -> Replace<K, V> {
        if self.len() == self.cap().get() {
            // if the cache is full, remove the last entry so we can use it for the new key.
            let old_key = KeyRef {
                k: unsafe { &(*(*(*self.tail).prev).key.as_ptr()) },
            };

            let old_node = self.map.remove(&old_key).unwrap();

            let node_ptr: *mut LRUEntry<K, V> = old_node.as_ptr();

            // read out the node's old key and value and then replace it
            let replaced = unsafe {
                (
                    mem::replace(&mut (*node_ptr).key, mem::MaybeUninit::new(k)).assume_init(),
                    mem::replace(&mut (*node_ptr).value, mem::MaybeUninit::new(v)).assume_init(),
                )
            };

            self.detach(node_ptr);

            (Some(replaced), old_node)
        } else {
            (None, unsafe {
                NonNull::new_unchecked(Box::into_raw(Box::new(LRUEntry::new(k, v))))
            })
        }
    }

    // Used internally by `put` and `push` to add a new entry to the lru.
    // Takes ownership of and returns entries replaced due to the cache's capacity
    // when `capture` is true.
    fn capturing_put(&mut self, k: K, mut v: V, capture: bool) -> Option<(K, V)> {
        let node_ref = self.map.get_mut(&KeyRef { k: &k });

        match node_ref {
            // if the key is already in the cache just update its
            // value and move it to the front of the list
            Some(node_ref) => {
                let node_ptr: *mut LRUEntry<K, V> = (*node_ref).as_ptr();

                unsafe {
                    core::ptr::swap(&mut v, &mut (*(*node_ptr).value.as_mut_ptr()));
                }

                self.detach(node_ptr);
                self.attach(node_ptr);

                Some((k, v))
            },
            None => {
                let (replaced, node) = self.replace_or_create_node(k, v);

                let node_ptr: *mut LRUEntry<K, V> = node.as_ptr();
                self.attach(node_ptr);

                let key_ref = KeyRef {
                    k: unsafe { (*node_ptr).key.as_ptr() },
                };
                self.map.insert(key_ref, node);

                replaced.filter(|_| capture)
            },
        }
    }

    /// Creates a new LRU Cache that holds at most `cap` items and
    /// uses the provided hash builder to hash keys.
    pub fn with_hasher(cap: NonZeroUsize, hasher: S) -> Self {
        LRUCache::construct(cap, HashMap::with_capacity_and_hasher(cap.get(), hasher))
    }

    /// Creates a new LRU Cache that never automatically evicts items and
    /// uses the provided hash builder to hash keys.
    pub fn unbounded_with_hasher(hasher: S) -> Self {
        LRUCache::construct(
            NonZeroUsize::new(usize::MAX).unwrap(),
            HashMap::with_hasher(hasher),
        )
    }

    /// An iterator visiting all entries in most-recently used order. The iterator element type is
    /// `(&K, &V)`.
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            len: self.len(),
            ptr: unsafe { (*self.head).next },
            end: unsafe { (*self.tail).prev },
            phantom_data: PhantomData,
        }
    }

    /// An iterator visiting all entries in most-recently-used order, giving a mutable reference on
    /// V.  The iterator element type is `(&K, &mut V)`.
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            len: self.len(),
            ptr: unsafe { (*self.head).next },
            end: unsafe { (*self.tail).prev },
            phantom_data: PhantomData,
        }
    }
}

impl<K, V> LRUCache<K, V>
where
    K: Hash + Eq,
{
    /// Creates a new LRU Cache that holds at most `cap` items.
    pub fn new(cap: NonZeroUsize) -> Self {
        LRUCache::construct(cap, HashMap::with_capacity(cap.get()))
    }

    /// Creates a new LRU Cache that never automatically evicts items.
    pub fn unbounded() -> Self {
        LRUCache::construct(NonZeroUsize::new(usize::MAX).unwrap(), HashMap::default())
    }
}

impl<K, V, S> Cache<K, V, S> for LRUCache<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    fn len(&self) -> usize { self.map.len() }

    fn cap(&self) -> NonZeroUsize { self.cap }

    fn is_empty(&self) -> bool { self.map.len() == 0 }

    fn put(&mut self, k: K, v: V) -> Option<V> { self.capturing_put(k, v, false).map(|(_, v)| v) }

    fn push(&mut self, k: K, v: V) -> Option<(K, V)> { self.capturing_put(k, v, true) }

    fn get<'a, Q>(&'a mut self, k: &Q) -> Option<&'a V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { &(*(*node_ptr).value.as_ptr()) })
        } else {
            None
        }
    }

    fn get_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            Some(unsafe { &mut (*(*node_ptr).value.as_mut_ptr()) })
        } else {
            None
        }
    }

    fn get_or_insert<F>(&'_ mut self, k: K, f: F) -> &'_ V
    where
        F: FnOnce() -> V,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { &(*(*node_ptr).value.as_ptr()) }
        } else {
            let v = f();
            let (_, node) = self.replace_or_create_node(k, v);

            let node_ptr: *mut LRUEntry<K, V> = node.as_ptr();
            self.attach(node_ptr);

            let key_ref = KeyRef {
                k: unsafe { (*node_ptr).key.as_ptr() },
            };
            self.map.insert(key_ref, node);

            unsafe { &(*(*node_ptr).value.as_ptr()) }
        }
    }

    fn get_or_insert_mut<F>(&'_ mut self, k: K, f: F) -> &'_ mut V
    where
        F: FnOnce() -> V,
    {
        if let Some(node) = self.map.get_mut(&KeyRef { k: &k }) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();

            self.detach(node_ptr);
            self.attach(node_ptr);

            unsafe { &mut (*(*node_ptr).value.as_mut_ptr()) }
        } else {
            let v = f();
            let (_, node) = self.replace_or_create_node(k, v);

            let node_ptr: *mut LRUEntry<K, V> = node.as_ptr();
            self.attach(node_ptr);

            let key_ref = KeyRef {
                k: unsafe { (*node_ptr).key.as_ptr() },
            };
            self.map.insert(key_ref, node);

            unsafe { &mut (*(*node_ptr).value.as_mut_ptr()) }
        }
    }

    fn peek<'a, Q>(&'a mut self, k: &Q) -> Option<&'a V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map
            .get(k)
            .map(|node| unsafe { &*node.as_ref().value.as_ptr() })
    }

    fn peek_mut<'a, Q>(&'a mut self, k: &Q) -> Option<&'a mut V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map
            .get_mut(k)
            .map(|node| unsafe { &mut *(*(*node).as_ptr()).value.as_mut_ptr() })
    }

    fn peek_last(&'_ mut self) -> Option<(&'_ K, &'_ V)> {
        if self.is_empty() {
            return None;
        }

        let (key, val) = unsafe {
            let node = (*self.tail).prev;

            (&(*(*node).key.as_ptr()), &(*(*node).value.as_ptr()))
        };

        Some((key, val))
    }

    fn contains<Q>(&self, k: &Q) -> bool
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.map.contains_key(k)
    }

    fn pop<Q>(&mut self, k: &Q) -> Option<V>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(k) {
            Some(node) => {
                let mut old_node = unsafe {
                    let mut old_node = *Box::from_raw(node.as_ptr());
                    std::ptr::drop_in_place(old_node.key.as_mut_ptr());

                    old_node
                };

                self.detach(&mut old_node);

                let LRUEntry { value, .. } = old_node;
                Some(unsafe { value.assume_init() })
            },
            None => None,
        }
    }

    fn pop_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        match self.map.remove(k) {
            Some(node) => {
                let mut old_node = unsafe { *Box::from_raw(node.as_ptr()) };
                self.detach(&mut old_node);

                let LRUEntry { key, value, .. } = old_node;
                Some(unsafe { (key.assume_init(), value.assume_init()) })
            },
            None => None,
        }
    }

    fn pop_last(&mut self) -> Option<(K, V)> {
        let node = self.detach_last()?;
        let node = *node;
        let LRUEntry { key, value, .. } = node;

        Some(unsafe { (key.assume_init(), value.assume_init()) })
    }

    fn promote<Q>(&mut self, k: &Q)
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();
            self.detach(node_ptr);
            self.attach(node_ptr);
        }
    }

    fn demote<Q>(&mut self, k: &Q)
    where
        KeyRef<K>: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(node) = self.map.get_mut(k) {
            let node_ptr: *mut LRUEntry<K, V> = (*node).as_ptr();
            self.detach(node_ptr);
            self.attach_last(node_ptr);
        }
    }

    fn resize(&mut self, cap: NonZeroUsize) {
        if cap == self.cap {
            return;
        }

        while self.map.len() > cap.get() {
            self.pop_last();
        }
        self.map.shrink_to_fit();

        self.cap = cap;
    }

    fn clear(&mut self) { while self.pop_last().is_some() {} }
}

impl<K, V, S> Drop for LRUCache<K, V, S> {
    fn drop(&mut self) {
        self.map.drain().for_each(|(_, node)| unsafe {
            let mut node = *Box::from_raw(node.as_ptr());
            std::ptr::drop_in_place((node).key.as_mut_ptr());
            std::ptr::drop_in_place((node).value.as_mut_ptr());
        });

        let _head = unsafe { *Box::from_raw(self.head) };
        let _tail = unsafe { *Box::from_raw(self.tail) };
    }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> IntoIterator for &'a LRUCache<K, V, S> {
    type IntoIter = Iter<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'a, K: Hash + Eq, V, S: BuildHasher> IntoIterator for &'a mut LRUCache<K, V, S> {
    type IntoIter = IterMut<'a, K, V>;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> IterMut<'a, K, V> { self.iter_mut() }
}

unsafe impl<K: Send, V: Send, S: Send> Send for LRUCache<K, V, S> {}
unsafe impl<K: Sync, V: Sync, S: Sync> Sync for LRUCache<K, V, S> {}

impl<K: Hash + Eq, V> fmt::Debug for LRUCache<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LRUCache")
            .field("len", &self.len())
            .field("cap", &self.cap())
            .finish()
    }
}

impl<K: Hash + Eq, V> IntoIterator for LRUCache<K, V> {
    type IntoIter = IntoIter<K, V>;
    type Item = (K, V);

    fn into_iter(self) -> IntoIter<K, V> { IntoIter { cache: self } }
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;
    use core::num::NonZeroUsize;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::LRUCache;
    use crate::lru::cache::Cache;
    extern crate alloc;

    fn assert_opt_eq<V: PartialEq + Debug>(opt: Option<&V>, v: V) {
        assert!(opt.is_some());
        assert_eq!(opt.unwrap(), &v);
    }

    fn assert_opt_eq_mut<V: PartialEq + Debug>(opt: Option<&mut V>, v: V) {
        assert!(opt.is_some());
        assert_eq!(opt.unwrap(), &v);
    }

    fn assert_opt_eq_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
        opt: Option<(&K, &V)>,
        kv: (K, V),
    ) {
        assert!(opt.is_some());
        let res = opt.unwrap();
        assert_eq!(res.0, &kv.0);
        assert_eq!(res.1, &kv.1);
    }

    fn assert_opt_eq_mut_tuple<K: PartialEq + Debug, V: PartialEq + Debug>(
        opt: Option<(&K, &mut V)>,
        kv: (K, V),
    ) {
        assert!(opt.is_some());
        let res = opt.unwrap();
        assert_eq!(res.0, &kv.0);
        assert_eq!(res.1, &kv.1);
    }

    #[test]
    fn test_unbounded() {
        let mut cache = LRUCache::unbounded();
        for i in 0..13370 {
            cache.put(i, ());
        }
        assert_eq!(cache.len(), 13370);
    }

    #[test]
    fn test_put_and_get() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());
        assert!(cache.is_empty());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);

        assert_eq!(cache.cap().get(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");
    }

    #[test]
    fn test_put_and_get_or_insert() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());
        assert!(cache.is_empty());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);

        assert_eq!(cache.cap().get(), 2);
        assert_eq!(cache.len(), 2);
        assert!(!cache.is_empty());
        assert_eq!(cache.get_or_insert("apple", || "orange"), &"red");
        assert_eq!(cache.get_or_insert("banana", || "orange"), &"yellow");
        assert_eq!(cache.get_or_insert("lemon", || "orange"), &"orange");
        assert_eq!(cache.get_or_insert("lemon", || "red"), &"orange");
    }

    #[test]
    fn test_put_and_get_or_insert_mut() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());
        assert!(cache.is_empty());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);

        assert_eq!(cache.cap().get(), 2);
        assert_eq!(cache.len(), 2);

        let v = cache.get_or_insert_mut("apple", || "orange");
        assert_eq!(v, &"red");
        *v = "blue";

        assert_eq!(cache.get_or_insert_mut("apple", || "orange"), &"blue");
        assert_eq!(cache.get_or_insert_mut("banana", || "orange"), &"yellow");
        assert_eq!(cache.get_or_insert_mut("lemon", || "orange"), &"orange");
        assert_eq!(cache.get_or_insert_mut("lemon", || "red"), &"orange");
    }

    #[test]
    fn test_put_and_get_mut() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.cap().get(), 2);
        assert_eq!(cache.len(), 2);
        assert_opt_eq_mut(cache.get_mut(&"apple"), "red");
        assert_opt_eq_mut(cache.get_mut(&"banana"), "yellow");
    }

    #[test]
    fn test_get_mut_and_update() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", 1);
        cache.put("banana", 3);

        {
            let v = cache.get_mut(&"apple").unwrap();
            *v = 4;
        }

        assert_eq!(cache.cap().get(), 2);
        assert_eq!(cache.len(), 2);
        assert_opt_eq_mut(cache.get_mut(&"apple"), 4);
        assert_opt_eq_mut(cache.get_mut(&"banana"), 3);
    }

    #[test]
    fn test_put_update() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("apple", "green"), Some("red"));

        assert_eq!(cache.len(), 1);
        assert_opt_eq(cache.get(&"apple"), "green");
    }

    #[test]
    fn test_put_removes_oldest() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        assert_eq!(cache.put("apple", "red"), None);
        assert_eq!(cache.put("banana", "yellow"), None);
        assert_eq!(cache.put("pear", "green"), None);

        assert!(cache.get(&"apple").is_none());
        assert_opt_eq(cache.get(&"banana"), "yellow");
        assert_opt_eq(cache.get(&"pear"), "green");

        // Even though we inserted "apple" into the cache earlier it has since been removed from
        // the cache so there is no current value for `put` to return.
        assert_eq!(cache.put("apple", "green"), None);
        assert_eq!(cache.put("tomato", "red"), None);

        assert!(cache.get(&"pear").is_none());
        assert_opt_eq(cache.get(&"apple"), "green");
        assert_opt_eq(cache.get(&"tomato"), "red");
    }

    #[test]
    fn test_peek() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_opt_eq(cache.peek(&"banana"), "yellow");
        assert_opt_eq(cache.peek(&"apple"), "red");

        cache.put("pear", "green");

        assert!(cache.peek(&"apple").is_none());
        assert_opt_eq(cache.peek(&"banana"), "yellow");
        assert_opt_eq(cache.peek(&"pear"), "green");
    }

    #[test]
    fn test_peek_mut() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
        assert_opt_eq_mut(cache.peek_mut(&"apple"), "red");
        assert!(cache.peek_mut(&"pear").is_none());

        cache.put("pear", "green");

        assert!(cache.peek_mut(&"apple").is_none());
        assert_opt_eq_mut(cache.peek_mut(&"banana"), "yellow");
        assert_opt_eq_mut(cache.peek_mut(&"pear"), "green");

        {
            let v = cache.peek_mut(&"banana").unwrap();
            *v = "green";
        }

        assert_opt_eq_mut(cache.peek_mut(&"banana"), "green");
    }

    #[test]
    fn test_peek_lru() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        assert!(cache.peek_last().is_none());

        cache.put("apple", "red");
        cache.put("banana", "yellow");
        assert_opt_eq_tuple(cache.peek_last(), ("apple", "red"));

        cache.get(&"apple");
        assert_opt_eq_tuple(cache.peek_last(), ("banana", "yellow"));

        cache.clear();
        assert!(cache.peek_last().is_none());
    }

    #[test]
    fn test_contains() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");
        cache.put("pear", "green");

        assert!(!cache.contains(&"apple"));
        assert!(cache.contains(&"banana"));
        assert!(cache.contains(&"pear"));
    }

    #[test]
    fn test_pop() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");

        let popped = cache.pop(&"apple");
        assert!(popped.is_some());
        assert_eq!(popped.unwrap(), "red");
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&"apple").is_none());
        assert_opt_eq(cache.get(&"banana"), "yellow");
    }

    #[test]
    fn test_pop_entry() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());
        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");

        let popped = cache.pop_entry(&"apple");
        assert!(popped.is_some());
        assert_eq!(popped.unwrap(), ("apple", "red"));
        assert_eq!(cache.len(), 1);
        assert!(cache.get(&"apple").is_none());
        assert_opt_eq(cache.get(&"banana"), "yellow");
    }

    #[test]
    fn test_pop_lru() {
        let mut cache = LRUCache::new(NonZeroUsize::new(200).unwrap());

        for i in 0..75 {
            cache.put(i, "A");
        }
        for i in 0..75 {
            cache.put(i + 100, "B");
        }
        for i in 0..75 {
            cache.put(i + 200, "C");
        }
        assert_eq!(cache.len(), 200);

        for i in 0..75 {
            assert_opt_eq(cache.get(&(74 - i + 100)), "B");
        }
        assert_opt_eq(cache.get(&25), "A");

        for i in 26..75 {
            assert_eq!(cache.pop_last(), Some((i, "A")));
        }
        for i in 0..75 {
            assert_eq!(cache.pop_last(), Some((i + 200, "C")));
        }
        for i in 0..75 {
            assert_eq!(cache.pop_last(), Some((74 - i + 100, "B")));
        }
        assert_eq!(cache.pop_last(), Some((25, "A")));
        for _ in 0..50 {
            assert_eq!(cache.pop_last(), None);
        }
    }

    #[test]
    fn test_clear() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put("apple", "red");
        cache.put("banana", "yellow");

        assert_eq!(cache.len(), 2);
        assert_opt_eq(cache.get(&"apple"), "red");
        assert_opt_eq(cache.get(&"banana"), "yellow");

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_resize_larger() {
        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        cache.put(1, "a");
        cache.put(2, "b");
        cache.resize(NonZeroUsize::new(4).unwrap());
        cache.put(3, "c");
        cache.put(4, "d");

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.get(&1), Some(&"a"));
        assert_eq!(cache.get(&2), Some(&"b"));
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_resize_smaller() {
        let mut cache = LRUCache::new(NonZeroUsize::new(4).unwrap());

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.put(4, "d");

        cache.resize(NonZeroUsize::new(2).unwrap());

        assert_eq!(cache.len(), 2);
        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.get(&3), Some(&"c"));
        assert_eq!(cache.get(&4), Some(&"d"));
    }

    #[test]
    fn test_send() {
        use std::thread;

        let mut cache = LRUCache::new(NonZeroUsize::new(4).unwrap());
        cache.put(1, "a");

        let handle = thread::spawn(move || {
            assert_eq!(cache.get(&1), Some(&"a"));
        });

        assert!(handle.join().is_ok());
    }

    #[test]
    fn test_iter_forwards() {
        let mut cache = LRUCache::new(NonZeroUsize::new(3).unwrap());
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next(), ("a", 1));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next(), ("a", 1));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn test_iter_backwards() {
        let mut cache = LRUCache::new(NonZeroUsize::new(3).unwrap());
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next_back(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next_back(), ("c", 3));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }

        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next_back(), ("b", 2));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next_back(), ("c", 3));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_iter_forwards_and_backwards() {
        let mut cache = LRUCache::new(NonZeroUsize::new(3).unwrap());
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        {
            // iter const
            let mut iter = cache.iter();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
        {
            // iter mut
            let mut iter = cache.iter_mut();
            assert_eq!(iter.len(), 3);
            assert_opt_eq_mut_tuple(iter.next(), ("c", 3));

            assert_eq!(iter.len(), 2);
            assert_opt_eq_mut_tuple(iter.next_back(), ("a", 1));

            assert_eq!(iter.len(), 1);
            assert_opt_eq_mut_tuple(iter.next(), ("b", 2));

            assert_eq!(iter.len(), 0);
            assert_eq!(iter.next_back(), None);
        }
    }

    #[test]
    fn test_iter_clone() {
        let mut cache = LRUCache::new(NonZeroUsize::new(3).unwrap());
        cache.put("a", 1);
        cache.put("b", 2);

        let mut iter = cache.iter();
        let mut iter_clone = iter.clone();

        assert_eq!(iter.len(), 2);
        assert_opt_eq_tuple(iter.next(), ("b", 2));
        assert_eq!(iter_clone.len(), 2);
        assert_opt_eq_tuple(iter_clone.next(), ("b", 2));

        assert_eq!(iter.len(), 1);
        assert_opt_eq_tuple(iter.next(), ("a", 1));
        assert_eq!(iter_clone.len(), 1);
        assert_opt_eq_tuple(iter_clone.next(), ("a", 1));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
        assert_eq!(iter_clone.len(), 0);
        assert_eq!(iter_clone.next(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut cache = LRUCache::new(NonZeroUsize::new(3).unwrap());
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);

        let mut iter = cache.into_iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(("a", 1)));

        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(("b", 2)));

        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(("c", 3)));

        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_that_pop_actually_detaches_node() {
        let mut cache = LRUCache::new(NonZeroUsize::new(5).unwrap());

        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.put("d", 4);
        cache.put("e", 5);

        assert_eq!(cache.pop(&"c"), Some(3));

        cache.put("f", 6);

        let mut iter = cache.iter();
        assert_opt_eq_tuple(iter.next(), ("f", 6));
        assert_opt_eq_tuple(iter.next(), ("e", 5));
        assert_opt_eq_tuple(iter.next(), ("d", 4));
        assert_opt_eq_tuple(iter.next(), ("b", 2));
        assert_opt_eq_tuple(iter.next(), ("a", 1));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_get_with_borrow() {
        use alloc::string::String;

        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        let key = String::from("apple");
        cache.put(key, "red");

        assert_opt_eq(cache.get("apple"), "red");
    }

    #[test]
    fn test_get_mut_with_borrow() {
        use alloc::string::String;

        let mut cache = LRUCache::new(NonZeroUsize::new(2).unwrap());

        let key = String::from("apple");
        cache.put(key, "red");

        assert_opt_eq_mut(cache.get_mut("apple"), "red");
    }

    #[test]
    fn test_no_memory_leaks() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) { DROP_COUNT.fetch_add(1, Ordering::SeqCst); }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = LRUCache::new(NonZeroUsize::new(1).unwrap());
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_clear() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) { DROP_COUNT.fetch_add(1, Ordering::SeqCst); }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = LRUCache::new(NonZeroUsize::new(1).unwrap());
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
            cache.clear();
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_resize() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) { DROP_COUNT.fetch_add(1, Ordering::SeqCst); }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = LRUCache::new(NonZeroUsize::new(1).unwrap());
            for i in 0..n {
                cache.put(i, DropCounter {});
            }
            cache.clear();
        }
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n);
    }

    #[test]
    fn test_no_memory_leaks_with_pop() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        #[derive(Hash, PartialEq, Eq)]
        struct KeyDropCounter(usize);

        impl Drop for KeyDropCounter {
            fn drop(&mut self) { DROP_COUNT.fetch_add(1, Ordering::SeqCst); }
        }

        let n = 100;
        for _ in 0..n {
            let mut cache = LRUCache::new(NonZeroUsize::new(1).unwrap());

            for i in 0..100 {
                cache.put(KeyDropCounter(i), i);
                cache.pop(&KeyDropCounter(i));
            }
        }

        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), n * n * 2);
    }

    #[test]
    fn test_promote_and_demote() {
        let mut cache = LRUCache::new(NonZeroUsize::new(5).unwrap());
        for i in 0..5 {
            cache.push(i, i);
        }
        cache.promote(&1);
        cache.promote(&0);
        cache.demote(&3);
        cache.demote(&4);
        assert_eq!(cache.pop_last(), Some((4, 4)));
        assert_eq!(cache.pop_last(), Some((3, 3)));
        assert_eq!(cache.pop_last(), Some((2, 2)));
        assert_eq!(cache.pop_last(), Some((1, 1)));
        assert_eq!(cache.pop_last(), Some((0, 0)));
        assert_eq!(cache.pop_last(), None);
    }
}
