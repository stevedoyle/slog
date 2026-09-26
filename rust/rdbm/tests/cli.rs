//! Runs the `rdbm` binary the way a user would from a shell.

mod support;

use std::collections::BTreeMap;
use std::process::{Command, Output};

use rdbm::{CreateOptions, Database};
use support::TempPath;

fn rdbm(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rdbm"))
        .args(args)
        .output()
        .expect("failed to run rdbm")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn create_put_then_get_in_separate_runs() {
    let tmp = TempPath::new("cli.db");
    let db = tmp.path().to_str().unwrap();

    assert!(rdbm(&["create", db]).status.success());
    assert!(rdbm(&["put", db, "key1", "value one"]).status.success());
    assert!(rdbm(&["put", db, "key2", "value two"]).status.success());

    let out = rdbm(&["get", db, "key1"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "value one\n");
    assert_eq!(stdout(&rdbm(&["get", db, "key2"])), "value two\n");
    assert_eq!(stdout(&rdbm(&["count", db])), "2\n");
    assert_eq!(stdout(&rdbm(&["check", db])), "ok\n");
}

#[test]
fn delete_then_get_reports_not_found() {
    let tmp = TempPath::new("cli-delete.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    rdbm(&["put", db, "key1", "value one"]);

    assert!(rdbm(&["delete", db, "key1"]).status.success());
    let out = rdbm(&["get", db, "key1"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
    assert!(stderr(&out).contains("key1: not found"));

    let out = rdbm(&["delete", db, "key1"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn list_prints_keys_and_dump_prints_pairs() {
    let tmp = TempPath::new("cli-list.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    rdbm(&["put", db, "b", "2"]);
    rdbm(&["put", db, "a", "1"]);

    let sorted_lines = |args: &[&str]| {
        let mut lines: Vec<String> = stdout(&rdbm(args)).lines().map(str::to_owned).collect();
        lines.sort();
        lines
    };
    assert_eq!(sorted_lines(&["list", db]), ["a", "b"]);
    assert_eq!(sorted_lines(&["dump", db]), ["a\t1", "b\t2"]);
}

#[test]
fn create_options_are_recorded() {
    let tmp = TempPath::new("cli-options.db");
    let db = tmp.path().to_str().unwrap();
    let out = rdbm(&["create", "--page-size", "512", "--max-depth", "3", db]);
    assert!(out.status.success(), "{}", stderr(&out));

    let stats = stdout(&rdbm(&["stats", db]));
    assert!(stats.contains("page_size=512\n"), "{stats}");
    assert!(stats.contains("max_depth=3\n"), "{stats}");
    assert_eq!(std::fs::metadata(db).unwrap().len(), 3 * 512);
}

#[test]
fn usage_errors_exit_2() {
    for args in [
        &[][..],
        &["frobnicate", "x.db"][..],
        &["get", "x.db"][..],
        &["create"][..],
        &["create", "--page-size", "big", "x.db"][..],
    ] {
        let out = rdbm(args);
        assert_eq!(out.status.code(), Some(2), "{args:?}");
        assert!(stderr(&out).contains("usage:"), "{args:?}");
    }
}

#[test]
fn errors_name_the_file_and_exit_1() {
    let tmp = TempPath::new("cli-missing.db");
    let db = tmp.path().to_str().unwrap();
    let out = rdbm(&["get", db, "k"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains(db), "{}", stderr(&out));

    rdbm(&["create", db]);
    let out = rdbm(&["create", db]);
    assert_eq!(out.status.code(), Some(1), "create over an existing file");
}

#[test]
fn firstkey_nextkey_walk_every_key_once() {
    let tmp = TempPath::new("cli-iter.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    for (k, v) in [("apple", "red"), ("banana", "yellow"), ("cherry", "red")] {
        rdbm(&["put", db, k, v]);
    }

    let mut seen = Vec::new();
    let mut out = rdbm(&["firstkey", db]);
    while out.status.success() {
        let key = stdout(&out).trim_end_matches('\n').to_owned();
        out = rdbm(&["nextkey", db, &key]);
        seen.push(key);
    }
    // The end of iteration is exit 1 with no output and no message.
    assert_eq!(out.status.code(), Some(1));
    assert_eq!((stdout(&out), stderr(&out)), (String::new(), String::new()));

    let listed: Vec<String> = stdout(&rdbm(&["list", db]))
        .lines()
        .map(|l| l.split('\t').next().unwrap().to_owned())
        .collect();
    assert_eq!(listed, seen, "list uses the same order");
    seen.sort();
    assert_eq!(seen, ["apple", "banana", "cherry"]);

    rdbm(&["delete", db, "banana"]);
    let out = rdbm(&["nextkey", db, "banana"]);
    assert!(out.status.success(), "nextkey resumes from a deleted key");
}

#[test]
fn firstkey_on_empty_database_exits_1() {
    let tmp = TempPath::new("cli-iter-empty.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    let out = rdbm(&["firstkey", db]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stdout(&out), "");
}

#[test]
fn reorganise_keeps_remaining_keys() {
    let tmp = TempPath::new("cli-reorg.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    for (k, v) in [("a", "1"), ("b", "2"), ("c", "3")] {
        rdbm(&["put", db, k, v]);
    }
    rdbm(&["delete", db, "a"]);
    rdbm(&["delete", db, "c"]);

    let out = rdbm(&["reorganise", db]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(stdout(&out), "", "silent on success");
    assert_eq!(stdout(&rdbm(&["get", db, "b"])), "2\n");
    assert_eq!(rdbm(&["get", db, "a"]).status.code(), Some(1));
    assert_eq!(stdout(&rdbm(&["check", db])), "ok\n");

    // gdbmtool's spelling works too.
    assert!(rdbm(&["reorganize", db]).status.success());
}

/// Parses `name=value` lines from `rdbm stats`.
fn stats(db: &str) -> BTreeMap<String, u64> {
    stdout(&rdbm(&["stats", db]))
        .lines()
        .map(|l| {
            let (name, value) = l.split_once('=').expect("name=value");
            (name.to_owned(), value.parse().expect("number"))
        })
        .collect()
}

#[test]
fn complete_workflow() {
    let tmp = TempPath::new("cli-demo.db");
    let db = tmp.path().to_str().unwrap();

    assert!(rdbm(&["create", db]).status.success());
    rdbm(&["put", db, "title", "The Lord of the Rings"]);
    rdbm(&["put", db, "author", "J.R.R. Tolkien"]);
    rdbm(&["put", db, "year", "1954"]);

    assert_eq!(
        stdout(&rdbm(&["get", db, "title"])),
        "The Lord of the Rings\n"
    );
    assert_eq!(stdout(&rdbm(&["get", db, "author"])), "J.R.R. Tolkien\n");

    let mut keys: Vec<String> = stdout(&rdbm(&["list", db]))
        .lines()
        .map(str::to_owned)
        .collect();
    keys.sort();
    assert_eq!(keys, ["author", "title", "year"]);

    let s = stats(db);
    assert_eq!(s["entries"], 3);
    assert_eq!(s["file_size"], std::fs::metadata(db).unwrap().len());
    assert_eq!(
        (s["buckets"], s["overflow_pages"], s["global_depth"]),
        (1, 0, 0)
    );

    assert!(rdbm(&["delete", db, "year"]).status.success());
    let out = rdbm(&["get", db, "year"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stderr(&out), "rdbm: year: not found\n");

    assert!(rdbm(&["reorganise", db]).status.success());
    assert_eq!(
        stdout(&rdbm(&["get", db, "title"])),
        "The Lord of the Rings\n"
    );
    assert_eq!(stats(db)["entries"], 2);
}

#[test]
fn stats_names_every_field() {
    let tmp = TempPath::new("cli-stats.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    let names: Vec<String> = stats(db).into_keys().collect();
    let mut expected = [
        "entries",
        "file_size",
        "buckets",
        "overflow_pages",
        "global_depth",
        "fill_percent",
        "free_pages",
        "page_size",
        "pages",
        "max_depth",
        "directory_size",
        "directory_pages",
    ];
    expected.sort();
    assert_eq!(names, expected);
}

#[test]
fn thousands_of_keys_via_the_cli() {
    let tmp = TempPath::new("cli-large.db");
    let db = tmp.path().to_str().unwrap();
    let n = 3000;
    {
        // Load through the library; thousands of synced CLI puts would take
        // tens of seconds.
        let mut d = Database::create(db, CreateOptions::default()).unwrap();
        d.set_sync(false);
        for i in 0..n {
            d.put(
                format!("key{i}").as_bytes(),
                format!("value {i}").as_bytes(),
            )
            .unwrap();
        }
        d.sync().unwrap();
    }

    let mut listed: Vec<String> = stdout(&rdbm(&["list", db]))
        .lines()
        .map(str::to_owned)
        .collect();
    listed.sort();
    let mut expected: Vec<String> = (0..n).map(|i| format!("key{i}")).collect();
    expected.sort();
    assert_eq!(listed, expected, "every key listed exactly once");

    for i in (0..n).step_by(97) {
        assert_eq!(
            stdout(&rdbm(&["get", db, &format!("key{i}")])),
            format!("value {i}\n")
        );
    }
    assert_eq!(stdout(&rdbm(&["count", db])), format!("{n}\n"));
    assert_eq!(stdout(&rdbm(&["check", db])), "ok\n");
    let s = stats(db);
    assert!(s["buckets"] > 1 && s["global_depth"] > 0, "{s:?}");
}

#[test]
fn help_and_version() {
    for flag in ["--help", "-h", "help"] {
        let out = rdbm(&[flag]);
        assert!(out.status.success(), "{flag}");
        for command in [
            "create",
            "put",
            "get",
            "delete",
            "list",
            "stats",
            "reorganise",
        ] {
            assert!(stdout(&out).contains(command), "{flag} mentions {command}");
        }
    }
    for flag in ["--version", "-V"] {
        let out = rdbm(&[flag]);
        assert!(out.status.success());
        assert_eq!(
            stdout(&out),
            format!("rdbm {}\n", env!("CARGO_PKG_VERSION"))
        );
    }
}
