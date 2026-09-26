//! Tests for first_key/next_key iteration, including mutation mid-iteration.

mod support;

use std::collections::{BTreeMap, BTreeSet};

use rdbm::hash::key_hash;
use rdbm::{CreateOptions, Database};
use support::TempPath;

fn small_pages(max_depth: u8) -> CreateOptions {
    CreateOptions {
        page_size: 512,
        max_depth,
    }
}

fn key(i: usize) -> Vec<u8> {
    format!("key{i}").into_bytes()
}

fn create(
    tmp: &TempPath,
    options: CreateOptions,
    keys: impl IntoIterator<Item = Vec<u8>>,
) -> Database {
    let mut db = Database::create(tmp.path(), options).unwrap();
    db.set_sync(false);
    for k in keys {
        db.put(&k, b"v").unwrap();
    }
    db
}

/// Walks first_key/next_key to the end, returning keys in the order seen.
fn walk(db: &Database) -> Vec<Vec<u8>> {
    let mut keys = Vec::new();
    let mut next = db.first_key().unwrap();
    while let Some(k) = next {
        next = db.next_key(&k).unwrap();
        keys.push(k);
    }
    keys
}

fn sorted(keys: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut keys = keys.to_vec();
    keys.sort();
    keys
}

#[test]
fn empty_database_has_no_first_key() {
    let tmp = TempPath::new("iter-empty.db");
    let db = create(&tmp, CreateOptions::default(), []);
    assert_eq!(db.first_key().unwrap(), None);
    assert_eq!(db.next_key(b"anything").unwrap(), None);
}

#[test]
fn every_key_is_returned_exactly_once() {
    let tmp = TempPath::new("iter-fruit.db");
    let fruit = [b"apple".to_vec(), b"banana".to_vec(), b"cherry".to_vec()];
    drop(create(&tmp, CreateOptions::default(), fruit.clone()));

    let db = Database::open(tmp.path()).unwrap();
    let seen = walk(&db);
    println!(
        "{:?}",
        seen.iter()
            .map(|k| String::from_utf8_lossy(k))
            .collect::<Vec<_>>()
    );
    assert_eq!(sorted(&seen), fruit);
}

#[test]
fn order_is_by_hash_then_key_and_matches_entries() {
    let tmp = TempPath::new("iter-order.db");
    let db = create(&tmp, small_pages(24), (0..1000).map(key));
    let seen = walk(&db);
    assert_eq!(seen.len(), 1000);
    assert!(
        seen.windows(2)
            .all(|w| (key_hash(&w[0]), &w[0]) < (key_hash(&w[1]), &w[1])),
        "keys are not in (hash, key) order"
    );
    let listed: Vec<Vec<u8>> = db.entries().map(|e| e.unwrap().0).collect();
    assert_eq!(listed, seen);
}

#[test]
fn deleted_key_no_longer_appears() {
    let tmp = TempPath::new("iter-delete.db");
    let fruit = [b"apple".to_vec(), b"banana".to_vec(), b"cherry".to_vec()];
    let mut db = create(&tmp, CreateOptions::default(), fruit);
    db.delete(b"banana").unwrap();
    drop(db);

    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(sorted(&walk(&db)), [b"apple".to_vec(), b"cherry".to_vec()]);
}

#[test]
fn next_key_works_from_a_deleted_or_absent_key() {
    let tmp = TempPath::new("iter-absent.db");
    let mut db = create(&tmp, small_pages(24), (0..200).map(key));
    let seen = walk(&db);

    // Deleting the current key does not lose the position.
    db.delete(&seen[50]).unwrap();
    assert_eq!(db.next_key(&seen[50]).unwrap().as_ref(), Some(&seen[51]));

    // A key that was never present resumes at its place in the order.
    let absent = b"never inserted";
    let expected = seen
        .iter()
        .find(|k| (key_hash(k), k.as_slice()) > (key_hash(absent), &absent[..]));
    assert_eq!(db.next_key(absent).unwrap().as_ref(), expected);
}

