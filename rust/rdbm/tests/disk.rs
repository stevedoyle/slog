//! Persistence tests for the on-disk database. Each test closes and reopens
//! the file to prove that data came from disk, not memory.

mod support;

use std::fs;

use rdbm::{CreateOptions, Database, Error};
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

fn value(i: usize) -> Vec<u8> {
    format!("value number {i}").into_bytes()
}

/// Creates a database and inserts keys `0..n` without syncing each write.
fn populated(tmp: &TempPath, options: CreateOptions, n: usize) -> Database {
    let mut db = Database::create(tmp.path(), options).unwrap();
    db.set_sync(false);
    for i in 0..n {
        assert_eq!(db.put(&key(i), &value(i)).unwrap(), None, "key {i}");
    }
    db.sync().unwrap();
    db
}

#[test]
fn keys_survive_reopen() {
    let tmp = TempPath::new("reopen.db");
    {
        let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
        db.put(b"key1", b"value one").unwrap();
        db.put(b"key2", b"value two").unwrap();
    }
    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(db.get(b"key1").unwrap(), Some(b"value one".to_vec()));
    assert_eq!(db.get(b"key2").unwrap(), Some(b"value two".to_vec()));
    assert_eq!(db.get(b"key3").unwrap(), None);
    assert_eq!(db.len(), 2);
    db.check().unwrap();
}

#[test]
fn deleted_key_stays_gone_after_reopen() {
    let tmp = TempPath::new("delete.db");
    {
        let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
        db.put(b"key1", b"value one").unwrap();
        db.put(b"key2", b"value two").unwrap();
        assert_eq!(db.delete(b"key1").unwrap(), Some(b"value one".to_vec()));
        assert_eq!(db.delete(b"key1").unwrap(), None);
    }
    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(db.get(b"key1").unwrap(), None);
    assert_eq!(db.get(b"key2").unwrap(), Some(b"value two".to_vec()));
    assert_eq!(db.len(), 1);
    db.check().unwrap();
}

#[test]
fn replacing_a_value_persists() {
    let tmp = TempPath::new("replace.db");
    {
        let mut db = Database::create(tmp.path(), small_pages(24)).unwrap();
        db.put(b"k", b"short").unwrap();
        let long = vec![b'x'; 400];
        assert_eq!(db.put(b"k", &long).unwrap(), Some(b"short".to_vec()));
        assert_eq!(db.put(b"k", b"tiny").unwrap(), Some(long));
    }
    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(db.get(b"k").unwrap(), Some(b"tiny".to_vec()));
    assert_eq!(db.len(), 1);
    db.check().unwrap();
}

#[test]
fn binary_and_empty_keys_and_values() {
    let tmp = TempPath::new("binary.db");
    {
        let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
        db.put(b"", b"empty key").unwrap();
        db.put(b"empty value", b"").unwrap();
        db.put(&[0, 255, b'\n', b'\t'], &[0; 64]).unwrap();
    }
    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(db.get(b"").unwrap(), Some(b"empty key".to_vec()));
    assert_eq!(db.get(b"empty value").unwrap(), Some(Vec::new()));
    assert_eq!(db.get(&[0, 255, b'\n', b'\t']).unwrap(), Some(vec![0; 64]));
}

#[test]
fn many_keys_split_buckets_and_relocate_the_directory() {
    let tmp = TempPath::new("splits.db");
    let n = 3000;
    drop(populated(&tmp, small_pages(24), n));

    let db = Database::open(tmp.path()).unwrap();
    let s = db.stats().unwrap();
    println!("{s:?}");
    assert!(s.buckets > 100, "{s:?}");
    // A 512-byte page holds 64 slots, so depth 7+ needs a relocated directory.
    assert!(s.global_depth >= 7 && s.directory_pages > 1, "{s:?}");
    assert_eq!(s.overflow_pages, 0);
    assert_eq!(s.entries, n as u64);
    for i in 0..n {
        assert_eq!(db.get(&key(i)).unwrap(), Some(value(i)), "key {i}");
    }
    db.check().unwrap();
}

