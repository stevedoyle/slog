//! Helpers shared by integration tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A unique path in the system temp directory, removed on drop.
pub struct TempPath(PathBuf);

impl TempPath {
    pub fn new(name: &str) -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let file = format!("rdbm-test-{}-{n}-{name}", std::process::id());
        let path = std::env::temp_dir().join(file);
        let _ = std::fs::remove_file(&path);
        TempPath(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}
