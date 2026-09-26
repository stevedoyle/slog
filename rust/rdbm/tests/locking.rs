//! File locking: readers share the file, a writer has it to itself, and
//! reorganise excludes everyone. Handles in one process lock against each
//! other just as separate processes do, so these tests use threads.
//!
//! Nothing here spawns a process: a child shares its parent's locks until it
//! execs, which would make a lock linger after its handle is dropped. The
//! CLI's locking is tested in `cli.rs`.

mod support;

use std::fs;
use std::thread;
use std::time::{Duration, Instant};

use rdbm::{CreateOptions, Database, Error, OpenOptions};
use support::TempPath;

/// Long enough for another thread to reach its blocking lock call.
const PAUSE: Duration = Duration::from_millis(200);

fn created(tmp: &TempPath) {
    let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
    db.put(b"k", b"v").unwrap();
}

fn waiting(read_only: bool) -> OpenOptions {
    OpenOptions {
        read_only,
        wait: true,
    }
}

#[test]
fn readers_share_the_file() {
    let tmp = TempPath::new("readers.db");
    created(&tmp);
    let a = Database::open_read_only(tmp.path()).unwrap();
    let b = Database::open_read_only(tmp.path()).unwrap();
    assert_eq!(a.get(b"k").unwrap(), b.get(b"k").unwrap());
}

#[test]
fn a_writer_excludes_readers_and_other_writers() {
    let tmp = TempPath::new("writer.db");
    created(&tmp);
    let writer = Database::open(tmp.path()).unwrap();
    assert!(matches!(Database::open(tmp.path()), Err(Error::Locked)));
    assert!(matches!(
        Database::open_read_only(tmp.path()),
        Err(Error::Locked)
    ));
    drop(writer);

    let reader = Database::open_read_only(tmp.path()).unwrap();
    assert!(matches!(Database::open(tmp.path()), Err(Error::Locked)));
    drop(reader);
    Database::open(tmp.path()).unwrap();
}

#[test]
fn a_new_database_is_locked_until_dropped() {
    let tmp = TempPath::new("create.db");
    let db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
    assert!(matches!(
        Database::open_read_only(tmp.path()),
        Err(Error::Locked)
    ));
    drop(db);
    Database::open_read_only(tmp.path()).unwrap();
}

#[test]
fn wait_blocks_until_the_lock_is_released() {
    let tmp = TempPath::new("wait.db");
    created(&tmp);
    let writer = Database::open(tmp.path()).unwrap();
    let start = Instant::now();
    let releaser = thread::spawn(move || {
        thread::sleep(PAUSE);
        drop(writer);
    });
    let reader = Database::open_with(tmp.path(), waiting(true)).unwrap();
    assert!(start.elapsed() >= PAUSE);
    assert_eq!(reader.get(b"k").unwrap(), Some(b"v".to_vec()));
    releaser.join().unwrap();
}

#[test]
fn reorganise_needs_the_file_to_itself() {
    let tmp = TempPath::new("reorg-locked.db");
    created(&tmp);
    let before = fs::read(tmp.path()).unwrap();
    let reader = Database::open_read_only(tmp.path()).unwrap();
    assert!(matches!(rdbm::reorganise(tmp.path()), Err(Error::Locked)));
    assert_eq!(fs::read(tmp.path()).unwrap(), before);

    // While it runs, nobody else can open the database.
    let releaser = thread::spawn(move || {
        thread::sleep(PAUSE);
        drop(reader);
    });
    rdbm::reorganise_with(tmp.path(), true).unwrap();
    releaser.join().unwrap();
    assert_eq!(
        Database::open(tmp.path()).unwrap().get(b"k").unwrap(),
        Some(b"v".to_vec())
    );
}

#[test]
fn a_waiter_follows_a_file_renamed_over_the_path() {
    // What reorganise does: rename a new file over the path while holding
    // the old file's lock. A handle already waiting on the old file must end
    // up with the new one, or its writes would go to the replaced file.
    let tmp = TempPath::new("renamed.db");
    let replacement = TempPath::new("replacement.db");
    created(&tmp);
    let holder = Database::open(tmp.path()).unwrap();

    let path = tmp.path().to_owned();
    let waiter = thread::spawn(move || {
        let mut db = Database::open_with(&path, waiting(false)).unwrap();
        let seen = db.get(b"new").unwrap();
        db.put(b"late", b"write").unwrap();
        seen
    });
    thread::sleep(PAUSE);
    {
        let mut db = Database::create(replacement.path(), CreateOptions::default()).unwrap();
        db.put(b"new", b"file").unwrap();
    }
    fs::rename(replacement.path(), tmp.path()).unwrap();
    drop(holder);

    assert_eq!(waiter.join().unwrap(), Some(b"file".to_vec()));
    let db = Database::open_read_only(tmp.path()).unwrap();
    assert_eq!(db.get(b"late").unwrap(), Some(b"write".to_vec()));
}
