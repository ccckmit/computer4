use std::alloc::{alloc, dealloc, Layout};
use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem;

struct Bucket<K, V> {
    hash: u64,
    key: Option<Box<K>>,
    value: Option<Box<V>>,
}

pub struct SwisstableMap<K, V> {
    buckets: *mut Bucket<K, V>,
    capacity: usize,
    len: usize,
    hasher: std::collections::hash_map::RandomState,
}

impl<K: Hash + Eq, V> SwisstableMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity_and_hasher(4, std::collections::hash_map::RandomState::new())
    }

    pub fn with_capacity_and_hasher(
        capacity: usize,
        hasher: std::collections::hash_map::RandomState,
    ) -> Self {
        let cap = capacity.next_power_of_two().max(16);
        let layout = Layout::array::<Bucket<K, V>>(cap).unwrap();
        let buckets = unsafe { alloc(layout) as *mut Bucket<K, V> };

        unsafe {
            for i in 0..cap {
                std::ptr::write(
                    buckets.add(i),
                    Bucket {
                        hash: 0,
                        key: None,
                        value: None,
                    },
                );
            }
        }

        SwisstableMap {
            buckets,
            capacity: cap,
            len: 0,
            hasher,
        }
    }

    fn probe_index(&self, hash: u64) -> usize {
        (hash as usize) & (self.capacity - 1)
    }

    pub fn insert(&mut self, mut key: K, mut value: V) -> Option<V> {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let mut hash = hasher.finish();

        let mut index = self.probe_index(hash);
        let mut dist = 0usize;

        loop {
            unsafe {
                let bucket = &mut *self.buckets.add(index);

                if bucket.key.is_none() {
                    bucket.hash = hash;
                    bucket.key = Some(Box::new(key));
                    bucket.value = Some(Box::new(value));
                    self.len += 1;
                    return None;
                }

                if bucket.key.as_ref().map(|k| **k == key).unwrap_or(false) {
                    let old_value = bucket.value.take().unwrap();
                    bucket.value = Some(Box::new(value));
                    return Some(*old_value);
                }

                if bucket.hash == hash {
                    // Hash matches but key doesn't - might be collision, continue
                } else {
                    let existing_index = self.probe_index(bucket.hash);
                    let existing_dist = (index.wrapping_sub(existing_index)) & (self.capacity - 1);

                    if dist > existing_dist {
                        let old_hash = bucket.hash;
                        let old_key = bucket.key.take().unwrap();
                        let old_value = bucket.value.take().unwrap();

                        bucket.hash = hash;
                        bucket.key = Some(Box::new(key));
                        bucket.value = Some(Box::new(value));

                        key = *old_key;
                        value = *old_value;
                        hash = old_hash;
                        dist = existing_dist;
                    }
                }

                dist += 1;
                index = (index + 1) & (self.capacity - 1);

                if dist >= self.capacity {
                    self.resize();
                    return self.insert(key, value);
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = self.probe_index(hash);
        let mut dist = 0usize;

        loop {
            unsafe {
                let bucket = &*self.buckets.add(index);

                if bucket.key.is_none() {
                    return None;
                }

                if bucket.hash == hash && bucket.key.as_ref().map(|k| **k == *key).unwrap_or(false)
                {
                    return bucket.value.as_ref().map(|v| &**v);
                }

                let existing_index = self.probe_index(bucket.hash);
                let existing_dist = (index.wrapping_sub(existing_index)) & (self.capacity - 1);

                if dist > existing_dist {
                    return None;
                }

                dist += 1;
                index = (index + 1) & (self.capacity - 1);

                if dist >= self.capacity {
                    return None;
                }
            }
        }
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let mut hasher = self.hasher.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();

        let mut index = self.probe_index(hash);
        let mut dist = 0usize;

        loop {
            unsafe {
                let bucket = &mut *self.buckets.add(index);

                if bucket.key.is_none() {
                    return None;
                }

                if bucket.hash == hash && bucket.key.as_ref().map(|k| **k == *key).unwrap_or(false)
                {
                    let result = bucket.value.take().map(|v| *v);
                    bucket.key.take();
                    bucket.hash = 0;
                    self.len -= 1;

                    self.remove_shift(index);
                    return result;
                }

                let existing_index = self.probe_index(bucket.hash);
                let existing_dist = (index.wrapping_sub(existing_index)) & (self.capacity - 1);

                if dist > existing_dist {
                    return None;
                }

                dist += 1;
                index = (index + 1) & (self.capacity - 1);

                if dist >= self.capacity {
                    return None;
                }
            }
        }
    }

    fn remove_shift(&mut self, mut hole: usize) {
        let mut i = (hole + 1) & (self.capacity - 1);

        unsafe {
            loop {
                let bucket_i = &mut *self.buckets.add(i);

                if bucket_i.key.is_none() {
                    break;
                }

                let ideal = self.probe_index(bucket_i.hash);

                if (hole.wrapping_sub(ideal)) < (i.wrapping_sub(ideal)) {
                    let bucket_hole = &mut *self.buckets.add(hole);
                    bucket_hole.hash = bucket_i.hash;
                    bucket_hole.key = bucket_i.key.take();
                    bucket_hole.value = bucket_i.value.take();

                    hole = i;
                }

                i = (i + 1) & (self.capacity - 1);

                if i == hole {
                    break;
                }
            }
        }
    }

    fn resize(&mut self) {
        let new_cap = self.capacity * 2;
        let layout = Layout::array::<Bucket<K, V>>(new_cap).unwrap();
        let mut new_buckets = unsafe { alloc(layout) as *mut Bucket<K, V> };

        unsafe {
            for i in 0..new_cap {
                std::ptr::write(
                    new_buckets.add(i),
                    Bucket {
                        hash: 0,
                        key: None,
                        value: None,
                    },
                );
            }

            let old_cap = self.capacity;
            let old_buckets = self.buckets;

            mem::swap(&mut self.buckets, &mut new_buckets);
            self.capacity = new_cap;
            self.len = 0;

            for i in 0..old_cap {
                let bucket = &mut *old_buckets.add(i);
                if let Some(key) = bucket.key.take() {
                    let value = bucket.value.take().unwrap();
                    let hash = bucket.hash;

                    let mut index = (hash as usize) & (self.capacity - 1);
                    loop {
                        let new_bucket = &mut *self.buckets.add(index);
                        if new_bucket.key.is_none() {
                            new_bucket.hash = hash;
                            new_bucket.key = Some(key);
                            new_bucket.value = Some(value);
                            self.len += 1;
                            break;
                        }
                        index = (index + 1) & (self.capacity - 1);
                    }
                }
            }

            dealloc(
                old_buckets as *mut u8,
                Layout::array::<Bucket<K, V>>(old_cap).unwrap(),
            );
        }
    }

    fn shrink_to_fit(&mut self) {
        if self.len < self.capacity / 4 && self.capacity > 16 {
            let new_cap = self.capacity / 2;
            let layout = Layout::array::<Bucket<K, V>>(new_cap).unwrap();
            let mut new_buckets = unsafe { alloc(layout) as *mut Bucket<K, V> };

            unsafe {
                for i in 0..new_cap {
                    std::ptr::write(
                        new_buckets.add(i),
                        Bucket {
                            hash: 0,
                            key: None,
                            value: None,
                        },
                    );
                }

                let old_cap = self.capacity;
                let old_buckets = self.buckets;
                mem::swap(&mut self.buckets, &mut new_buckets);
                self.capacity = new_cap;
                self.len = 0;

                for i in 0..old_cap {
                    let bucket = &mut *old_buckets.add(i);
                    if bucket.key.is_some() {
                        let hash = bucket.hash;
                        let key = bucket.key.take().unwrap();
                        let value = bucket.value.take().unwrap();

                        let mut index = (hash as usize) & (self.capacity - 1);
                        loop {
                            let new_bucket = &mut *self.buckets.add(index);
                            if new_bucket.key.is_none() {
                                new_bucket.hash = hash;
                                new_bucket.key = Some(key);
                                new_bucket.value = Some(value);
                                self.len += 1;
                                break;
                            }
                            index = (index + 1) & (self.capacity - 1);
                        }
                    }
                }

                dealloc(
                    new_buckets as *mut u8,
                    Layout::array::<Bucket<K, V>>(old_cap).unwrap(),
                );
            }
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<K, V> Drop for SwisstableMap<K, V> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.capacity {
                std::ptr::drop_in_place(self.buckets.add(i));
            }
            dealloc(
                self.buckets as *mut u8,
                Layout::array::<Bucket<K, V>>(self.capacity).unwrap(),
            );
        }
    }
}

impl<K: Hash + Eq + fmt::Debug, V: fmt::Debug> fmt::Debug for SwisstableMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.into_iter()).finish()
    }
}

