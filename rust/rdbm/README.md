# rdbm

A solution to the [DBM Coding Challenge](https://codingchallenges.substack.com/p/coding-challenge-138-dmb) in Rust: a dbm-style key-value store built on extendible hashing and stored in a single file, plus `rdbm`, a small `gdbmtool`-like command-line tool. It is not intended to be a production-quality database, but it is a good starting point for learning about hash tables, on-disk formats, and Rust.

## Build and test

```console
$ cargo build --release
$ cargo test
```

The binary is at `target/release/rdbm`. It runs on Unix only.

## Usage

```console
$ rdbm create test.db
$ rdbm put test.db key1 "value one"
$ rdbm put test.db key2 "value two"
$ rdbm get test.db key1
value one
$ rdbm delete test.db key1
$ rdbm get test.db key1
rdbm: key1: not found
$ rdbm count test.db
1
```

| Command | Effect |
|---------|--------|
| `rdbm create [--page-size N] [--max-depth N] FILE` | Create a database; fails if FILE exists |
| `rdbm put FILE KEY VALUE` | Insert or replace KEY |
| `rdbm get FILE KEY` | Print the value of KEY |
| `rdbm delete FILE KEY` | Remove KEY |
| `rdbm count FILE` | Print the number of entries |
| `rdbm list FILE` | Print every entry as `KEY<TAB>VALUE`, in iteration order |
| `rdbm firstkey FILE` | Print the first key |
| `rdbm nextkey FILE KEY` | Print the key after KEY; works even if KEY was deleted |
| `rdbm stats FILE` | Print file statistics as `name=value` lines |
| `rdbm check FILE` | Verify the file's structure; prints `ok` |

Output goes to stdout and errors to stderr, so the commands compose in pipelines, for example `rdbm list test.db | sort`. The exit status is 0 on success, 1 if the key was not found, iteration ended, or an error occurred, and 2 on a usage error.

By default, pages are 4096 bytes and the directory grows to at most 2^24 slots. Past that depth, buckets chain overflow pages instead of splitting. Every change is flushed to disk before the command exits.

## How it works

- **Extendible hashing.** A directory of `2^global_depth` slots maps the top bits of a key's hash to a bucket. A full bucket splits on its own, and the directory doubles only when needed, so the table never rehashes everything at once.
- **Single-file format.** The file holds a header page, the directory pages, and then bucket, overflow, and free pages. Records within a page are located through a slot array.
- **Iteration.** Keys come back in a fixed order, sorted by hash and then key bytes, so `nextkey` stays correct even when keys are deleted, inserted, or buckets split during a walk.
- **Hashing.** Keys are hashed with FNV-1a followed by MurmurHash3's `fmix64` finalizer. The finalizer spreads every input bit into the top bits that the directory uses.

See [docs/design.md](docs/design.md) for the full on-disk format with an annotated hex dump, the algorithms, the invariants, and the limitations.

## Library

The storage engine is a library crate that `rdbm` builds on:

```rust
use rdbm::{CreateOptions, Database};

let mut db = Database::create("test.db", CreateOptions::default())?;
db.put(b"key1", b"value one")?;
assert_eq!(db.get(b"key1")?, Some(b"value one".to_vec()));
```

`rdbm::HashTable` is an in-memory version of the same algorithm, and `examples/hashtable.rs` exercises it from stdin.
