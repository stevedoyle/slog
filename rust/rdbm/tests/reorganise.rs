//! Tests for rebuilding a database to reclaim space.

mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use rdbm::{CreateOptions, Database, Error, reorganise};
use support::TempPath;

fn key(i: usize) -> Vec<u8> {
    format!("key{i}").into_bytes()
}

fn value(i: usize) -> Vec<u8> {
    format!("value for key {i}").into_bytes()
}

fn file_size(path: &Path) -> u64 {
    fs::metadata(path).unwrap().len()
}

/// Inserts keys `0..inserted`, then deletes `0..deleted`.
fn fragmented(tmp: &TempPath, options: CreateOptions, inserted: usize, deleted: usize) {
    let mut db = Database::create(tmp.path(), options).unwrap();
    db.set_sync(false);
    for i in 0..inserted {
        db.put(&key(i), &value(i)).unwrap();
    }
    for i in 0..deleted {
        db.delete(&key(i)).unwrap();
    }
    db.sync().unwrap();
}

fn temp_file_for(path: &Path) -> std::path::PathBuf {
    let name = path.file_name().unwrap().to_str().unwrap();
    path.with_file_name(format!(".{name}.reorganise"))
}

#[test]
fn shrinks_a_fragmented_database_and_keeps_every_key() {
    let tmp = TempPath::new("reorg-frag.db");
    fragmented(&tmp, CreateOptions::default(), 3000, 2900);
    let before = Database::open(tmp.path()).unwrap().stats().unwrap();
    let size_before = file_size(tmp.path());

    reorganise(tmp.path()).unwrap();

    let db = Database::open(tmp.path()).unwrap();
    let after = db.stats().unwrap();
    println!("before: {before:?}\nafter:  {after:?}");
    assert!(file_size(tmp.path()) < size_before / 4);
    assert!(after.buckets < before.buckets);
    assert!(after.global_depth < before.global_depth);
    assert_eq!(after.free_pages, 0);
    assert_eq!(after.entries, 100);
    for i in 0..3000 {
        let expected = (i >= 2900).then(|| value(i));
        assert_eq!(db.get(&key(i)).unwrap(), expected, "key {i}");
    }
    db.check().unwrap();
    assert!(!temp_file_for(tmp.path()).exists());
}

#[test]
fn spec_scenario_keeps_remaining_key() {
    let tmp = TempPath::new("reorg-spec.db");
    {
        let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
        db.put(b"a", b"1").unwrap();
        db.put(b"b", b"2").unwrap();
        db.put(b"c", b"3").unwrap();
        db.delete(b"a").unwrap();
        db.delete(b"c").unwrap();
    }
    let size_before = file_size(tmp.path());
    reorganise(tmp.path()).unwrap();

    // Three small keys never split the first bucket, so the file was
    // already minimal: header, directory, one bucket.
    assert!(file_size(tmp.path()) <= size_before);
    let db = Database::open(tmp.path()).unwrap();
    assert_eq!(db.get(b"b").unwrap(), Some(b"2".to_vec()));
    assert_eq!(db.get(b"a").unwrap(), None);
    assert_eq!(db.len(), 1);
    db.check().unwrap();
}

#[test]
fn drops_free_pages_and_unneeded_overflow() {
    let tmp = TempPath::new("reorg-overflow.db");
    let options = CreateOptions {
        page_size: 512,
        max_depth: 2,
    };
    fragmented(&tmp, options, 600, 500);
    let before = Database::open(tmp.path()).unwrap().stats().unwrap();
    assert!(before.free_pages > 0, "{before:?}");

    reorganise(tmp.path()).unwrap();

    let db = Database::open(tmp.path()).unwrap();
    let after = db.stats().unwrap();
    println!("before: {before:?}\nafter:  {after:?}");
    assert_eq!(after.free_pages, 0);
    assert!(after.overflow_pages <= before.overflow_pages);
    assert!(after.pages < before.pages);
    assert_eq!(db.options(), options, "options are preserved");
    for i in 500..600 {
        assert_eq!(db.get(&key(i)).unwrap(), Some(value(i)), "key {i}");
    }
    db.check().unwrap();
}

#[test]
fn reorganising_twice_changes_nothing() {
    let tmp = TempPath::new("reorg-twice.db");
    fragmented(&tmp, CreateOptions::default(), 1000, 500);
    reorganise(tmp.path()).unwrap();
    let once = fs::read(tmp.path()).unwrap();
    reorganise(tmp.path()).unwrap();
    assert_eq!(fs::read(tmp.path()).unwrap(), once);
}

#[test]
fn empty_database() {
    let tmp = TempPath::new("reorg-empty.db");
    fragmented(&tmp, CreateOptions::default(), 200, 200);
    reorganise(tmp.path()).unwrap();
    let db = Database::open(tmp.path()).unwrap();
    assert!(db.is_empty());
    assert_eq!(db.stats().unwrap().pages, 3);
    db.check().unwrap();
}

#[test]
fn preserves_file_permissions() {
    let tmp = TempPath::new("reorg-perms.db");
    fragmented(&tmp, CreateOptions::default(), 10, 5);
    fs::set_permissions(tmp.path(), fs::Permissions::from_mode(0o640)).unwrap();
    reorganise(tmp.path()).unwrap();
    let mode = fs::metadata(tmp.path()).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o640);
}

#[test]
fn corrupt_database_is_left_untouched() {
    let tmp = TempPath::new("reorg-corrupt.db");
    fragmented(&tmp, CreateOptions::default(), 10, 0);
    // Claim one more entry than the buckets hold.
    let mut bytes = fs::read(tmp.path()).unwrap();
    bytes[48] += 1;
    fs::write(tmp.path(), &bytes).unwrap();

    let err = reorganise(tmp.path()).unwrap_err();
    assert!(matches!(err, Error::Corrupt(_)), "{err}");
    assert_eq!(fs::read(tmp.path()).unwrap(), bytes);
    assert!(
        !temp_file_for(tmp.path()).exists(),
        "temporary file removed"
    );
}

#[test]
fn existing_temporary_file_is_not_clobbered() {
    let tmp = TempPath::new("reorg-stale.db");
    fragmented(&tmp, CreateOptions::default(), 10, 0);
    let temp = temp_file_for(tmp.path());
    fs::write(&temp, b"someone else's").unwrap();

    let err = reorganise(tmp.path()).unwrap_err();
    assert!(err.to_string().contains("remove it and retry"), "{err}");
    assert_eq!(fs::read(&temp).unwrap(), b"someone else's");
    fs::remove_file(&temp).unwrap();
}

#[test]
fn missing_database_is_an_error() {
    let tmp = TempPath::new("reorg-missing.db");
    assert!(matches!(reorganise(tmp.path()), Err(Error::Io(_))));
    assert!(!temp_file_for(tmp.path()).exists());
}