impl<K: Hash + Eq, V> Default for SwisstableMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Iter<'a, K, V> {
    map: &'a SwisstableMap<K, V>,
    index: usize,
}

impl<'a, K, V> IntoIterator for &'a SwisstableMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            index: 0,
        }
    }
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.map.capacity {
            unsafe {
                let bucket = &*self.map.buckets.add(self.index);
                self.index += 1;
                if let Some(ref key) = bucket.key {
                    if let Some(ref value) = bucket.value {
                        return Some((key.as_ref(), value.as_ref()));
                    }
                }
            }
        }
        None
    }
}

pub struct SwisstableSet<T> {
    map: SwisstableMap<T, ()>,
}

impl<T: Hash + Eq> SwisstableSet<T> {
    pub fn new() -> Self {
        SwisstableSet {
            map: SwisstableMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        SwisstableSet {
            map: SwisstableMap::with_capacity_and_hasher(
                capacity,
                std::collections::hash_map::RandomState::new(),
            ),
        }
    }

    pub fn insert(&mut self, value: T) -> bool {
        self.map.insert(value, ()).is_none()
    }

    pub fn contains(&self, value: &T) -> bool {
        self.map.get(value).is_some()
    }

    pub fn remove(&mut self, value: &T) -> Option<()> {
        self.map.remove(value)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

impl<T: Hash + Eq> Default for SwisstableSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq + fmt::Debug> fmt::Debug for SwisstableSet<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.into_iter()).finish()
    }
}

