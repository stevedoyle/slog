//! Measures how lookup, iteration, and reorganise times grow with size.
//!
//!   cargo run --release --example bench -- DIR [N...]
//!
//! For each N (default 1000 10000 100000 1000000), creates DIR/bench.db,
//! loads N keys with per-write sync off, then times:
//!
//!   - get of every key, in a shuffled order (hits)
//!   - get of N absent keys (misses)
//!   - a full walk with entries(), and one with first_key/next_key
//!   - reorganise of the full database
//!   - reorganise after deleting 90% of the keys, and the size it reclaims
//!
//! Reads are served from the operating system's page cache, so these are
//! CPU and system-call costs, not disk latency.

use std::path::Path;
use std::time::{Duration, Instant};

use rdbm::{CreateOptions, Database, reorganise};

fn key(i: u64) -> Vec<u8> {
    format!("key{i:08}").into_bytes()
}

fn value(i: u64) -> Vec<u8> {
    format!("value for key {i}, padded to a typical size").into_bytes()
}

/// A fixed pseudo-random permutation of 0..n (multiplicative, n-coprime).
fn shuffled(n: u64) -> impl Iterator<Item = u64> {
    let step = (0..)
        .map(|k| 2_654_435_761u64 + k)
        .find(|s| gcd(*s, n.max(1)) == 1)
        .unwrap_or(1);
    (0..n).map(move |i| (i * step) % n)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn per_op(elapsed: Duration, n: u64) -> String {
    format!("{:.2} us", elapsed.as_secs_f64() * 1e6 / n as f64)
}

fn bench(dir: &Path, n: u64) -> rdbm::Result<()> {
    let path = dir.join("bench.db");
    let _ = std::fs::remove_file(&path);

    let mut db = Database::create(&path, CreateOptions::default())?;
    db.set_sync(false);
    let t = Instant::now();
    for i in 0..n {
        db.put(&key(i), &value(i))?;
    }
    db.sync()?;
    let load = t.elapsed();
    let s = db.stats()?;

    let t = Instant::now();
    for i in shuffled(n) {
        assert_eq!(db.get(&key(i))?, Some(value(i)));
    }
    let hits = t.elapsed();

    let t = Instant::now();
    for i in n..2 * n {
        assert_eq!(db.get(&key(i))?, None);
    }
    let misses = t.elapsed();

    let t = Instant::now();
    let listed = db.entries().try_fold(0u64, |c, e| e.map(|_| c + 1))?;
    let entries = t.elapsed();
    assert_eq!(listed, n);

    let t = Instant::now();
    let mut walked = 0u64;
    let mut next = db.first_key()?;
    while let Some(k) = next {
        next = db.next_key(&k)?;
        walked += 1;
    }
    let walk = t.elapsed();
    assert_eq!(walked, n);

    drop(db);
    let t = Instant::now();
    reorganise(&path)?;
    let reorg_full = t.elapsed();

    let mut db = Database::open(&path)?;
    db.set_sync(false);
    for i in 0..n - n / 10 {
        db.delete(&key(i))?;
    }
    db.sync()?;
    let before = std::fs::metadata(&path)?.len();
    drop(db);
    let t = Instant::now();
    reorganise(&path)?;
    let reorg = t.elapsed();
    let after = std::fs::metadata(&path)?.len();
    std::fs::remove_file(&path)?;

    println!(
        "| {n:>9} | {:>5} | {:>10} | {:>9} | {:>9} | {:>9} | {:>7} | {:>8} | {:>9} | {:>9} | {:>9} | {:>9} -> {:<9} |",
        s.global_depth,
        format!("{} KiB", s.file_size / 1024),
        per_op(load, n),
        per_op(hits, n),
        per_op(misses, n),
        format!("{:.0} ms", entries.as_secs_f64() * 1e3),
        format!("{:.0} ms", walk.as_secs_f64() * 1e3),
        per_op(walk, n),
        format!("{:.0} ms", reorg_full.as_secs_f64() * 1e3),
        format!("{:.0} ms", reorg.as_secs_f64() * 1e3),
        format!("{} KiB", before / 1024),
        format!("{} KiB", after / 1024),
    );
    Ok(())
}

fn main() -> rdbm::Result<()> {
    let mut args = std::env::args().skip(1);
    let Some(dir) = args.next() else {
        eprintln!("usage: bench DIR [N...]");
        std::process::exit(2);
    };
    let mut sizes: Vec<u64> = args.filter_map(|a| a.parse().ok()).collect();
    if sizes.is_empty() {
        sizes = vec![1_000, 10_000, 100_000, 1_000_000];
    }
    println!(
        "| keys | depth | file | put | get hit | get miss | entries() | next_key walk | per next_key | reorganise all | reorganise after 90% deleted | file before -> after |"
    );
    println!("|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|");
    for n in sizes {
        bench(Path::new(&dir), n)?;
    }
    Ok(())
}
