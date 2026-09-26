//! Extendible hash table mapping arbitrary byte-string keys to byte-string values.
//!
//! A directory of `2^global_depth` slots points at buckets. A key's slot is the
//! top `global_depth` bits of its hash. Each bucket holds at most
//! `bucket_capacity` entries and has a `local_depth`: all of its keys share the
//! top `local_depth` hash bits, and `2^(global_depth - local_depth)` adjacent
//! directory slots point at it.
//!
//! When an insert finds its bucket full, only that bucket splits: its local
//! depth grows by one and its entries are divided by the next hash bit. If the
//! local depth already equals the global depth, the directory doubles first.
//! No other bucket is touched, so the table never rehashes everything at once.

use std::num::NonZeroUsize;

use crate::hash::{HASH_BITS, bit, key_hash, prefix};

const DEFAULT_BUCKET_CAPACITY: usize = 32;

#[derive(Debug, Clone)]
struct Entry {
    hash: u64,
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
struct Bucket {
    local_depth: u32,
    entries: Vec<Entry>,
}

impl Bucket {
    fn position(&self, hash: u64, key: &[u8]) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.hash == hash && e.key == key)
    }
}

#[derive(Debug, Clone)]
pub struct HashTable {
    /// Bucket ids, indexed by the top `global_depth` bits of a hash.
    directory: Vec<usize>,
    /// Buckets by id. A split appends the new bucket, so ids are stable.
    buckets: Vec<Bucket>,
    global_depth: u32,
    bucket_capacity: usize,
    len: usize,
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HashTable {
    pub fn new() -> Self {
        Self::with_bucket_capacity(
            NonZeroUsize::new(DEFAULT_BUCKET_CAPACITY).expect("default capacity is non-zero"),
        )
    }

    /// Creates an empty table whose buckets hold at most `capacity` entries.
    pub fn with_bucket_capacity(capacity: NonZeroUsize) -> Self {
        Self {
            directory: vec![0],
            buckets: vec![Bucket {
                local_depth: 0,
                entries: Vec::new(),
            }],
            global_depth: 0,
            bucket_capacity: capacity.get(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn global_depth(&self) -> u32 {
        self.global_depth
    }

    pub fn directory_len(&self) -> usize {
        self.directory.len()
    }

    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    pub fn bucket_capacity(&self) -> usize {
        self.bucket_capacity
    }

    /// The id of the bucket that `key` belongs in, whether or not it is present.
    pub fn bucket_of(&self, key: &[u8]) -> usize {
        self.bucket_id(key_hash(key))
    }

    fn bucket_id(&self, hash: u64) -> usize {
        self.directory[prefix(hash, self.global_depth)]
    }

    /// Inserts or replaces `key`. Returns the previous value, if any.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        let hash = key_hash(key);
        let id = self.bucket_id(hash);
        if let Some(pos) = self.buckets[id].position(hash, key) {
            let entry = &mut self.buckets[id].entries[pos];
            return Some(std::mem::replace(&mut entry.value, value.to_vec()));
        }

        let mut id = id;
        while self.buckets[id].entries.len() >= self.bucket_capacity && self.can_split(id, hash) {
            self.split(id);
            id = self.bucket_id(hash);
        }
        self.buckets[id].entries.push(Entry {
            hash,
            key: key.to_vec(),
            value: value.to_vec(),
        });
        self.len += 1;
        None
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let hash = key_hash(key);
        let bucket = &self.buckets[self.bucket_id(hash)];
        bucket
            .position(hash, key)
            .map(|pos| bucket.entries[pos].value.as_slice())
    }

    /// Removes `key`. Returns its value if it was present.
    ///
    /// Buckets are never merged and the directory never shrinks; the freed
    /// slot is reused by later inserts into the same bucket.
    pub fn delete(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let hash = key_hash(key);
        let id = self.bucket_id(hash);
        let bucket = &mut self.buckets[id];
        let pos = bucket.position(hash, key)?;
        self.len -= 1;
        Some(bucket.entries.swap_remove(pos).value)
    }

    /// Iterates over all `(key, value)` pairs in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.buckets
            .iter()
            .flat_map(|b| &b.entries)
            .map(|e| (e.key.as_slice(), e.value.as_slice()))
    }

    /// Splitting helps only if some entry's hash differs from the incoming one.
    /// If every hash is identical, no bit can separate them, so the bucket is
    /// allowed to exceed its capacity instead.
    fn can_split(&self, id: usize, hash: u64) -> bool {
        let bucket = &self.buckets[id];
        bucket.local_depth < HASH_BITS && bucket.entries.iter().any(|e| e.hash != hash)
    }

    fn split(&mut self, id: usize) {
        let depth = self.buckets[id].local_depth;
        if depth == self.global_depth {
            self.double_directory();
        }

        let entries = std::mem::take(&mut self.buckets[id].entries);
        let (moved, kept): (Vec<_>, Vec<_>) = entries.into_iter().partition(|e| bit(e.hash, depth));
        self.buckets[id] = Bucket {
            local_depth: depth + 1,
            entries: kept,
        };
        let new_id = self.buckets.len();
        self.buckets.push(Bucket {
            local_depth: depth + 1,
            entries: moved,
        });

        // The slots pointing at `id` form one aligned block; its upper half
        // (next hash bit set) now points at the new bucket.
        let shift = self.global_depth - (depth + 1);
        for (slot, target) in self.directory.iter_mut().enumerate() {
            if *target == id && (slot >> shift) & 1 == 1 {
                *target = new_id;
            }
        }
    }

    /// Doubles the directory. With most-significant-bit indexing, old slot `i`
    /// becomes slots `2i` and `2i + 1`, both pointing at the same bucket.
    fn double_directory(&mut self) {
        self.directory = self.directory.iter().flat_map(|&b| [b, b]).collect();
        self.global_depth += 1;
    }

    /// Verifies the structural invariants of the table, returning a
    /// description of the first violation found.
    pub fn check_consistency(&self) -> Result<(), String> {
        let g = self.global_depth;
        if self.directory.len() != 1usize << g {
            return Err(format!(
                "directory has {} slots, expected 2^{g}",
                self.directory.len()
            ));
        }

        // For each bucket: the prefix its slots share, and how many slots.
        let mut block = vec![None; self.buckets.len()];
        let mut refs = vec![0usize; self.buckets.len()];
        for (slot, &id) in self.directory.iter().enumerate() {
            let bucket = self
                .buckets
                .get(id)
                .ok_or(format!("slot {slot} points at missing bucket {id}"))?;
            if bucket.local_depth > g {
                return Err(format!("bucket {id} is deeper than the directory"));
            }
            let p = slot >> (g - bucket.local_depth);
            match block[id] {
                None => block[id] = Some(p),
                Some(q) if q != p => {
                    return Err(format!("bucket {id} is referenced by non-adjacent slots"));
                }
                Some(_) => {}
            }
            refs[id] += 1;
        }

        let mut total = 0;
        for (id, bucket) in self.buckets.iter().enumerate() {
            let d = bucket.local_depth;
            if d > g {
                return Err(format!("bucket {id} is deeper than the directory"));
            }
            if refs[id] != 1usize << (g - d) {
                return Err(format!(
                    "bucket {id} has {} slots, expected 2^{}",
                    refs[id],
                    g - d
                ));
            }
            let Some(p) = block[id] else {
                return Err(format!("bucket {id} is unreachable"));
            };
            let overflowing = bucket.entries.len() > self.bucket_capacity;
            if overflowing && bucket.entries.windows(2).any(|w| w[0].hash != w[1].hash) {
                return Err(format!("bucket {id} is over capacity but could split"));
            }
            for (i, e) in bucket.entries.iter().enumerate() {
                if e.hash != key_hash(&e.key) {
                    return Err(format!("bucket {id} has an entry with a stale hash"));
                }
                if prefix(e.hash, d) != p {
                    return Err(format!("bucket {id} holds a key from another bucket"));
                }
                if bucket.entries[..i].iter().any(|o| o.key == e.key) {
                    return Err(format!("bucket {id} holds a duplicate key"));
                }
            }
            total += bucket.entries.len();
        }
        if total != self.len {
            return Err(format!("len is {}, but buckets hold {total}", self.len));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny(capacity: usize) -> HashTable {
        HashTable::with_bucket_capacity(NonZeroUsize::new(capacity).unwrap())
    }

    #[test]
    fn empty_table() {
        let t = HashTable::new();
        assert!(t.is_empty());
        assert_eq!(t.get(b"missing"), None);
        assert_eq!(
            (t.global_depth(), t.directory_len(), t.bucket_count()),
            (0, 1, 1)
        );
        t.check_consistency().unwrap();
    }

    #[test]
    fn put_then_get() {
        let mut t = HashTable::new();
        assert_eq!(t.put(b"key", b"value"), None);
        assert_eq!(t.get(b"key"), Some(&b"value"[..]));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn put_replaces_and_returns_old_value() {
        let mut t = HashTable::new();
        t.put(b"k", b"v1");
        assert_eq!(t.put(b"k", b"v2"), Some(b"v1".to_vec()));
        assert_eq!(t.get(b"k"), Some(&b"v2"[..]));
        assert_eq!(t.len(), 1);
    }

    #[test]
    fn replacing_in_a_full_bucket_does_not_split() {
        let mut t = tiny(1);
        t.put(b"k", b"v1");
        t.put(b"k", b"v2");
        assert_eq!(t.bucket_count(), 1);
    }

    #[test]
    fn delete_removes_key() {
        let mut t = HashTable::new();
        t.put(b"k", b"v");
        assert_eq!(t.delete(b"k"), Some(b"v".to_vec()));
        assert_eq!(t.get(b"k"), None);
        assert_eq!(t.delete(b"k"), None);
        assert!(t.is_empty());
    }

    #[test]
    fn empty_key_and_value_are_valid() {
        let mut t = HashTable::new();
        t.put(b"", b"");
        assert_eq!(t.get(b""), Some(&b""[..]));
    }

    #[test]
    fn binary_keys_and_values() {
        let mut t = HashTable::new();
        let key = [0u8, 255, 0, 10, 13];
        let value = [0u8; 1024];
        t.put(&key, &value);
        assert_eq!(t.get(&key), Some(&value[..]));
        assert_eq!(t.get(&key[..4]), None);
    }

    #[test]
    fn first_overflow_splits_the_only_bucket() {
        let mut t = tiny(2);
        t.put(b"a", b"1");
        t.put(b"b", b"2");
        assert_eq!(t.bucket_count(), 1);
        t.put(b"c", b"3");
        assert!(t.bucket_count() >= 2);
        assert!(t.global_depth() >= 1);
        t.check_consistency().unwrap();
        for k in [b"a", b"b", b"c"] {
            assert!(t.get(k).is_some());
        }
    }

    #[test]
    fn split_below_global_depth_does_not_double_directory() {
        let mut t = tiny(1);
        let mut i = 0u32;
        // Grow until some bucket is shallower than the directory.
        while t.buckets.iter().all(|b| b.local_depth == t.global_depth) {
            t.put(&i.to_le_bytes(), b"");
            i += 1;
        }
        let shallow = (0..t.bucket_count())
            .find(|&id| t.buckets[id].local_depth < t.global_depth)
            .unwrap();
        let depth_before = t.global_depth();
        let dir_before = t.directory_len();
        t.split(shallow);
        assert_eq!(t.global_depth(), depth_before);
        assert_eq!(t.directory_len(), dir_before);
        t.check_consistency().unwrap();
    }

    #[test]
    fn many_keys_survive_splits() {
        let mut t = tiny(4);
        let n = 10_000u32;
        for i in 0..n {
            t.put(&i.to_le_bytes(), format!("v{i}").as_bytes());
        }
        t.check_consistency().unwrap();
        assert_eq!(t.len(), n as usize);
        for i in 0..n {
            assert_eq!(t.get(&i.to_le_bytes()), Some(format!("v{i}").as_bytes()));
        }
        for i in (0..n).step_by(2) {
            assert!(t.delete(&i.to_le_bytes()).is_some());
        }
        t.check_consistency().unwrap();
        assert_eq!(t.len(), (n / 2) as usize);
        for i in 0..n {
            assert_eq!(t.get(&i.to_le_bytes()).is_some(), i % 2 == 1);
        }
    }

    #[test]
    fn identical_hashes_overflow_instead_of_splitting_forever() {
        let mut t = tiny(1);
        let hash = key_hash(b"x");
        t.buckets[0].entries.push(Entry {
            hash,
            key: b"x".to_vec(),
            value: b"1".to_vec(),
        });
        t.len = 1;
        assert!(!t.can_split(0, hash));
        assert!(t.can_split(0, !hash));
    }

    #[test]
    fn iter_visits_every_entry_once() {
        let mut t = tiny(3);
        for i in 0..100u8 {
            t.put(&[i], &[i]);
        }
        let mut keys: Vec<u8> = t.iter().map(|(k, _)| k[0]).collect();
        keys.sort_unstable();
        assert_eq!(keys, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn consistency_check_detects_misplaced_key() {
        let mut t = tiny(1);
        for i in 0..8u8 {
            t.put(&[i], b"");
        }
        // Buckets own disjoint hash prefixes, so any other bucket is wrong.
        let from = t.bucket_of(&[0]);
        let to = (0..t.bucket_count()).find(|&id| id != from).unwrap();
        let entry = t.buckets[from].entries.pop().unwrap();
        t.buckets[to].entries.push(entry);
        assert!(t.check_consistency().is_err());
    }
}
