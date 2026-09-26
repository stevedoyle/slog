//! Exercise the in-memory hash table from a line-oriented command stream.
//!
//! Reads commands from stdin, one per line, and writes results to stdout:
//!
//!   put KEY VALUE   store VALUE under KEY (VALUE may contain spaces)
//!   get KEY         print the value, or report "not found" on stderr
//!   delete KEY      remove KEY, or report "not found" on stderr
//!   count           print the number of entries
//!   list            print every "KEY VALUE" pair, sorted by key
//!   stats           print global depth, bucket count, and directory size
//!
//! Example:
//!   printf 'put a 1\nget a\ndelete a\ncount\n' | cargo run -q --example hashtable

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use rdbm::HashTable;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let mut out = io::stdout().lock();
    let mut table = HashTable::new();
    let mut failed = false;

    for (lineno, line) in stdin.lock().lines().enumerate() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("hashtable: read error: {e}");
                return ExitCode::FAILURE;
            }
        };
        if let Err(msg) = run(&mut table, &line, &mut out) {
            eprintln!("hashtable: line {}: {msg}", lineno + 1);
            failed = true;
        }
    }

    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn run(table: &mut HashTable, line: &str, out: &mut impl Write) -> Result<(), String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(());
    }
    let (cmd, rest) = line.split_once(' ').unwrap_or((line, ""));
    let io_err = |e: io::Error| format!("write error: {e}");

    match cmd {
        "put" => {
            let (key, value) = rest.split_once(' ').ok_or("usage: put KEY VALUE")?;
            table.put(key.as_bytes(), value.as_bytes());
        }
        "get" => {
            let value = table
                .get(rest.as_bytes())
                .ok_or(format!("{rest}: not found"))?;
            out.write_all(value).map_err(io_err)?;
            writeln!(out).map_err(io_err)?;
        }
        "delete" => {
            table
                .delete(rest.as_bytes())
                .ok_or(format!("{rest}: not found"))?;
        }
        "count" => writeln!(out, "{}", table.len()).map_err(io_err)?,
        "stats" => writeln!(
            out,
            "global depth={}, buckets={}, directory size={}",
            table.global_depth(),
            table.bucket_count(),
            table.directory_len()
        )
        .map_err(io_err)?,
        "list" => {
            let mut pairs: Vec<_> = table.iter().collect();
            pairs.sort_unstable();
            for (k, v) in pairs {
                out.write_all(k).map_err(io_err)?;
                out.write_all(b" ").map_err(io_err)?;
                out.write_all(v).map_err(io_err)?;
                writeln!(out).map_err(io_err)?;
            }
        }
        _ => return Err(format!("unknown command: {cmd}")),
    }
    Ok(())
}
