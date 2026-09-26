//! Rebuilds a database file from scratch to reclaim wasted space.
//!
//! Deletes never merge buckets, shrink the directory, or shorten the file,
//! so a database that has shrunk keeps its peak size. Reorganising copies
//! every record into a fresh database built with the same options, then
//! atomically renames it over the original.
//!
//! The copy is written to a temporary file next to the original, so the
//! rename stays within one filesystem. Until the rename, the original is
//! untouched; after it, the new file is complete and synced. A crash at any
//! point leaves one or the other, plus possibly a stale temporary file.

use std::fs;
use std::path::{Path, PathBuf};

use crate::db::Database;
use crate::error::{Error, Result};

/// Rebuilds the database at `path` in place.
///
/// Fails, leaving the original unchanged, if the database cannot be read,
/// if the temporary file already exists, or if the copy does not hold
/// exactly the original's records.
pub fn reorganise(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let old = Database::open_read_only(path)?;
    let temp = temp_path(path)?;
    let new = Database::create(&temp, old.options()).map_err(|e| match e {
        Error::Io(io) if io.kind() == std::io::ErrorKind::AlreadyExists => {
            Error::InvalidOption(format!(
                "{} exists: another reorganise is running, or one was interrupted \
                 (if so, remove it and retry)",
                temp.display()
            ))
        }
        e => e,
    })?;

    // From here on the temporary file is ours to clean up.
    let result = copy_into(&old, new).and_then(|()| {
        fs::set_permissions(&temp, fs::metadata(path)?.permissions())?;
        fs::rename(&temp, path)?;
        sync_parent(path)
    });
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// `dir/.name.reorganise`: hidden, and in the same directory as `path`.
fn temp_path(path: &Path) -> Result<PathBuf> {
    let name = path
        .file_name()
        .ok_or_else(|| Error::InvalidOption(format!("{} is not a file", path.display())))?;
    let mut temp = std::ffi::OsString::from(".");
    temp.push(name);
    temp.push(".reorganise");
    Ok(path.with_file_name(temp))
}

fn copy_into(old: &Database, mut new: Database) -> Result<()> {
    new.set_sync(false);
    let mut copied = 0u64;
    for entry in old.entries() {
        let (key, value) = entry?;
        new.put(&key, &value)?;
        copied += 1;
    }
    if copied != old.len() || new.len() != old.len() {
        return Err(Error::Corrupt(format!(
            "header counts {} entries, but {copied} were read and {} copied; \
             run `rdbm check` on the original",
            old.len(),
            new.len()
        )));
    }
    new.sync()
}

/// Flushes the rename itself, which lives in the parent directory.
fn sync_parent(path: &Path) -> Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
