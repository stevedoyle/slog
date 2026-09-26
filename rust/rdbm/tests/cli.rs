//! Runs the `rdbm` binary the way a user would from a shell.

mod support;

use std::process::{Command, Output};

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
fn list_prints_tab_separated_pairs() {
    let tmp = TempPath::new("cli-list.db");
    let db = tmp.path().to_str().unwrap();
    rdbm(&["create", db]);
    rdbm(&["put", db, "b", "2"]);
    rdbm(&["put", db, "a", "1"]);

    let mut lines: Vec<String> = stdout(&rdbm(&["list", db]))
        .lines()
        .map(str::to_owned)
        .collect();
    lines.sort();
    assert_eq!(lines, ["a\t1", "b\t2"]);
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