#[test]
fn deleting_each_key_as_it_is_visited_visits_all() {
    let tmp = TempPath::new("iter-drain.db");
    let n = 500;
    let mut db = create(&tmp, small_pages(24), (0..n).map(key));

    let mut seen = BTreeSet::new();
    let mut next = db.first_key().unwrap();
    while let Some(k) = next {
        assert!(db.delete(&k).unwrap().is_some());
        next = db.next_key(&k).unwrap();
        assert!(seen.insert(k), "key visited twice");
    }
    assert_eq!(seen.len(), n);
    assert!(db.is_empty());
    db.check().unwrap();
}

#[test]
fn inserts_and_splits_during_iteration_do_not_skip_or_repeat() {
    let tmp = TempPath::new("iter-grow.db");
    let n = 300;
    let mut db = create(&tmp, small_pages(24), (0..n).map(key));
    let buckets_before = db.stats().unwrap().buckets;

    let mut visits = BTreeMap::<Vec<u8>, usize>::new();
    let mut next = db.first_key().unwrap();
    let mut added = 0;
    while let Some(k) = next {
        // Grow the table as we go, forcing splits and directory doubling.
        for _ in 0..3 {
            db.put(format!("new{added}").as_bytes(), &[0; 40]).unwrap();
            added += 1;
        }
        next = db.next_key(&k).unwrap();
        *visits.entry(k).or_default() += 1;
    }

    assert!(db.stats().unwrap().buckets > buckets_before * 2);
    assert!(visits.values().all(|&v| v == 1), "a key was visited twice");
    for i in 0..n {
        assert!(visits.contains_key(&key(i)), "original key {i} was skipped");
    }
    db.check().unwrap();
}

#[test]
fn keys_inserted_into_freed_space_are_picked_up() {
    let tmp = TempPath::new("iter-refill.db");
    let mut db = create(&tmp, small_pages(24), (0..100).map(key));
    // Free one record's worth of space per deletion, per bucket.
    let mut freed = BTreeMap::<u64, usize>::new();
    for i in (0..100).step_by(3) {
        *freed.entry(db.bucket_of(&key(i))).or_default() += 1;
        db.delete(&key(i)).unwrap();
    }
    let shape = db.stats().unwrap();

    // Fresh keys "f0".. are shorter than "key0".., so each fits in the space
    // of a deleted record. Place them only where space was freed.
    let fresh: Vec<Vec<u8>> = (0..)
        .map(|j| format!("f{j}").into_bytes())
        .filter(|k| match freed.get_mut(&db.bucket_of(k)) {
            Some(n) if *n > 0 => {
                *n -= 1;
                true
            }
            _ => false,
        })
        .take(20)
        .collect();
    for k in &fresh {
        db.put(k, b"v").unwrap();
    }
    let refilled = db.stats().unwrap();
    assert_eq!(
        (refilled.pages, refilled.buckets),
        (shape.pages, shape.buckets)
    );
    drop(db);

    let db = Database::open(tmp.path()).unwrap();
    let mut expected: Vec<Vec<u8>> = (0..100).filter(|i| i % 3 != 0).map(key).collect();
    expected.extend(fresh);
    expected.sort();
    assert_eq!(sorted(&walk(&db)), expected);
}

#[test]
fn iteration_crosses_overflow_pages() {
    for max_depth in [0, 2] {
        let tmp = TempPath::new("iter-overflow.db");
        let db = create(&tmp, small_pages(max_depth), (0..300).map(key));
        assert!(db.stats().unwrap().overflow_pages > 0);
        let expected: Vec<Vec<u8>> = sorted(&(0..300).map(key).collect::<Vec<_>>());
        assert_eq!(sorted(&walk(&db)), expected, "max depth {max_depth}");
    }
}
