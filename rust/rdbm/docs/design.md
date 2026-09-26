# rdbm Design

## Purpose

rdbm is a Rust implementation of a dbm-style key-value store and a
`gdbmtool`-like command-line utility for inspecting and editing one. It is
built incrementally. This document describes the design as it stands and the
direction it is heading.

## Goals and non-goals

Goals:

- Store and retrieve arbitrary byte-string keys and values.
- Grow on demand, never rehashing the whole table at once.
- Keep each component small, pure where possible, and testable on its own.
- Expose functionality through text-stream tools that compose in pipelines.

Non-goals, for now:

- File-format compatibility with GNU gdbm or ndbm.
- Concurrent access from multiple threads or processes.
- Crash safety or transactions. Writes are flushed, but a crash in the
  middle of a multi-page update can leave the file inconsistent.

## Current status

| Step | Scope | Status |
|------|-------|--------|
| 1 | In-memory hash table: put, get, delete | Done |
| 2 | Replace it with extendible hashing | Done |
| 3 | Persist to a single file; `rdbm` CLI | Done |
| 4 | Iterate with `firstkey` and `nextkey` | Done |
| 5 | Reorganise: rebuild to reclaim space | Done |
| Later | Large values, locking, buffered writes | Not started |

## Crate layout

```text
rdbmtool/
├── Cargo.toml                  # package "rdbm": library + `rdbm` binary
├── src/
│   ├── lib.rs                  # re-exports the public API
│   ├── main.rs                 # `rdbm` command-line tool
│   ├── hash.rs                 # FNV-1a, fmix64, hash-bit helpers
│   ├── hashtable.rs            # in-memory extendible hash table
│   ├── db.rs                   # on-disk extendible hash database
│   ├── format.rs               # on-disk byte layout: pure encode/decode
│   ├── pager.rs                # positioned page reads and writes
│   ├── reorganise.rs           # rebuild a file to reclaim space
│   └── error.rs                # error type
├── tests/
│   ├── extendible_hashing.rs   # in-memory split, delete, reuse scenarios
│   ├── disk.rs                 # persistence, overflow, corruption
│   ├── iteration.rs            # first_key/next_key, mutation mid-iteration
│   ├── reorganise.rs           # shrinking, failure handling
│   ├── cli.rs                  # runs the `rdbm` binary
│   └── support/mod.rs          # temporary file helper
└── examples/
    └── hashtable.rs            # stdin-driven test program for HashTable
```

The storage engine is a library, so it can be tested without any I/O and
reused by more than one front-end. Command-line tools are thin layers that
parse text and call the library.

Each module in the storage engine does one job, and they depend in one
direction:

```text
main.rs ──► reorganise.rs ──► db.rs   (copy one database into another)
main.rs ──► db.rs ──► format.rs   (bytes ⇄ structs, no I/O)
              │  └──► pager.rs    (page I/O, no interpretation)
              └─────► hash.rs     (pure functions)
hashtable.rs ───────► hash.rs
```

`HashTable` is the in-memory form of the same algorithm. It has no I/O, so it
is the clearest statement of the algorithm and a reference for the disk
version.

## Extendible hash table

### Overview

Extendible hashing has two parts:

- **Directory.** An array of `2^global_depth` slots, each holding a bucket id.
  A key's slot is the top `global_depth` bits of its hash.
- **Buckets.** Each bucket holds at most `bucket_capacity` entries and has a
  `local_depth`. Every key in a bucket shares the same top `local_depth` hash
  bits, and `2^(global_depth - local_depth)` adjacent slots point at it.

A lookup is one directory index plus a scan of one bucket. Once the table is
on disk, that is typically one or two page reads.

### Public API

