// SPDX-License-Identifier: MPL-2.0

use std::fs;

use crate::util::*;

#[test]
fn cli_help_message() {
    let output = cargo_ksdk(&["-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk <COMMAND>");
}

#[test]
fn cli_new_help_message() {
    let output = cargo_ksdk(&["new", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk new [OPTIONS] <name>");
}

#[test]
fn cli_build_help_message() {
    let output = cargo_ksdk(&["build", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk build [OPTIONS]");
}

#[test]
fn cli_run_help_message() {
    let output = cargo_ksdk(&["run", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk run [OPTIONS]");
}

#[test]
fn cli_test_help_message() {
    let output = cargo_ksdk(&["test", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk test [OPTIONS] [TESTNAME]");
}

#[test]
fn cli_check_help_message() {
    let output = cargo_ksdk(&["check", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk check");
}

#[test]
fn cli_clippy_help_message() {
    let output = cargo_ksdk(&["clippy", "-h"]).output().unwrap();
    assert_success(&output);
    assert_stdout_contains_msg(&output, "cargo ksdk clippy");
}

#[test]
fn cli_new_crate_with_hyphen() {
    let output = cargo_ksdk(&["new", "--kernel", "my-first-os"])
        .output()
        .unwrap();
    assert_success(&output);
    assert!(fs::metadata("my-first-os").is_ok());
    let _ = fs::remove_dir_all("my-first-os");
}