impl<'a, T: Hash + Eq> IntoIterator for &'a SwisstableSet<T> {
    type Item = &'a T;
    type IntoIter = IterSet<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterSet {
            iter: (&self.map).into_iter(),
        }
    }
}

pub struct IterSet<'a, T> {
    iter: Iter<'a, T, ()>,
}

impl<'a, T> Iterator for IterSet<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(k, &())| k)
    }
}

fn main() {
    let mut map = SwisstableMap::new();
    map.insert("key1", 100);
    map.insert("key2", 200);
    map.insert("key3", 300);

    println!("len: {}", map.len());
    println!("key1 = {:?}", map.get(&"key1"));
    println!("key2 = {:?}", map.get(&"key2"));

    for (k, v) in &map {
        println!("{:?}: {:?}", k, v);
    }

    map.remove(&"key2");
    println!("After remove key2: {:?}", map.get(&"key2"));

    let mut set = SwisstableSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);
    println!("\nSet len: {}", set.len());
    println!("Contains 2: {}", set.contains(&2));
    println!("Set: {:?}", set);
}

#[cfg(test)]
mod tests {
    use super::*;

    mod swisstable_map_tests {
        use super::*;

        #[test]
        fn test_new_map_is_empty() {
            let map: SwisstableMap<i32, i32> = SwisstableMap::new();
            assert!(map.is_empty());
            assert_eq!(map.len(), 0);
        }

        #[test]
        fn test_insert_single_element() {
            let mut map = SwisstableMap::new();
            let prev = map.insert(1, 100);
            assert!(prev.is_none());
            assert_eq!(map.len(), 1);
            assert_eq!(map.get(&1), Some(&100));
        }

        #[test]
        fn test_insert_multiple_elements() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            map.insert(2, 200);
            map.insert(3, 300);
            assert_eq!(map.len(), 3);
            assert_eq!(map.get(&1), Some(&100));
            assert_eq!(map.get(&2), Some(&200));
            assert_eq!(map.get(&3), Some(&300));
        }