```rust
impl HashTable {
    pub fn new() -> Self;                                   // capacity 32
    pub fn with_bucket_capacity(capacity: NonZeroUsize) -> Self;

    pub fn put(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>>;
    pub fn get(&self, key: &[u8]) -> Option<&[u8]>;
    pub fn delete(&mut self, key: &[u8]) -> Option<Vec<u8>>;
    pub fn iter(&self) -> impl Iterator<Item = (&[u8], &[u8])>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    // Introspection, for tests and debugging tools
    pub fn global_depth(&self) -> u32;
    pub fn directory_len(&self) -> usize;
    pub fn bucket_count(&self) -> usize;
    pub fn bucket_capacity(&self) -> usize;
    pub fn bucket_of(&self, key: &[u8]) -> usize;
    pub fn check_consistency(&self) -> Result<(), String>;
}

// in rdbm::hash
pub fn key_hash(key: &[u8]) -> u64;   // placement hash
pub fn fnv1a(bytes: &[u8]) -> u64;    // raw FNV-1a
```

- `put` inserts a key, or replaces its value and returns the previous one.
  Replacing a value never splits a bucket.
- `get` borrows the stored value. It does not copy it.
- `delete` removes a key and returns its value, or `None` if it was absent.
- `iter` visits every pair once, in an unspecified order.
- `bucket_of` returns the id of the bucket a key belongs in, present or not.
- `check_consistency` checks every structural invariant and describes the
  first violation it finds.

Keys and values are copied into the table on `put`. Empty keys and values are
valid, and any byte may appear in either. Bucket capacity is a `NonZeroUsize`,
so a zero-capacity table cannot be constructed.

### Data model

```rust
struct Entry {
    hash: u64,        // cached key_hash(key)
    key: Vec<u8>,
    value: Vec<u8>,
}

struct Bucket {
    local_depth: u32,
    entries: Vec<Entry>,
}

pub struct HashTable {
    directory: Vec<usize>,   // 2^global_depth bucket ids
    buckets: Vec<Bucket>,    // indexed by bucket id
    global_depth: u32,
    bucket_capacity: usize,
    len: usize,
}
```

A split appends the new bucket to `buckets`, so bucket ids never change. On
disk, a bucket id will become a page address.

The table starts with `global_depth = 0`: a one-slot directory pointing at a
single empty bucket.

### Hash function

`key_hash(key) = fmix64(fnv1a(key))`.

**FNV-1a** is the base hash. It gives the same result on every run and every
machine, so a key maps to the same bucket when a file is reopened. Rust's
default `SipHash` uses random keys and does not give this.

**fmix64**, the 64-bit finalizer from MurmurHash3, is applied on top. FNV-1a
alone is unsuitable for indexing by the top bits of the hash. Its last step
multiplies by a prime whose set bits are all low, so the final input byte
barely reaches the high bits. Keys such as `key 1` and `key 2` share roughly
the first 20 hash bits.

This was found in testing. With raw FNV-1a and 4-entry buckets, the fifth
insert split 19 times in a row, and 64 keys produced a directory of about 4
million slots. With `fmix64`, the same 64 keys give a 64-slot directory.

`fmix64` is a bijection: distinct FNV hashes stay distinct. The overflow rule
below relies on this.

Neither function resists hash flooding. An attacker who chooses the keys can
still force deep splits and a large directory. This is acceptable for a local
database tool.

### Directory indexing

`prefix(hash, d)` is the top `d` bits of the hash, with `prefix(hash, 0) = 0`.
The directory slot for a key is `prefix(hash, global_depth)`.

Indexing by the most significant bits means that when the directory doubles,
old slot `i` becomes new slots `2i` and `2i + 1`. Both point at the same
bucket, so doubling is a straight copy of each slot twice.

### Insert and split

`put` works as follows:

1. Find the key's bucket. If the key is already there, replace the value and
   stop.
2. While the bucket is full and splitting could help (see below):
   1. If the bucket's `local_depth` equals `global_depth`, double the
      directory and increment `global_depth`.
   2. Increase the bucket's `local_depth` from `d` to `d + 1`. Partition its
      entries by hash bit `d`, counting from the most significant bit. Entries
      with a 0 stay; entries with a 1 move to a new bucket with the same local
      depth.
   3. The slots pointing at the old bucket form one aligned block. Point the
      upper half of that block (next bit set) at the new bucket.
   4. Look up the key's bucket again.
3. Append the entry.

