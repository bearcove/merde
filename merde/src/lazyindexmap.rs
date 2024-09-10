use std::borrow::Borrow;
use std::fmt;
use std::hash::Hash;
use std::slice::Iter as SliceIter;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use ahash::AHashMap;

/// Like [IndexMap](https://docs.rs/indexmap/latest/indexmap/) but only builds the lookup map when it's needed.
///
/// Shamelessly adapted from jiter: <https://github.com/pydantic/jiter/blob/dd25fd094f60fb3c5018ca597e049f903622379e/crates/jiter/src/lazy_index_map.rs>
pub struct LazyIndexMap<K, V> {
    vec: Vec<(K, V)>,
    map: OnceLock<AHashMap<K, usize>>,
    last_find: AtomicUsize,
}

impl<K, V> Default for LazyIndexMap<K, V>
where
    K: Clone + fmt::Debug + Eq + Hash,
    V: fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone, V: Clone> Clone for LazyIndexMap<K, V> {
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            map: self.map.clone(),
            last_find: AtomicUsize::new(0),
        }
    }
}

impl<K, V> fmt::Debug for LazyIndexMap<K, V>
where
    K: Clone + fmt::Debug + Eq + Hash,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter_unique()).finish()
    }
}

// picked to be a good tradeoff after experimenting with `lazy_map_lookup` benchmark, should cover most models
const HASHMAP_THRESHOLD: usize = 16;

/// Like [IndexMap](https://docs.rs/indexmap/latest/indexmap/) but only builds the lookup map when it's needed.
impl<K, V> LazyIndexMap<K, V>
where
    K: Clone + fmt::Debug + Eq + Hash,
    V: fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            vec: Default::default(),
            map: OnceLock::new(),
            last_find: AtomicUsize::new(0),
        }
    }

    pub fn insert(&mut self, key: impl Into<K>, value: impl Into<V>) {
        let key = key.into();
        let value = value.into();
        if let Some(map) = self.map.get_mut() {
            map.insert(key.clone(), self.vec.len());
        }
        self.vec.push((key, value));
    }

    pub fn len(&self) -> usize {
        self.get_map().len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + PartialEq<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let vec_len = self.vec.len();
        // if the vec is longer than the threshold, we use the hashmap for lookups
        if vec_len > HASHMAP_THRESHOLD {
            self.get_map().get(key).map(|&i| &self.vec[i].1)
        } else {
            // otherwise we find the value in the vec
            // we assume the most likely position for the match is at `last_find + 1`
            let first_try = self.last_find.load(Ordering::Relaxed) + 1;
            for i in first_try..first_try + vec_len {
                let index = i % vec_len;
                let (k, v) = &self.vec[index];
                if k == key {
                    self.last_find.store(index, Ordering::Relaxed);
                    return Some(v);
                }
            }
            None
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> {
        self.vec.iter().map(|(k, _)| k)
    }

    pub fn iter(&self) -> SliceIter<'_, (K, V)> {
        self.vec.iter()
    }

    pub fn iter_unique(&self) -> impl Iterator<Item = (&K, &V)> {
        IterUnique {
            vec: &self.vec,
            map: self.get_map(),
            index: 0,
        }
    }

    fn get_map(&self) -> &AHashMap<K, usize> {
        self.map.get_or_init(|| {
            self.vec
                .iter()
                .enumerate()
                .map(|(index, (key, _))| (key.clone(), index))
                .collect()
        })
    }

    fn get_map_mut(&mut self) -> &mut AHashMap<K, usize> {
        self.map.get_or_init(|| {
            self.vec
                .iter()
                .enumerate()
                .map(|(index, (key, _))| (key.clone(), index))
                .collect()
        })
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let vec_len = self.vec.len();

        let mut k_index = None;
        // if the vec is longer than the threshold, we use the hashmap for lookups
        if vec_len > HASHMAP_THRESHOLD {
            k_index = Some(self.get_map().remove(key)?)
        } else {
            // otherwise we find the value in the vec
            // we assume the most likely position for the match is at `last_find + 1`
            let first_try = self.last_find.load(Ordering::Relaxed) + 1;
            for i in first_try..first_try + vec_len {
                let index = i % vec_len;
                let (k, v) = &self.vec[index];
                if k == key {
                    self.last_find.store(index, Ordering::Relaxed);
                    k_index = Some(index);
                }
            }
        };

        // swap the last element with the one we want to remove, then pop
        // the last element off the back
        let k_index = k_index?;

        if vec_len == 1 {
            Some(self.vec.pop().unwrap().1)
        } else {
            let last_index = vec_len - 1;
            self.vec.swap(k_index, last_index);
            Some(self.vec.pop().unwrap().1)
        }
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for LazyIndexMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}

struct IterUnique<'a, K, V> {
    vec: &'a Vec<(K, V)>,
    map: &'a AHashMap<K, usize>,
    index: usize,
}

impl<'a, K: Hash + Eq, V> Iterator for IterUnique<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.vec.len() {
            let (k, v) = &self.vec[self.index];
            if let Some(map_index) = self.map.get(k) {
                if *map_index == self.index {
                    self.index += 1;
                    return Some((k, v));
                }
            }
            self.index += 1;
        }
        None
    }
}