        #[test]
        fn test_insert_update_existing_key() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            let old = map.insert(1, 999);
            assert_eq!(old, Some(100));
            assert_eq!(map.len(), 1);
            assert_eq!(map.get(&1), Some(&999));
        }

        #[test]
        fn test_insert_string_keys() {
            let mut map = SwisstableMap::new();
            map.insert("apple", 1);
            map.insert("banana", 2);
            map.insert("cherry", 3);
            assert_eq!(map.len(), 3);
            assert_eq!(map.get(&"apple"), Some(&1));
            assert_eq!(map.get(&"banana"), Some(&2));
            assert_eq!(map.get(&"cherry"), Some(&3));
        }

        #[test]
        fn test_get_nonexistent_key() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            assert_eq!(map.get(&2), None);
        }

        #[test]
        fn test_get_from_empty_map() {
            let map: SwisstableMap<i32, i32> = SwisstableMap::new();
            assert_eq!(map.get(&1), None);
        }

        #[test]
        fn test_remove_single_element() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            let removed = map.remove(&1);
            assert_eq!(removed, Some(100));
            assert!(map.is_empty());
            assert_eq!(map.get(&1), None);
        }

        #[test]
        fn test_remove_nonexistent_key() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            let removed = map.remove(&2);
            assert!(removed.is_none());
            assert_eq!(map.len(), 1);
        }

        #[test]
        fn test_remove_from_empty_map() {
            let mut map: SwisstableMap<i32, i32> = SwisstableMap::new();
            let removed = map.remove(&1);
            assert!(removed.is_none());
        }

        #[test]
        fn test_remove_then_insert() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            map.remove(&1);
            map.insert(1, 200);
            assert_eq!(map.len(), 1);
            assert_eq!(map.get(&1), Some(&200));
        }

        #[test]
        fn test_is_empty_after_operations() {
            let mut map = SwisstableMap::new();
            assert!(map.is_empty());
            map.insert(1, 100);
            assert!(!map.is_empty());
            map.remove(&1);
            assert!(map.is_empty());
        }

        #[test]
        fn test_capacity_initial() {
            let map: SwisstableMap<i32, i32> = SwisstableMap::new();
            assert!(map.capacity() >= 16);
        }

        #[test]
        fn test_capacity_increases_on_resize() {
            let mut map = SwisstableMap::new();
            let initial_cap = map.capacity();
            for i in 0..100 {
                map.insert(i, i);
            }
            assert!(map.capacity() > initial_cap);
        }

        #[test]
        fn test_many_insertions() {
            let mut map = SwisstableMap::new();
            for i in 0..1000 {
                map.insert(i, i * 10);
            }
            assert_eq!(map.len(), 1000);
            for i in 0..1000 {
                assert_eq!(map.get(&i), Some(&(i * 10)));
            }
        }

        #[test]
        fn test_many_insertions_with_duplicate_updates() {
            let mut map = SwisstableMap::new();
            for i in 0..100 {
                map.insert(i, 0);
            }
            for i in 0..100 {
                map.insert(i, i);
            }
            assert_eq!(map.len(), 100);
            for i in 0..100 {
                assert_eq!(map.get(&i), Some(&i));
            }
        }

        #[test]
        fn test_many_removals() {
            let mut map = SwisstableMap::new();
            for i in 0..100 {
                map.insert(i, i);
            }
            for i in 0..50 {
                map.remove(&i);
            }
            assert_eq!(map.len(), 50);
            for i in 0..50 {
                assert_eq!(map.get(&i), None);
            }
            for i in 50..100 {
                assert_eq!(map.get(&i), Some(&i));
            }
        }

        #[test]
        fn test_collision_handling() {
            let mut map = SwisstableMap::new();
            let a = 1000;
            let b = 2000;
            let c = 3000;
            map.insert(a, 1);
            map.insert(b, 2);
            map.insert(c, 3);
            assert_eq!(map.get(&a), Some(&1));
            assert_eq!(map.get(&b), Some(&2));
            assert_eq!(map.get(&c), Some(&3));
        }

        #[test]
        fn test_iterate_empty_map() {
            let map: SwisstableMap<i32, i32> = SwisstableMap::new();
            let mut count = 0;
            for _ in &map {
                count += 1;
            }
            assert_eq!(count, 0);
        }

        #[test]
        fn test_iterate_single_element() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            let mut count = 0;
            for (k, v) in &map {
                assert_eq!(*k, 1);
                assert_eq!(*v, 100);
                count += 1;
            }
            assert_eq!(count, 1);
        }

        #[test]
        fn test_iterate_multiple_elements() {
            let mut map = SwisstableMap::new();
            for i in 0..10 {
                map.insert(i, i * 10);
            }
            let mut count = 0;
            for (k, v) in &map {
                assert_eq!(*v, *k * 10);
                count += 1;
            }
            assert_eq!(count, 10);
        }

        #[test]
        fn test_debug_trait() {
            let mut map = SwisstableMap::new();
            map.insert(1, 100);
            map.insert(2, 200);
            let debug_str = format!("{:?}", map);
            assert!(debug_str.contains("1"));
            assert!(debug_str.contains("100"));
            assert!(debug_str.contains("2"));
            assert!(debug_str.contains("200"));
        }

        #[test]
        fn test_default_trait() {
            let map: SwisstableMap<i32, i32> = Default::default();
            assert!(map.is_empty());
        }

        #[test]
        fn test_tuple_keys() {
            let mut map = SwisstableMap::new();
            map.insert((1, 2), 100);
            map.insert((3, 4), 200);
            assert_eq!(map.len(), 2);
            assert_eq!(map.get(&(1, 2)), Some(&100));
            assert_eq!(map.get(&(3, 4)), Some(&200));
        }

        #[test]
        fn test_vec_as_value() {
            let mut map = SwisstableMap::new();
            map.insert(1, vec![1, 2, 3]);
            map.insert(2, vec![4, 5, 6]);
            assert_eq!(map.get(&1), Some(&vec![1, 2, 3]));
            assert_eq!(map.get(&2), Some(&vec![4, 5, 6]));
        }

        #[test]
        fn test_large_key_values() {
            let mut map = SwisstableMap::new();
            let large_string = "a".repeat(1000);
            map.insert(large_string.clone(), large_string.clone());
            assert_eq!(map.get(&large_string), Some(&large_string));
        }
    }

    mod swisstable_set_tests {
        use super::*;

        #[test]
        fn test_new_set_is_empty() {
            let set: SwisstableSet<i32> = SwisstableSet::new();
            assert!(set.is_empty());
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_set_insert_single() {
            let mut set = SwisstableSet::new();
            let inserted = set.insert(1);
            assert!(inserted);
            assert_eq!(set.len(), 1);
            assert!(set.contains(&1));
        }

        #[test]
        fn test_set_insert_duplicate() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            let inserted = set.insert(1);
            assert!(!inserted);
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn test_set_insert_multiple() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            set.insert(2);
            set.insert(3);
            assert_eq!(set.len(), 3);
            assert!(set.contains(&1));
            assert!(set.contains(&2));
            assert!(set.contains(&3));
        }

        #[test]
        fn test_set_does_not_contain() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            assert!(!set.contains(&2));
        }

        #[test]
        fn test_set_remove() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            set.insert(2);
            let removed = set.remove(&1);
            assert_eq!(removed, Some(()));
            assert_eq!(set.len(), 1);
            assert!(!set.contains(&1));
            assert!(set.contains(&2));
        }

        #[test]
        fn test_set_remove_nonexistent() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            let removed = set.remove(&2);
            assert!(removed.is_none());
            assert_eq!(set.len(), 1);
        }

        #[test]
        fn test_set_is_empty_after_clear() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            set.remove(&1);
            assert!(set.is_empty());
        }

        #[test]
        fn test_set_with_capacity() {
            let set = SwisstableSet::<i32>::with_capacity(100);
            assert_eq!(set.len(), 0);
        }

        #[test]
        fn test_set_many_insertions() {
            let mut set = SwisstableSet::new();
            for i in 0..1000 {
                set.insert(i);
            }
            assert_eq!(set.len(), 1000);
            for i in 0..1000 {
                assert!(set.contains(&i));
            }
        }

        #[test]
        fn test_set_iterate_empty() {
            let set: SwisstableSet<i32> = SwisstableSet::new();
            let mut count = 0;
            for _ in &set {
                count += 1;
            }
            assert_eq!(count, 0);
        }

        #[test]
        fn test_set_iterate_multiple() {
            let mut set = SwisstableSet::new();
            for i in 0..10 {
                set.insert(i);
            }
            let mut count = 0;
            for val in &set {
                assert!(*val >= 0 && *val < 10);
                count += 1;
            }
            assert_eq!(count, 10);
        }

        #[test]
        fn test_set_debug_trait() {
            let mut set = SwisstableSet::new();
            set.insert(1);
            set.insert(2);
            let debug_str = format!("{:?}", set);
            assert!(debug_str.contains("1"));
            assert!(debug_str.contains("2"));
        }

        #[test]
        fn test_set_default_trait() {
            let set: SwisstableSet<i32> = Default::default();
            assert!(set.is_empty());
        }

        #[test]
        fn test_set_string_elements() {
            let mut set = SwisstableSet::new();
            set.insert("apple");
            set.insert("banana");
            set.insert("cherry");
            assert_eq!(set.len(), 3);
            assert!(set.contains(&"apple"));
            assert!(set.contains(&"banana"));
            assert!(set.contains(&"cherry"));
            assert!(!set.contains(&"durian"));
        }
    }

    mod swisstable_map_stress_tests {
        use super::*;

        #[test]
        fn test_stress_many_keys() {
            let mut map = SwisstableMap::new();
            let n = 10000;
            for i in 0..n {
                map.insert(i, i);
            }
            assert_eq!(map.len(), n);
            for i in 0..n {
                assert_eq!(map.get(&i), Some(&i));
            }
        }

        #[test]
        fn test_stress_mixed_operations() {
            let mut map = SwisstableMap::new();
            for i in 0..1000 {
                map.insert(i, i);
            }
            for i in 0..500 {
                map.remove(&i);
            }
            assert!(map.len() >= 500);
            for i in 0..500 {
                map.insert(i + 1000, i + 1000);
            }
            assert!(map.len() >= 1500);
        }

        #[test]
        fn test_stress_reinsert_removed() {
            let mut map = SwisstableMap::new();
            for i in 0..100 {
                map.insert(i, i);
            }
            for i in 0..100 {
                map.remove(&i);
            }
            for i in 0..100 {
                map.insert(i, i * 2);
            }
            assert_eq!(map.len(), 100);
            for i in 0..100 {
                assert_eq!(map.get(&i), Some(&(i * 2)));
            }
        }

        #[test]
        fn test_stress_alternating_insert_remove() {
            let mut map = SwisstableMap::new();
            for iteration in 0..100 {
                let base = iteration * 100;
                for i in base..base + 100 {
                    map.insert(i, i);
                }
                for i in base..base + 50 {
                    map.remove(&i);
                }
            }
            assert!(map.len() >= 5000);
            for i in (0..10000).step_by(100) {
                if let Some(j) = (i as usize).checked_add(50) {
                    for k in i..j {
                        assert!(!map.get(&k).is_some(), "key {} should be removed", k);
                    }
                }
            }
        }

        #[test]
        fn test_stress_many_string_keys() {
            let mut map = SwisstableMap::new();
            let n = 100;
            for i in 0..n {
                let key = format!("key_{}", i);
                let value = format!("value_{}", i);
                map.insert(key.clone(), value.clone());
            }
            assert_eq!(map.len(), n);
        }

        #[test]
        fn test_stress_all_remove_then_reload() {
            let mut map = SwisstableMap::new();
            let n = 500;
            for i in 0..n {
                map.insert(i, i);
            }
            assert_eq!(map.len(), n);
            for i in 0..n {
                map.remove(&i);
            }
            assert!(map.is_empty());
            for i in 0..n {
                map.insert(i, i * 10);
            }
            assert_eq!(map.len(), n);
            for i in 0..n {
                assert_eq!(map.get(&i), Some(&(i * 10)));
            }
        }
    }

    mod swisstable_set_stress_tests {
        use super::*;

        #[test]
        fn test_stress_many_insertions() {
            let mut set = SwisstableSet::new();
            let n = 10000;
            for i in 0..n {
                set.insert(i);
            }
            assert_eq!(set.len(), n);
            for i in 0..n {
                assert!(set.contains(&i));
            }
        }

        #[test]
        fn test_stress_mixed_operations() {
            let mut set = SwisstableSet::new();
            for i in 0..1000 {
                set.insert(i);
            }
            for i in 0..500 {
                set.remove(&i);
            }
            for i in 0..500 {
                assert!(!set.contains(&i));
            }
            for i in 500..1000 {
                assert!(set.contains(&i));
            }
            for i in 1000..2000 {
                set.insert(i);
            }
            assert!(set.len() >= 1000);
            for i in 1000..2000 {
                set.insert(i);
            }
            assert!(set.len() >= 1500);
        }

        #[test]
        fn test_stress_reinsert_removed() {
            let mut set = SwisstableSet::new();
            for i in 0..100 {
                set.insert(i);
            }
            for i in 0..100 {
                set.remove(&i);
            }
            for i in 0..100 {
                set.insert(i * 2);
            }
            assert_eq!(set.len(), 100);
            for i in 0..100 {
                assert!(set.contains(&(i * 2)));
            }
        }

        #[test]
        fn test_stress_alternating_insert_remove() {
            let mut set = SwisstableSet::new();
            for iteration in 0..100 {
                let base = iteration * 100;
                for i in base..base + 100 {
                    set.insert(i);
                }
                for i in base..base + 50 {
                    set.remove(&i);
                }
            }
            assert!(set.len() >= 5000);
        }
    }

    mod swisstable_edge_case_tests {
        use super::*;

        #[test]
        fn test_zero_key() {
            let mut map = SwisstableMap::new();
            map.insert(0, 100);
            assert_eq!(map.get(&0), Some(&100));
            let mut set = SwisstableSet::new();
            set.insert(0);
            assert!(set.contains(&0));
        }

        #[test]
        fn test_negative_keys() {
            let mut map = SwisstableMap::new();
            map.insert(-1, 100);
            map.insert(-100, 200);
            assert_eq!(map.get(&-1), Some(&100));
            assert_eq!(map.get(&-100), Some(&200));
        }

        #[test]
        fn test_max_usize_key() {
            let mut map = SwisstableMap::new();
            map.insert(usize::MAX, 100);
            assert_eq!(map.get(&usize::MAX), Some(&100));
        }

        #[test]
        fn test_zero_value() {
            let mut map = SwisstableMap::new();
            map.insert(1, 0);
            assert_eq!(map.get(&1), Some(&0));
        }

        #[test]
        fn test_empty_string_key() {
            let mut map = SwisstableMap::new();
            map.insert("", 100);
            assert_eq!(map.get(&""), Some(&100));
        }

        #[test]
        fn test_empty_string_value() {
            let mut map = SwisstableMap::new();
            map.insert("key", "");
            assert_eq!(map.get(&"key"), Some(&""));
        }

        #[test]
        fn test_single_char_string_key() {
            let mut map = SwisstableMap::new();
            map.insert("a", 1);
            map.insert("b", 2);
            map.insert("c", 3);
            assert_eq!(map.get(&"a"), Some(&1));
            assert_eq!(map.get(&"b"), Some(&2));
            assert_eq!(map.get(&"c"), Some(&3));
        }

        #[test]
        fn test_vec_key() {
            let mut map = SwisstableMap::new();
            map.insert(vec![1, 2, 3], 100);
            assert_eq!(map.get(&vec![1, 2, 3]), Some(&100));
            assert_eq!(map.get(&vec![1, 2]), None);
        }

        #[test]
        fn test_bool_keys() {
            let mut map = SwisstableMap::new();
            map.insert(true, 1);
            map.insert(false, 0);
            assert_eq!(map.get(&true), Some(&1));
            assert_eq!(map.get(&false), Some(&0));
        }

        #[test]
        fn test_char_keys() {
            let mut map = SwisstableMap::new();
            map.insert('a', 1);
            map.insert('b', 2);
            map.insert('c', 3);
            assert_eq!(map.get(&'a'), Some(&1));
            assert_eq!(map.get(&'b'), Some(&2));
            assert_eq!(map.get(&'c'), Some(&3));
        }

        #[test]
        fn test_all_ascii_chars() {
            let mut map = SwisstableMap::new();
            for i in 0..128 {
                let c = char::from_u32(i).unwrap();
                map.insert(c, i);
            }
            assert_eq!(map.len(), 128);
        }

        #[test]
        fn test_option_value() {
            let mut map = SwisstableMap::new();
            map.insert(1, Some(100));
            map.insert(2, None);
            assert_eq!(map.get(&1), Some(&Some(100)));
            assert_eq!(map.get(&2), Some(&None));
        }

        #[test]
        fn test_result_value() {
            let mut map: SwisstableMap<i32, Result<i32, &str>> = SwisstableMap::new();
            map.insert(1, Ok(100));
            map.insert(2, Err("error"));
            assert_eq!(map.get(&1), Some(&Ok(100)));
            assert_eq!(map.get(&2), Some(&Err("error")));
        }
    }
}