One insert can split several times if every entry lands on the same side of
the new bit. Only the full bucket is ever touched; other buckets are never
rehashed. Because each entry caches its hash, a split never recomputes one.

Step 2.3 scans the whole directory. That is O(directory) per split, which is
simple and cheap in memory. The block's start and length could be computed
directly if this becomes a hotspot on disk.

### Overflow

If every entry in a full bucket, plus the incoming key, has the same 64-bit
hash, no bit can separate them and splitting would never end. In that case,
or when `local_depth` has reached 64, the entry is appended anyway and the
bucket exceeds its capacity. Since `fmix64` is a bijection, this requires a
64-bit FNV-1a collision among more than `bucket_capacity` keys, which in
practice does not happen.

### Delete

`delete` removes the entry with `swap_remove`. Order within a bucket is not
meaningful, so nothing is lost.

Buckets are never merged and the directory never shrinks. The freed slot is
reused by the next insert into that bucket, so deleting and re-inserting the
same keys causes no splits. This matches gdbm, which also does not coalesce
buckets.

### Invariants

`check_consistency` verifies all of the following:

- The directory has exactly `2^global_depth` slots.
- Every slot points at an existing bucket.
- For each bucket with local depth `d`:
  - `d <= global_depth`
  - exactly `2^(global_depth - d)` slots point at it, forming one aligned block
  - every entry's cached hash equals `key_hash(key)`
  - every entry's `prefix(hash, d)` matches the bucket's block
  - no key appears twice
  - it holds at most `bucket_capacity` entries, unless it is overflowing with
    identical hashes
- The bucket sizes add up to `len`.

### Complexity

With `n` keys, bucket capacity `b`, and directory size `D`:

| Operation | Average | Worst case |
|-----------|---------|------------|
| `get` | O(b) | O(b), or O(n) in an overflowing bucket |
| `put`, no split | O(b) | O(b) |
| `put`, split | O(b + D) | O(64 · (b + D)): one split per shared hash bit |
| `delete` | O(b) | O(b) |
| `iter` | O(n + buckets) | O(n + buckets) |

With a well-mixed hash, `D` stays within a small factor of `n / b`.

## On-disk database

`Database` in `src/db.rs` is the persistent version of the extendible hash
table. It uses the same hashing, directory indexing, and split rule, and
stores everything in one file.

### Why a single file

The original dbm used two files: `.dir` for the directory and `.pag` for
buckets. gdbm and Berkeley DB use one. A single file was chosen because:

- Creating, copying, or deleting a database is one operation on one path.
- The directory and the buckets can never be mismatched by copying one file
  without the other.
- One page allocator serves the directory and the buckets. When the directory
  moves, its old pages become free pages for buckets.

### File layout

The file is an array of fixed-size pages, numbered from 0. All integers are
little-endian. Page 0 is always the header, so page number 0 also means
"none" in any pointer field.

```text
page 0          header
dir_page ..     directory: 2^global_depth page numbers (u64)
other pages     bucket, overflow, and free pages, in any order
```

A new database has 3 pages: the header, a one-slot directory, and one empty
bucket. The file length is always exactly `page_count × page_size`, and
opening a file whose length disagrees fails.

#### Header

The first 56 bytes of page 0. The rest of the page is zero.

| Offset | Size | Field |
|--------|------|-------|
| 0 | 4 | Magic `RDBM` |
| 4 | 4 | Format version, currently 1 |
| 8 | 4 | Page size in bytes |
| 12 | 1 | Global depth |
| 13 | 1 | Max depth |
| 14 | 2 | Reserved, 0 |
| 16 | 8 | First directory page |
| 24 | 8 | Number of directory pages |
| 32 | 8 | Total number of pages in the file |
| 40 | 8 | First free page, 0 if none |
| 48 | 8 | Number of entries |

The page size and max depth are fixed when the database is created. The page
size must be a power of two from 512 to 32768 (default 4096). The max depth
may be 0 to 32 (default 24).

#### Directory

