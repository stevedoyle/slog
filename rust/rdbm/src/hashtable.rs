//! In-memory hash table mapping arbitrary byte-string keys to byte-string values.
//!
//! Collisions are resolved by separate chaining: each bucket holds a vector of
//! entries. The bucket count is always a power of two so a bucket index is a
//! cheap mask of the hash. The table doubles when the load factor exceeds
//! `MAX_LOAD_NUM / MAX_LOAD_DEN`.

const INITIAL_BUCKETS: usize = 16;
const MAX_LOAD_NUM: usize = 3;
const MAX_LOAD_DEN: usize = 4;

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// 64-bit FNV-1a hash. Deterministic across runs, which matters once buckets
/// live on disk.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(FNV_OFFSET_BASIS, |hash, &b| {
        (hash ^ u64::from(b)).wrapping_mul(FNV_PRIME)
    })
}

#[derive(Debug, Clone)]
struct Entry {
    hash: u64,
    key: Vec<u8>,
    value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct HashTable {
    buckets: Vec<Vec<Entry>>,
    len: usize,
}

impl Default for HashTable {
    fn default() -> Self {
        Self::new()
    }
}

impl HashTable {
    pub fn new() -> Self {
        Self::with_buckets(INITIAL_BUCKETS)
    }

    fn with_buckets(n: usize) -> Self {
        debug_assert!(n.is_power_of_two());
        Self {
            buckets: (0..n).map(|_| Vec::new()).collect(),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn bucket_index(&self, hash: u64) -> usize {
        // Truncation is intentional: only the low bits select the bucket.
        (hash as usize) & (self.buckets.len() - 1)
    }

    /// Inserts or replaces `key`. Returns the previous value, if any.
    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>> {
        let hash = fnv1a(key);
        let idx = self.bucket_index(hash);
        if let Some(entry) = self.buckets[idx]
            .iter_mut()
            .find(|e| e.hash == hash && e.key == key)
        {
            return Some(std::mem::replace(&mut entry.value, value.to_vec()));
        }

        self.buckets[idx].push(Entry {
            hash,
            key: key.to_vec(),
            value: value.to_vec(),
        });
        self.len += 1;
        if self.len * MAX_LOAD_DEN > self.buckets.len() * MAX_LOAD_NUM {
            self.grow();
        }
        None
    }

    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let hash = fnv1a(key);
        self.buckets[self.bucket_index(hash)]
            .iter()
            .find(|e| e.hash == hash && e.key == key)
            .map(|e| e.value.as_slice())
    }

    /// Removes `key`. Returns its value if it was present.
    pub fn delete(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        let hash = fnv1a(key);
        let idx = self.bucket_index(hash);
        let bucket = &mut self.buckets[idx];
        let pos = bucket.iter().position(|e| e.hash == hash && e.key == key)?;
        self.len -= 1;
        Some(bucket.swap_remove(pos).value)
    }

    /// Iterates over all `(key, value)` pairs in unspecified order.
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])> {
        self.buckets
            .iter()
            .flatten()
            .map(|e| (e.key.as_slice(), e.value.as_slice()))
    }

    fn grow(&mut self) {
        let mut bigger = Self::with_buckets(self.buckets.len() * 2);
        for entry in self.buckets.drain(..).flatten() {
            let idx = bigger.bucket_index(entry.hash);
            bigger.buckets[idx].push(entry);
        }
        bigger.len = self.len;
        *self = bigger;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_known_vectors() {
        assert_eq!(fnv1a(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(fnv1a(b"foobar"), 0x8594_4171_f739_67e8);
    }

    #[test]
    fn empty_table() {
        let t = HashTable::new();
        assert!(t.is_empty());
        assert_eq!(t.get(b"missing"), None);
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
    fn collisions_within_one_bucket() {
        // A single-bucket table forces every key into the same chain.
        let mut t = HashTable::with_buckets(1);
        for i in 0..3u8 {
            let hash = fnv1a(&[i]);
            t.buckets[0].push(Entry {
                hash,
                key: vec![i],
                value: vec![i * 2],
            });
            t.len += 1;
        }
        assert_eq!(t.get(&[1]), Some(&[2u8][..]));
        assert_eq!(t.delete(&[0]), Some(vec![0]));
        assert_eq!(t.get(&[2]), Some(&[4u8][..]));
        assert_eq!(t.get(&[1]), Some(&[2u8][..]));
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn many_keys_survive_resizing() {
        let mut t = HashTable::new();
        let n = 10_000u32;
        for i in 0..n {
            t.put(&i.to_le_bytes(), format!("v{i}").as_bytes());
        }
        assert_eq!(t.len(), n as usize);
        assert!(t.buckets.len() > INITIAL_BUCKETS);
        for i in 0..n {
            assert_eq!(t.get(&i.to_le_bytes()), Some(format!("v{i}").as_bytes()));
        }
        for i in (0..n).step_by(2) {
            assert!(t.delete(&i.to_le_bytes()).is_some());
        }
        assert_eq!(t.len(), (n / 2) as usize);
        for i in 0..n {
            assert_eq!(t.get(&i.to_le_bytes()).is_some(), i % 2 == 1);
        }
    }

    #[test]
    fn iter_visits_every_entry_once() {
        let mut t = HashTable::new();
        for i in 0..100u8 {
            t.put(&[i], &[i]);
        }
        let mut keys: Vec<u8> = t.iter().map(|(k, _)| k[0]).collect();
        keys.sort_unstable();
        assert_eq!(keys, (0..100).collect::<Vec<_>>());
    }
}
