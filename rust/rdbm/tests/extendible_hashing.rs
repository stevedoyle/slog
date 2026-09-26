//! End-to-end exercise of extendible hashing through the public API.
//!
//! Run with `cargo test --test extendible_hashing -- --nocapture` to see the
//! directory state printed after every insert.

use std::num::NonZeroUsize;

use rdbm::HashTable;

const BUCKET_CAPACITY: usize = 4;
const KEYS: usize = 64;

fn key(i: usize) -> Vec<u8> {
    format!("key {i}").into_bytes()
}

fn value(i: usize) -> Vec<u8> {
    format!("value {i}").into_bytes()
}

fn state(t: &HashTable) -> String {
    format!(
        "global depth={}, buckets={}, directory size={}",
        t.global_depth(),
        t.bucket_count(),
        t.directory_len()
    )
}

fn table_with_keys(n: usize, verbose: bool) -> (HashTable, usize) {
    let mut t = HashTable::with_bucket_capacity(NonZeroUsize::new(BUCKET_CAPACITY).unwrap());
    let mut splits = 0;
    for i in 1..=n {
        let before = t.bucket_count();
        assert_eq!(t.put(&key(i), &value(i)), None);
        let split = t.bucket_count() - before;
        splits += split;
        if verbose {
            let marker = if split > 0 { "  <- split" } else { "" };
            println!("Inserting key {i:>2}:  {}{marker}", state(&t));
        }
        t.check_consistency()
            .unwrap_or_else(|e| panic!("after inserting key {i}: {e}"));
    }
    (t, splits)
}

#[test]
fn inserts_split_buckets_and_every_key_is_retrievable() {
    let (t, splits) = table_with_keys(KEYS, true);
    println!("{splits} splits, final state: {}", state(&t));

    assert!(splits >= 3, "expected at least 3 splits, got {splits}");
    assert_eq!(t.bucket_count(), splits + 1);
    assert_eq!(t.directory_len(), 1 << t.global_depth());
    assert_eq!(t.len(), KEYS);
    for i in 1..=KEYS {
        assert_eq!(t.get(&key(i)), Some(value(i).as_slice()), "key {i}");
    }
}

#[test]
fn deleted_keys_are_gone_and_freed_space_is_reused() {
    let (mut t, _) = table_with_keys(KEYS, false);
    let deleted: Vec<usize> = (1..=KEYS).step_by(5).collect();

    // Remember where each deleted key lived.
    let homes: Vec<usize> = deleted.iter().map(|&i| t.bucket_of(&key(i))).collect();
    for &i in &deleted {
        assert_eq!(t.delete(&key(i)), Some(value(i)), "key {i}");
    }
    t.check_consistency().unwrap();
    assert_eq!(t.len(), KEYS - deleted.len());
    for i in 1..=KEYS {
        assert_eq!(t.get(&key(i)).is_some(), !deleted.contains(&i), "key {i}");
    }

    // Deletes never merge buckets or shrink the directory.
    let shape = (t.global_depth(), t.bucket_count(), t.directory_len());

    // Re-inserting the deleted keys fits in the space they left behind, so
    // nothing splits and each key lands back in its original bucket.
    for (&i, &home) in deleted.iter().zip(&homes) {
        assert_eq!(t.put(&key(i), b"reinserted"), None);
        assert_eq!(t.bucket_of(&key(i)), home, "key {i}");
        assert_eq!(t.get(&key(i)), Some(&b"reinserted"[..]));
    }
    assert_eq!(
        (t.global_depth(), t.bucket_count(), t.directory_len()),
        shape
    );
    t.check_consistency().unwrap();

    // Brand-new keys go wherever their hash prefix points; the consistency
    // check confirms every key sits in the bucket its prefix selects.
    let fresh = KEYS + 1..KEYS + 21;
    for i in fresh.clone() {
        t.put(&key(i), &value(i));
    }
    t.check_consistency().unwrap();
    assert_eq!(t.len(), KEYS + fresh.len());
    for i in fresh {
        assert_eq!(t.get(&key(i)), Some(value(i).as_slice()), "key {i}");
    }
}