`2^global_depth` little-endian `u64` page numbers in consecutive pages,
padded with zeros to a whole page. Slot `i` holds the bucket page for keys
whose hash starts with the bits of `i`.

#### Bucket, overflow, and free pages

All three kinds share one layout: a 16-byte header, then a slot array growing
forward, then free space, then record bytes packed backward from the end of
the page.

```text
+--------+--------+--------+-----+------------+---------+---------+
| header | slot 0 | slot 1 | ... | free space | rec 1   | rec 0   |
+--------+--------+--------+-----+------------+---------+---------+
0        16       32                                     page_size
```

Page header:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 1 | Kind: 1 bucket, 2 overflow, 3 free |
| 1 | 1 | Local depth of the bucket this page belongs to (0 for free) |
| 2 | 2 | Number of records |
| 4 | 4 | Reserved, 0 |
| 8 | 8 | Next page: the next overflow page, or the next free page. 0 if none |

Slot, one per record:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 8 | Hash of the key |
| 8 | 2 | Offset of the record within the page |
| 10 | 2 | Key length |
| 12 | 2 | Value length |
| 14 | 2 | Reserved, 0 |

A record is the key bytes followed immediately by the value bytes. A record
costs 16 bytes of slot plus its key and value, so one page holds at most
`page_size − 32` bytes of key and value. A larger record is rejected with
`Error::TooLarge` before anything is written.

Storing the hash in the slot lets a bucket split without rehashing keys, and
lets `check` detect a damaged key. The 32 KiB maximum page size keeps every
offset and length within 16 bits.

### Buckets and overflow chains

A bucket is a chain of pages: one bucket page (kind 1) followed by zero or
more overflow pages (kind 2), linked by the next field. Every page in a
chain records the bucket's local depth.

An insert that no longer fits in a single page splits the bucket, as in the
in-memory table. The chain grows overflow pages only when a split cannot
help:

- the bucket is already at the max depth, or
- every record in the bucket, including the new one, has the same hash.

The max depth caps the directory: at 24 it is 16 Mi slots, or 128 MiB. Past
that point buckets grow by chaining instead of doubling the directory
further. A small max depth makes overflow easy to trigger in tests.

When a chain is written, its records are repacked in order into as few pages
as they need. Pages whose bytes did not change are not rewritten. Overflow
pages that are no longer needed go on the free list; extra pages come from
the free list first, then from the end of the file.

### Operations

The directory is read into memory when the database is opened and is written
through on every change. Bucket pages are never cached.

- **get** reads the key's bucket page. It follows overflow pages only if the
  key is not found and the chain continues. Without overflow, a lookup is one
  page read.
- **put** reads the key's chain and works out whether the result fits in one
  page. If it fits, or splitting cannot help, it rewrites the chain. If not,
  it splits the bucket and tries again.
- **delete** reads the chain, removes the record, and rewrites the chain,
  which may free overflow pages. Buckets are never merged and the directory
  never shrinks, as in gdbm.

A split writes, in this order: the directory, if it doubled; the new bucket;
the directory slots that now point at it; and the old bucket without the
moved records.

When the doubled directory no longer fits in its pages, it is written to new
pages at the end of the file, the header is updated to point at it, and the
old directory pages go on the free list.

Every mutation ends by writing the header, which holds the entry count, page
count, and free list head.

### Iteration

`first_key()` returns the first key, and `next_key(key)` returns the key
after `key`, following dbm's `firstkey` and `nextkey`. `entries()`, and
therefore `rdbm list`, returns records in the same order.

**Order.** Keys are ordered by `(key_hash(key), key bytes)`. The key bytes
only break ties between keys whose hashes are identical. Each bucket owns
one run of directory slots, which is a contiguous range of hash prefixes, so
walking the directory from slot 0 visits buckets in hash order. The order
looks random, as expected for a hash-based store, but it is a fixed total
order.

**`next_key(key)`** returns the smallest key ordered after `key`:

1. Start at the directory slot for `key`'s hash. Earlier buckets hold only
   keys that sort before it.
2. Read that bucket's chain and pick the smallest record ordered after
   `key`.
