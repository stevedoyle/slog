//! rdbm: create, query, and modify an rdbm database file.
//!
//! Each invocation performs one operation and exits, so it composes with
//! shell pipelines and scripts. Results go to stdout, diagnostics to stderr.
//!
//! Run `rdbm --help` for the list of commands.
//!
//! Exit status: 0 on success, 1 if the key was not found, iteration ended,
//! or an error occurred, 2 on a usage error.
//!
//! `firstkey` and `nextkey` walk the keys in hash order and exit 1 without a
//! message at the end, so they drive a shell loop:
//!
//!   key=$(rdbm firstkey db) && while :; do
//!       echo "$key"; key=$(rdbm nextkey db "$key") || break
//!   done

use std::io::{self, Write};
use std::process::ExitCode;

use rdbm::{CreateOptions, Database};

const HELP: &str = "\
usage: rdbm COMMAND [ARGS]

Commands:
  create [--page-size N] [--max-depth N] FILE
                     Create an empty database; fails if FILE exists
  put FILE KEY VALUE Insert KEY, or update its value
  get FILE KEY       Print the value of KEY
  delete FILE KEY    Delete KEY
  list FILE          Print every key, one per line
  dump FILE          Print every entry as KEY<TAB>VALUE
  count FILE         Print the number of keys
  firstkey FILE      Print the first key in iteration order
  nextkey FILE KEY   Print the key after KEY in iteration order
  stats FILE         Print statistics as name=value lines
  check FILE         Verify the file's structure
  reorganise FILE    Rebuild the file to reclaim space (also: reorganize)

Options:
  -h, --help         Print this help
  -V, --version      Print the version

Exit status: 0 on success; 1 if the key was not found, iteration ended, or
an error occurred; 2 on a usage error.";

enum Failure {
    Usage(String),
    NotFound(String),
    /// Iteration reached the end; exit 1 without a message.
    End,
    Error(String),
    /// Stdout was closed by the reader, as in `rdbm list db | head`.
    BrokenPipe,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(Failure::Usage(msg)) => {
            if !msg.is_empty() {
                eprintln!("rdbm: {msg}");
            }
            eprintln!("{HELP}");
            ExitCode::from(2)
        }
        Err(Failure::NotFound(msg) | Failure::Error(msg)) => {
            eprintln!("rdbm: {msg}");
            ExitCode::FAILURE
        }
        Err(Failure::End | Failure::BrokenPipe) => ExitCode::FAILURE,
    }
}