#[test]
fn overflow_pages_hold_keys_past_max_depth() {
    let tmp = TempPath::new("overflow.db");
    let n = 400;
    // Max depth 2 allows at most 4 buckets; 400 records need about 30 pages.
    drop(populated(&tmp, small_pages(2), n));

    let mut db = Database::open(tmp.path()).unwrap();
    let s = db.stats().unwrap();
    println!("after insert: {s:?}");
    assert_eq!(s.global_depth, 2);
    assert!(s.overflow_pages > 10, "{s:?}");
    for i in 0..n {
        assert_eq!(db.get(&key(i)).unwrap(), Some(value(i)), "key {i}");
    }
    db.check().unwrap();

    // Deleting everything frees every overflow page.
    db.set_sync(false);
    for i in 0..n {
        assert_eq!(db.delete(&key(i)).unwrap(), Some(value(i)), "key {i}");
    }
    db.sync().unwrap();
    drop(db);
    let mut db = Database::open(tmp.path()).unwrap();
    let emptied = db.stats().unwrap();
    println!("after delete: {emptied:?}");
    assert_eq!(emptied.overflow_pages, 0);
    assert_eq!(emptied.free_pages, s.overflow_pages);
    assert!(db.is_empty());
    db.check().unwrap();

    // Re-inserting reuses the freed pages; the file does not grow.
    db.set_sync(false);
    for i in 0..n {
        db.put(&key(i), &value(i)).unwrap();
    }
    db.sync().unwrap();
    drop(db);
    let db = Database::open(tmp.path()).unwrap();
    let refilled = db.stats().unwrap();
    assert_eq!(refilled.pages, s.pages);
    assert_eq!(refilled.free_pages, 0);
    for i in 0..n {
        assert_eq!(db.get(&key(i)).unwrap(), Some(value(i)), "key {i}");
    }
    db.check().unwrap();
}

#[test]
fn max_depth_zero_is_a_single_overflowing_bucket() {
    let tmp = TempPath::new("depth0.db");
    drop(populated(&tmp, small_pages(0), 100));
    let db = Database::open(tmp.path()).unwrap();
    let s = db.stats().unwrap();
    assert_eq!((s.global_depth, s.buckets), (0, 1));
    assert!(s.overflow_pages > 0);
    assert_eq!(db.entries().count(), 100);
    db.check().unwrap();
}

#[test]
fn entries_visits_every_pair_once() {
    let tmp = TempPath::new("entries.db");
    let db = populated(&tmp, small_pages(24), 500);
    let mut keys: Vec<Vec<u8>> = db.entries().map(|e| e.unwrap().0).collect();
    keys.sort();
    let mut expected: Vec<Vec<u8>> = (0..500).map(key).collect();
    expected.sort();
    assert_eq!(keys, expected);
}

#[test]
fn oversized_record_is_rejected_without_changing_the_file() {
    let tmp = TempPath::new("toolarge.db");
    let mut db = Database::create(tmp.path(), small_pages(24)).unwrap();
    db.put(b"k", b"v").unwrap();
    let before = fs::read(tmp.path()).unwrap();

    // 512 - 16 byte page header - 16 byte slot = 480 bytes of key and value.
    let err = db.put(b"big", &[0; 478]).unwrap_err();
    assert!(
        matches!(
            err,
            Error::TooLarge {
                size: 481,
                max: 480
            }
        ),
        "{err}"
    );
    assert_eq!(fs::read(tmp.path()).unwrap(), before);

    db.put(b"big", &[0; 477]).unwrap();
    assert_eq!(db.get(b"big").unwrap(), Some(vec![0; 477]));
    db.check().unwrap();
}