3. If there is none, skip to the first slot of the next bucket and repeat.
   In that bucket, every key sorts after `key`, so the result is its
   smallest record.

`key` does not need to be present. The iterator keeps no cursor state
between calls: the key itself is the position. That is what dbm's interface
requires, since `nextkey` receives only a key.

**Mutation during iteration.** Because the position is a point in a fixed
order, not a slot in a page, iteration stays well-defined while the database
changes:

| Change mid-iteration | Effect |
|----------------------|--------|
| Delete the current key | Safe. `next_key` on the deleted key resumes correctly. |
| Delete any other key | That key is not returned, unless it already was. |
| Insert a key | Returned if it sorts after the current position, otherwise not. |
| A bucket splits or the directory doubles | No effect. The order does not depend on the bucket layout. |

Every key present for the whole iteration is returned exactly once.

**Why no tombstones.** The original dbm marked deleted records as empty
slots, so that positions within a bucket stayed stable during iteration.
Here, iteration never depends on a record's position, so `delete` removes the
record and compacts the page. Freed space is reused immediately, and pages
never fill up with dead slots.

**Cost.** `next_key` reads one bucket chain, or more if it must skip past
empty buckets. It sorts nothing: finding the minimum is a linear scan. A full
walk with `next_key` therefore reads each chain about once per key in it.
`entries()` reads each chain once and sorts its records, so `rdbm list`
uses it.

### Reorganise

`rdbm::reorganise(path)` (`rdbm reorganise FILE`) rebuilds a database from
scratch.

**What it reclaims.** Deletes already remove records and compact their page,
so the rebuild never finds empty slots. The waste it reclaims is structural:

- Buckets never merge. After most keys are deleted, the file keeps its peak
  bucket count, with most buckets nearly empty.
- The directory never shrinks, so it stays at its peak global depth.
- Free pages go on the free list, but the file never gets shorter.
- Overflow chains built up at max depth stay longer than the remaining
  records need.

For example, inserting 3,000 keys with 4 KiB pages and then deleting 2,900
leaves 40 buckets, global depth 6, and 168 KiB. Reorganising gives 2
buckets, global depth 1, and 16 KiB.

**How it works.**

1. Open the original read-only.
2. Create `.NAME.reorganise` in the same directory, with the same page size
   and max depth. It must not already exist.
3. Insert every record from `entries()`, with per-write sync off.
4. Check that the number of records read and the new database's count both
   equal the original's header count. If not, fail, pointing the user to
   `rdbm check`.
5. Sync the new file and copy the original's permission bits onto it.
6. `rename` it over the original, then sync the parent directory so the
   rename itself is durable.

On any failure the temporary file is removed and the original is unchanged.
If the temporary file already exists, reorganise fails without touching it:
another reorganise may be running, or an earlier one was interrupted.

**Crash safety.** The original is never modified. Until the rename, it is
intact; after the rename, the new file is complete and synced. `rename` is
atomic on POSIX, so a crash at any point leaves one database or the other,
never a mix, plus possibly a stale temporary file.

**Concurrency.** Without locking, a process that has the database open
across a reorganise keeps reading the old file. If it writes, its writes go
to the old file and are lost. Reorganise when nothing else is using the
database.

**Determinism.** Records are inserted in iteration order and the hash is
fixed, so reorganising an already reorganised file produces identical bytes.

### Durability

By default each mutation calls `sync_data` after writing its pages and
header, so a completed `put` or `delete` is on stable storage.
`Database::set_sync(false)` skips this. Writes still reach the operating
system right away and survive a process crash, but they can be lost in a
power failure or OS crash until `Database::sync` is called.

Measured on macOS, where `sync_data` does a full flush to the physical disk
(`F_FULLFSYNC`), with 4 KiB pages and 2,000 puts:

| Mode | Time per put |
|------|--------------|
| Sync every write (default) | ~4.9 ms |
| `set_sync(false)` | ~15 µs |

Neither mode is crash-atomic. A crash in the middle of a multi-page write,
such as a split, can leave the file inconsistent. `check` detects most such
damage, but nothing repairs it yet.