fn run(args: &[String]) -> Result<(), Failure> {
    let Some((command, rest)) = args.split_first() else {
        return Err(Failure::Usage(String::new()));
    };
    let rest: Vec<&str> = rest.iter().map(String::as_str).collect();
    match (command.as_str(), rest.as_slice()) {
        ("-h" | "--help" | "help", []) => write_line(&[HELP.as_bytes()]),
        ("-V" | "--version", []) => {
            write_line(&[concat!("rdbm ", env!("CARGO_PKG_VERSION")).as_bytes()])
        }
        ("create", _) => create(&rest),
        ("put", [file, key, value]) => {
            let mut db = open(file)?;
            db.put(key.as_bytes(), value.as_bytes())
                .map_err(|e| failed(file, e))?;
            Ok(())
        }
        ("get", [file, key]) => {
            let db = open_read_only(file)?;
            let value = db
                .get(key.as_bytes())
                .map_err(|e| failed(file, e))?
                .ok_or_else(|| Failure::NotFound(format!("{key}: not found")))?;
            write_line(&[&value])
        }
        ("delete", [file, key]) => {
            let mut db = open(file)?;
            db.delete(key.as_bytes())
                .map_err(|e| failed(file, e))?
                .ok_or_else(|| Failure::NotFound(format!("{key}: not found")))?;
            Ok(())
        }
        ("firstkey", [file]) => {
            let db = open_read_only(file)?;
            let key = db.first_key().map_err(|e| failed(file, e))?;
            write_line(&[&key.ok_or(Failure::End)?])
        }
        ("nextkey", [file, key]) => {
            let db = open_read_only(file)?;
            let next = db.next_key(key.as_bytes()).map_err(|e| failed(file, e))?;
            write_line(&[&next.ok_or(Failure::End)?])
        }
        ("count", [file]) => {
            let db = open_read_only(file)?;
            write_line(&[db.len().to_string().as_bytes()])
        }
        ("list", [file]) => {
            let db = open_read_only(file)?;
            let mut out = io::stdout().lock();
            for entry in db.entries() {
                let (key, _) = entry.map_err(|e| failed(file, e))?;
                write_to(&mut out, &[&key])?;
            }
            Ok(())
        }
        ("dump", [file]) => {
            let db = open_read_only(file)?;
            let mut out = io::stdout().lock();
            for entry in db.entries() {
                let (key, value) = entry.map_err(|e| failed(file, e))?;
                write_to(&mut out, &[&key, b"\t", &value])?;
            }
            Ok(())
        }
        ("stats", [file]) => {
            let db = open_read_only(file)?;
            let s = db.stats().map_err(|e| failed(file, e))?;
            // The first lines answer "how big is it, and does it need a
            // reorganise?"; the rest describe the file's structure.
            let text = format!(
                "entries={}\nfile_size={}\nbuckets={}\noverflow_pages={}\n\
                 global_depth={}\nfill_percent={}\nfree_pages={}\n\
                 page_size={}\npages={}\nmax_depth={}\n\
                 directory_size={}\ndirectory_pages={}",
                s.entries,
                s.file_size,
                s.buckets,
                s.overflow_pages,
                s.global_depth,
                s.fill_percent(),
                s.free_pages,
                s.page_size,
                s.pages,
                s.max_depth,
                s.directory_size,
                s.directory_pages
            );
            write_line(&[text.as_bytes()])
        }
        ("reorganise" | "reorganize", [file]) => {
            rdbm::reorganise(file).map_err(|e| failed(file, e))
        }
        ("check", [file]) => {
            let db = open_read_only(file)?;
            db.check().map_err(|e| failed(file, e))?;
            write_line(&[b"ok"])
        }
        (
            "put" | "get" | "delete" | "firstkey" | "nextkey" | "count" | "list" | "dump" | "stats"
            | "check" | "reorganise" | "reorganize",
            _,
        ) => Err(Failure::Usage(format!(
            "wrong number of arguments to {command}"
        ))),
        _ => Err(Failure::Usage(format!("unknown command: {command}"))),
    }
}

fn create(args: &[&str]) -> Result<(), Failure> {
    let mut options = CreateOptions::default();
    let mut file = None;
    let mut args = args.iter();
    while let Some(&arg) = args.next() {
        match arg {
            "--page-size" => options.page_size = number(arg, args.next())?,
            "--max-depth" => options.max_depth = number(arg, args.next())?,
            _ if arg.starts_with("--") => {
                return Err(Failure::Usage(format!("unknown option: {arg}")));
            }
            _ if file.is_none() => file = Some(arg),
            _ => return Err(Failure::Usage("create takes one FILE".into())),
        }
    }
    let file = file.ok_or_else(|| Failure::Usage("create needs a FILE".into()))?;
    Database::create(file, options).map_err(|e| failed(file, e))?;
    Ok(())
}

fn number<T: std::str::FromStr>(flag: &str, value: Option<&&str>) -> Result<T, Failure> {
    value
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| Failure::Usage(format!("{flag} needs a number")))
}

fn open(file: &str) -> Result<Database, Failure> {
    Database::open(file).map_err(|e| failed(file, e))
}

fn open_read_only(file: &str) -> Result<Database, Failure> {
    Database::open_read_only(file).map_err(|e| failed(file, e))
}

fn failed(file: &str, e: rdbm::Error) -> Failure {
    Failure::Error(format!("{file}: {e}"))
}

fn write_line(parts: &[&[u8]]) -> Result<(), Failure> {
    write_to(&mut io::stdout().lock(), parts)
}

fn write_to(out: &mut impl Write, parts: &[&[u8]]) -> Result<(), Failure> {
    parts
        .iter()
        .try_for_each(|p| out.write_all(p))
        .and_then(|()| out.write_all(b"\n"))
        .map_err(|e| match e.kind() {
            io::ErrorKind::BrokenPipe => Failure::BrokenPipe,
            _ => Failure::Error(format!("write error: {e}")),
        })
}
