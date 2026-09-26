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
//!
//! The original is held under an exclusive lock from before the copy until
//! after the rename, so no other handle can write to it and lose the write
//! to the replaced file.

use std::fs;
use std::io;
use std::os::unix::fs::{MetadataExt, chown};
use std::path::{Path, PathBuf};

use crate::db::{CreateOptions, Database};
use crate::error::{Error, Result};

/// Rebuilds the database at `path` in place, keeping its owner, group, and
/// permission bits. If `path` is a symlink, the file it points to is
/// rebuilt and the link is left alone.
///
/// Fails, leaving the original unchanged, if the database cannot be read,
/// if any other handle has it open ([`Error::Locked`]), if the temporary
/// file already exists, if the rebuilt file cannot be given the original's
/// owner and group, or if the copy does not hold exactly the original's
/// records.
///
/// One failure comes after the original has been replaced: if the
/// directory cannot be flushed, the error says the database was
/// reorganised, but a crash could still bring back the old file.
pub fn reorganise(path: impl AsRef<Path>) -> Result<()> {
    reorganise_with(path, false)
}

/// Like [`reorganise`], but if `wait` is set and another handle has the
/// database open, waits for it to be closed instead of failing.
pub fn reorganise_with(path: impl AsRef<Path>, wait: bool) -> Result<()> {
    // Renaming over a symlink would replace the link with a regular file
    // and leave its target stale, so work on the file it points to.
    let path = &fs::canonicalize(path.as_ref())?;
    // Held until the function returns, which is after the rename.
    let old = Database::open_read_only_exclusive(path, wait)?;
    let original = fs::metadata(path)?;
    let temp = temp_path(path)?;
    let new = create_temp(&temp, old.options())?;

    // Until the rename, the original is unchanged and the temporary file is
    // ours to clean up. Ownership comes first, so that a copy which could not
    // take the original's place is never made.
    let replaced = keep_owner(&temp, &original)
        .and_then(|()| copy_into(&old, new))
        .and_then(|()| {
            // After the chown, which may clear the set-user-ID and set-group-ID bits.
            fs::set_permissions(&temp, original.permissions())?;
            fs::rename(&temp, path)?;
            Ok(())
        });
    if replaced.is_err() {
        let _ = fs::remove_file(&temp);
        return replaced;
    }
    sync_parent(path).map_err(|e| {
        Error::Io(io::Error::new(
            e.kind(),
            format!(
                "the database was reorganised, but flushing its directory failed, \
                 so a crash could bring back the old file: {e}"
            ),
        ))
    })
}

/// Gives the temporary file the original's owner and group. Otherwise a
/// reorganise run by another user, such as root, would leave the database
/// owned by that user, and a file created in a directory with a different
/// group would take that group.
fn keep_owner(temp: &Path, original: &fs::Metadata) -> Result<()> {
    let created = fs::metadata(temp)?;
    let owner = (original.uid(), original.gid());
    if (created.uid(), created.gid()) == owner {
        return Ok(());
    }
    chown(temp, Some(owner.0), Some(owner.1)).map_err(|e| {
        Error::Io(io::Error::new(
            e.kind(),
            format!(
                "cannot give the rebuilt file the original's owner {} and group {}: {e}",
                owner.0, owner.1
            ),
        ))
    })
}

/// Creates the temporary database, owner-only until the copy is complete
/// and given the original's permissions, so that a private database is
/// never readable by others.
fn create_temp(temp: &Path, options: CreateOptions) -> Result<Database> {
    Database::create_with_mode(temp, options, 0o600).map_err(|e| match e {
        Error::Io(io) if io.kind() == std::io::ErrorKind::AlreadyExists => {
            Error::InvalidOption(format!(
                "{} exists: another reorganise is running, or one was interrupted \
                 (if so, remove it and retry)",
                temp.display()
            ))
        }
        e => e,
    })
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
fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    fs::File::open(parent)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn temp_file_is_created_owner_only() {
        let temp = std::env::temp_dir().join(format!(".rdbm-temp-mode-{}", std::process::id()));
        let _ = fs::remove_file(&temp);
        drop(create_temp(&temp, CreateOptions::default()).unwrap());
        let mode = fs::metadata(&temp).unwrap().permissions().mode();
        fs::remove_file(&temp).unwrap();
        assert_eq!(mode & 0o077, 0, "temp file mode is {mode:o}");
    }
}