### Consistency check

`Database::check` (`rdbm check`) walks the whole file and verifies that:

- every page is exactly one of: the header, a directory page, a bucket page,
  an overflow page, or a free page, and none are unreferenced
- each bucket is referenced by one aligned run of `2^(global − local)`
  directory slots
- every page in a chain has the right kind and local depth, and chains and
  the free list do not loop
- every record's stored hash matches its key, its hash prefix matches its
  bucket, and no key appears twice
- no bucket has overflow pages when it could have split instead
- the header's entry count matches the records

Opening a database checks less but costs less: the magic, version, page size,
depths, file length, directory location, and that every directory slot is a
valid page number.

### Limitations

- Records larger than one page are rejected. gdbm supports arbitrary sizes;
  that would need a separate kind of large-value page.
- There is no file locking. Two processes writing the same file at once will
  corrupt it.
- Only Unix is supported, because page I/O uses
  `std::os::unix::fs::FileExt`.

## Command-line tool

`rdbm` (`src/main.rs`) performs one operation per run on a database file.

| Command | Effect | Output |
|---------|--------|--------|
| `rdbm create [--page-size N] [--max-depth N] FILE` | Create a database; fails if FILE exists | None |
| `rdbm put FILE KEY VALUE` | Insert or replace KEY | None |
| `rdbm get FILE KEY` | Look up KEY | The value and a newline |
| `rdbm delete FILE KEY` | Remove KEY | None |
| `rdbm count FILE` | Count entries | The number |
| `rdbm list FILE` | Show every entry, in iteration order | `KEY<TAB>VALUE` per line |
| `rdbm firstkey FILE` | First key in iteration order | The key |
| `rdbm nextkey FILE KEY` | Key after KEY in iteration order | The key |
| `rdbm stats FILE` | Show file statistics | `name=value` per line |
| `rdbm check FILE` | Verify the file | `ok` |
| `rdbm reorganise FILE` | Rebuild the file to reclaim space; `reorganize` also works | None |

Results go to stdout, diagnostics to stderr, prefixed `rdbm:`. The exit
status is 0 on success, 1 if the key was not found, iteration ended, or an
error occurred, and 2 on a usage error. At the end of iteration, `firstkey`
and `nextkey` exit 1 without printing anything, so they can drive a shell
loop:

```sh
key=$(rdbm firstkey db) && while :; do
    echo "$key"; key=$(rdbm nextkey db "$key") || break
done
```

Read-only commands open the file read-only. If the reader closes the pipe,
as in `rdbm list db | head`, rdbm exits quietly.

```console
$ rdbm create test.db
$ rdbm put test.db key1 "value one"
$ rdbm put test.db key2 "value two"
$ rdbm get test.db key1
value one
$ rdbm delete test.db key1
$ rdbm get test.db key1
rdbm: key1: not found
$ rdbm list test.db | sort
key2	value two
```

**Subcommands vs. separate tools.** The design philosophy prefers many small
executables. `rdbm` uses subcommands instead, because the operations share
all of their setup (opening and validating the file), the target is a
drop-in for `gdbmtool` and `ccdbm`-style usage, and one binary keeps
installation simple. Each subcommand still does one thing and composes
through stdout and exit codes.

`list` is not safe for keys or values containing tabs or newlines. A later
dump format should escape them.

### Inspecting a file with xxd

A database with 512-byte pages holding `key1` and `key2`:

