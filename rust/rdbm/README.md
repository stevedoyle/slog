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
| `rdbm list FILE` | Print every key, one per line, in iteration order |
| `rdbm dump FILE` | Print every entry as `KEY<TAB>VALUE` |
| `rdbm firstkey FILE` | Print the first key |
| `rdbm nextkey FILE KEY` | Print the key after KEY; works even if KEY was deleted |
| `rdbm stats FILE` | Print statistics as `name=value` lines |
| `rdbm check FILE` | Verify the file's structure; prints `ok` |
| `rdbm reorganise FILE` | Rebuild the file to reclaim space left by deletes (`reorganize` also works) |

`rdbm --help` lists the commands, and `rdbm --version` prints the version.

Each command locks the file while it runs: readers share it, and a writer has it to itself. A command that finds the file locked by another process fails at once; put `--wait` (or `-w`) before the command to wait instead, for example in scripts that run commands in parallel: `rdbm --wait put test.db key3 "value three"`.

Output goes to stdout and errors to stderr, so the commands compose in pipelines, for example `rdbm list test.db | sort`. The exit status is 0 on success, 1 if the key was not found, iteration ended, or an error occurred, and 2 on a usage error.

`stats` shows how big the database is and whether a reorganise would help:

```console
$ rdbm stats test.db
entries=500
file_size=139264
buckets=32
overflow_pages=0
global_depth=5
fill_percent=12
free_pages=0
page_size=4096
pages=34
max_depth=24
directory_size=32
directory_pages=1
```

A low `fill_percent` or a large `free_pages` means `rdbm reorganise` will shrink the file.

By default, pages are 4096 bytes and the directory grows to at most 2^24 slots. Past that depth, buckets chain overflow pages instead of splitting. Every change is flushed to disk before the command exits.

## How it works

- **Extendible hashing.** A directory of `2^global_depth` slots maps the top bits of a key's hash to a bucket. A full bucket splits on its own, and the directory doubles only when needed, so the table never rehashes everything at once.
- **Single-file format.** The file holds a header page, the directory pages, and then bucket, overflow, and free pages. Records within a page are located through a slot array.
- **Iteration.** Keys come back in a fixed order, sorted by hash and then key bytes, so `nextkey` stays correct even when keys are deleted, inserted, or buckets split during a walk.
- **Reorganise.** Deletes never merge buckets or shrink the file. `reorganise` copies every record into a fresh file and atomically renames it over the original, keeping its owner, group, and permissions. Given a symlink, it rebuilds the file the link points to.
- **Locking.** Every open handle holds an `flock` lock, shared for readers and exclusive for writers and `reorganise`. A conflicting open fails with `Error::Locked`, or waits if asked to.
- **Hashing.** Keys are hashed with FNV-1a followed by MurmurHash3's `fmix64` finalizer. The finalizer spreads every input bit into the top bits that the directory uses.

## Performance

On an Apple M1 Pro, a lookup takes about 3 µs whether the database holds 10,000 or 1,000,000 keys. A reorganise takes about 5.4 µs per record. Run `cargo run --release --example bench -- /tmp` to measure on your machine. From the shell, each `rdbm` call costs a few milliseconds of process startup, and each write also waits for a disk flush.

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