#[test]
fn create_refuses_to_overwrite() {
    let tmp = TempPath::new("exists.db");
    fs::write(tmp.path(), b"precious").unwrap();
    assert!(Database::create(tmp.path(), CreateOptions::default()).is_err());
    assert_eq!(fs::read(tmp.path()).unwrap(), b"precious");
}

#[test]
fn create_rejects_bad_options() {
    let tmp = TempPath::new("options.db");
    for options in [
        CreateOptions {
            page_size: 1000,
            max_depth: 24,
        },
        CreateOptions {
            page_size: 4096,
            max_depth: 33,
        },
    ] {
        let err = Database::create(tmp.path(), options).unwrap_err();
        assert!(matches!(err, Error::InvalidOption(_)), "{err}");
        assert!(!tmp.path().exists());
    }
}

#[test]
fn open_rejects_files_that_are_not_databases() {
    let tmp = TempPath::new("garbage.db");
    assert!(matches!(Database::open(tmp.path()), Err(Error::Io(_))));

    fs::write(tmp.path(), b"").unwrap();
    assert!(matches!(Database::open(tmp.path()), Err(Error::Corrupt(_))));

    fs::write(tmp.path(), vec![b'x'; 4096]).unwrap();
    assert!(matches!(Database::open(tmp.path()), Err(Error::Corrupt(_))));
}

#[test]
fn open_rejects_truncated_file() {
    let tmp = TempPath::new("truncated.db");
    drop(Database::create(tmp.path(), CreateOptions::default()).unwrap());
    let bytes = fs::read(tmp.path()).unwrap();
    fs::write(tmp.path(), &bytes[..bytes.len() - 1]).unwrap();
    assert!(matches!(Database::open(tmp.path()), Err(Error::Corrupt(_))));
}

#[test]
fn check_detects_a_corrupted_record() {
    let tmp = TempPath::new("corrupt.db");
    {
        let mut db = Database::create(tmp.path(), small_pages(24)).unwrap();
        db.put(b"key1", b"value one").unwrap();
    }
    // The only bucket is page 2; its one record ends the page. Flip a key byte.
    let mut bytes = fs::read(tmp.path()).unwrap();
    let key_start = 3 * 512 - "key1value one".len();
    assert_eq!(&bytes[key_start..key_start + 4], b"key1");
    bytes[key_start] = b'K';
    fs::write(tmp.path(), &bytes).unwrap();

    let db = Database::open(tmp.path()).unwrap();
    let err = db.check().unwrap_err();
    assert!(err.to_string().contains("wrong hash"), "{err}");
}

#[test]
fn read_only_handle_rejects_writes() {
    let tmp = TempPath::new("readonly.db");
    drop(Database::create(tmp.path(), CreateOptions::default()).unwrap());
    let mut db = Database::open_read_only(tmp.path()).unwrap();
    assert!(db.put(b"k", b"v").is_err());
}

#[test]
fn stats_report_file_size_and_fill() {
    let tmp = TempPath::new("stats.db");
    let db = populated(&tmp, CreateOptions::default(), 2000);
    let full = db.stats().unwrap();
    assert_eq!(full.file_size, fs::metadata(tmp.path()).unwrap().len());
    assert_eq!(full.file_size, full.pages * u64::from(full.page_size));
    assert!((30..=100).contains(&full.fill_percent()), "{full:?}");
    drop(db);

    let mut db = Database::open(tmp.path()).unwrap();
    db.set_sync(false);
    for i in 0..1900 {
        db.delete(&key(i)).unwrap();
    }
    let sparse = db.stats().unwrap();
    assert!(
        sparse.fill_percent() < full.fill_percent() / 4,
        "{sparse:?}"
    );
    drop(db);

    rdbm::reorganise(tmp.path()).unwrap();
    let compact = Database::open(tmp.path()).unwrap().stats().unwrap();
    assert!(
        compact.fill_percent() > sparse.fill_percent() * 2,
        "{compact:?}"
    );
    assert!(compact.file_size < sparse.file_size);
}