```console
$ rdbm create --page-size 512 hex.db
$ rdbm put hex.db key1 "value one"
$ rdbm put hex.db key2 "value two"
$ xxd -a hex.db
00000000: 5244 424d 0100 0000 0002 0000 0018 0000  RDBM............
00000010: 0100 0000 0000 0000 0100 0000 0000 0000  ................
00000020: 0300 0000 0000 0000 0000 0000 0000 0000  ................
00000030: 0200 0000 0000 0000 0000 0000 0000 0000  ................
00000040: 0000 0000 0000 0000 0000 0000 0000 0000  ................
*
00000200: 0200 0000 0000 0000 0000 0000 0000 0000  ................
00000210: 0000 0000 0000 0000 0000 0000 0000 0000  ................
*
00000400: 0100 0200 0000 0000 0000 0000 0000 0000  ................
00000410: b877 6e53 d745 e1dd f301 0400 0900 0000  .wnS.E..........
00000420: b230 2761 6106 6ad8 e601 0400 0900 0000  .0'aa.j.........
00000430: 0000 0000 0000 0000 0000 0000 0000 0000  ................
*
000005e0: 0000 0000 0000 6b65 7932 7661 6c75 6520  ......key2value 
000005f0: 7477 6f6b 6579 3176 616c 7565 206f 6e65  twokey1value one
```

- `0x000`: magic `RDBM`, version 1, page size `0x200` (512), global depth 0,
  max depth `0x18` (24).
- `0x010`: the directory is at page 1 and is 1 page long; the file has 3
  pages; there are no free pages; there are 2 entries.
- `0x200`, page 1: the directory. Its one slot points at page 2.
- `0x400`, page 2: a bucket page (kind 1) at depth 0 with 2 records and no
  overflow page.
- `0x410`: slot 0 holds the hash of `key1`, offset `0x1f3` (499), key length
  4, and value length 9.
- `0x5f3`: `key1value one`, packed against the end of the page. `key2`'s
  record sits just before it, at offset `0x1e6`.

## Test program

`examples/hashtable.rs` reads one command per line from stdin and writes
results to stdout.

| Command | Effect | Output |
|---------|--------|--------|
| `put KEY VALUE` | Store VALUE under KEY | None |
| `get KEY` | Look up KEY | The value, or `KEY: not found` on stderr |
| `delete KEY` | Remove KEY | None, or `KEY: not found` on stderr |
| `count` | Count entries | The number of entries |
| `list` | Show all entries | `KEY VALUE` per line, sorted by key |
| `stats` | Show table shape | `global depth=G, buckets=B, directory size=D` |

Blank lines and lines starting with `#` are ignored. Errors are reported on
stderr as `hashtable: line N: MESSAGE`, and processing continues. The exit
status is 1 if any command failed, and 0 otherwise.

```console
$ { for i in $(seq 1 200); do echo "put k$i v$i"; done; echo stats; echo "get k137"; } \
    | cargo run -q --example hashtable
global depth=4, buckets=9, directory size=16
v137
```

Limitations of this program, not of the library:

- Keys cannot contain spaces, because the first space ends the key.
- Input must be valid UTF-8, because lines are read as Rust `String`s.

It is an example program rather than an installed binary. For the persistent
store, use `rdbm`.

## Testing

Run everything with `cargo test`.

Unit tests in `src/hash.rs` cover:

- FNV-1a against known values
- `key_hash` spreading the last byte of a key into the top bits
- `fmix64` producing no collisions on a sample
- `prefix` and `bit` selecting the most significant bits

Unit tests in `src/format.rs` cover:

- the header round trip, and rejecting a bad magic number, version, page
  size, or depth
- the page round trip, including an exactly full page and a free page
- records packed against the end of the page at the expected offsets
- rejecting an overfull page on encode
- rejecting on decode an unknown kind, a slot array overrunning the page, a
  record outside the record area, and a free page with records
- the directory round trip with padding
- `pack` filling pages in order, and returning one empty page for no records

Unit tests in `src/hashtable.rs` cover:

- insert, replace, and delete, including deleting a missing key
- replacing a value in a full bucket without splitting
- empty and binary keys and values
- the first overflow splitting the only bucket
- a split below global depth leaving the directory alone
- 10,000 keys in 4-entry buckets, then deleting half of them
- the overflow rule for identical hashes
- `iter` returning every entry exactly once
- `check_consistency` catching a key in the wrong bucket

Integration tests in `tests/extendible_hashing.rs` use 4-entry buckets and
64 keys:

- **Splits and retrieval.** After each insert, check consistency and print the
  directory state. Then require at least 3 splits and retrieve every key.
