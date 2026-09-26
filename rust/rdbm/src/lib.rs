//! Building blocks for a Rust implementation of dbm/gdbmtool.

pub mod db;
pub mod error;
pub mod format;
pub mod hash;
pub mod hashtable;
mod pager;

pub use db::{CreateOptions, Database, Stats};
pub use error::{Error, Result};
pub use hashtable::HashTable;
