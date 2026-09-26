//! Reorganise keeps the database's owner and group. Changing the owner needs
//! root, so this tests the group, which goes through the same `chown`.
//!
//! It runs `id -G`, so it lives in a test binary of its own: a spawned child
//! shares the parent's file locks until it execs, which could make a lock
//! held by another test linger.

mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, chown};
use std::process::Command;

use rdbm::{CreateOptions, Database};
use support::TempPath;

#[test]
fn reorganise_keeps_the_group() {
    let tmp = TempPath::new("group.db");
    {
        let mut db = Database::create(tmp.path(), CreateOptions::default()).unwrap();
        db.put(b"k", b"v").unwrap();
    }
    let created = fs::metadata(tmp.path()).unwrap().gid();
    let groups = Command::new("id").arg("-G").output().unwrap();
    let other = String::from_utf8_lossy(&groups.stdout)
        .split_whitespace()
        .filter_map(|g| g.parse::<u32>().ok())
        .find(|&g| g != created);
    let Some(other) = other else {
        eprintln!("skipped: the current user belongs to only one group");
        return;
    };
    chown(tmp.path(), None, Some(other)).unwrap();

    rdbm::reorganise(tmp.path()).unwrap();
    let after = fs::metadata(tmp.path()).unwrap();
    assert_eq!(after.gid(), other, "group was reset to {}", after.gid());
    assert_eq!(
        Database::open_read_only(tmp.path())
            .unwrap()
            .get(b"k")
            .unwrap(),
        Some(b"v".to_vec())
    );
}