- **Delete and reuse.**
  1. Delete every fifth key, and confirm those keys are gone and the rest
     remain.
  2. Re-insert the deleted keys. Confirm each lands back in its original
     bucket and the directory shape is unchanged (no splits).
  3. Insert 20 new keys, and check consistency and retrieval.

Integration tests in `tests/disk.rs` close and reopen the file before
checking, so the results must come from disk:

- keys survive a reopen, and a deleted key stays gone after one
- replacing a value with a larger and then a smaller one
- binary and empty keys and values
- 3,000 keys in 512-byte pages: over 100 buckets, a directory relocated to
  several pages, every key retrievable, and `check` passing
- overflow with max depth 2 and 400 keys:
  1. Insert. Expect more than 10 overflow pages and every key retrievable.
  2. Delete everything. Every overflow page goes on the free list.
  3. Re-insert. The file does not grow and the free list is used up.
- max depth 0: one bucket with an overflow chain
- `entries` visiting every pair once
- an oversized record rejected without changing the file
- `create` refusing to overwrite a file, and rejecting bad options
- `open` rejecting a missing, empty, non-database, or truncated file
- `check` detecting a key byte flipped on disk
- a read-only handle rejecting writes

Integration tests in `tests/iteration.rs` cover:

- an empty database
- `apple`, `banana`, and `cherry` each returned exactly once after a reopen
- 1,000 keys returned in strictly increasing `(hash, key)` order, the same
  order as `entries`
- a deleted key no longer appearing
- `next_key` resuming from a deleted key and from a key never inserted
- deleting each key as it is visited: all 500 are still visited
- inserting 3 keys at every step of a 300-key walk, which more than doubles
  the bucket count: every original key is visited once and no key twice
- keys inserted into space freed by deletes, with no new pages, appearing
  after a reopen
- walking across overflow chains at max depth 0 and 2

Integration tests in `tests/reorganise.rs` cover:

- 3,000 keys inserted and 2,900 deleted: the file shrinks to under a quarter
  of its size, with fewer buckets, a smaller global depth, no free pages,
  every remaining key retrievable, and `check` passing
- the three-key scenario: the file is already minimal, so it does not grow,
  and the remaining key survives
- max depth 2: free pages dropped, overflow not increased, page size and max
  depth preserved
- reorganising twice produces byte-identical files
- an emptied database rebuilds to the minimum 3 pages
- file permissions preserved
- a header whose entry count disagrees with the buckets: `Error::Corrupt`,
  with the original byte-identical and no temporary file left
- an existing temporary file left untouched
- a missing database

Bulk tests turn per-write sync off and call `sync` once at the end. With it
on, each insert would cost several milliseconds.

`tests/cli.rs` runs the `rdbm` binary. It covers the create, put, and get
scenario across separate runs; delete followed by get failing; `list` output;
`create` options showing in `stats`; a `firstkey`/`nextkey` walk matching
`list`, including a quiet exit 1 at the end; `reorganise` keeping the
remaining keys, printing nothing, and accepting `reorganize`; and the exit status for usage
errors and other failures.

To see the directory trace:

```console
$ cargo test --test extendible_hashing -- --nocapture
Inserting key  1:  global depth=0, buckets=1, directory size=1
...
Inserting key  5:  global depth=1, buckets=2, directory size=2  <- split
Inserting key  6:  global depth=2, buckets=3, directory size=4  <- split
...
Inserting key 10:  global depth=3, buckets=4, directory size=8  <- split
...
23 splits, final state: global depth=6, buckets=24, directory size=64
```

## Future work

- **Large values.** Store records larger than a page in dedicated pages,
  referenced from the slot.
- **Locking.** Take an `flock` lock: shared for readers, exclusive for
  writers.
- **Buffered writes.** Cache dirty pages and write them in one batch on
  `sync`, instead of on every mutation.
- **Crash safety.** A write-ahead log or copy-on-write pages would make each
  mutation atomic.
- **Reorganise options.** Change the page size or max depth while
  rebuilding.
- **Import and export.** A text dump that escapes arbitrary bytes, for
  `rdbm dump` and `rdbm load`.
