// SPDX-License-Identifier: MPL-2.0

use std::{
    fs::remove_dir_all,
    path::{Path, PathBuf},
};

use crate::util::*;

const KERNEL_NAME: &str = "myos";
const LIB_NAME: &str = "my_module";

#[test]
fn create_kernel_in_workspace() {
    const WORKSPACE_NAME: &str = "/tmp/kernel_workspace";
    if Path::new(WORKSPACE_NAME).exists() {
        remove_dir_all(WORKSPACE_NAME).unwrap();
    }
    create_workspace(WORKSPACE_NAME, &[KERNEL_NAME]);
    let kernel_path = PathBuf::from(WORKSPACE_NAME).join(KERNEL_NAME);
    let manifest_path = kernel_path.join("Cargo.toml");

    let mut cmd = cargo_ksdk(["new", "--kernel", KERNEL_NAME]);
    cmd.current_dir(WORKSPACE_NAME);
    let output = cmd.output().unwrap();
    depends_on_local_kstd(&manifest_path);
    assert_success(&output);
    remove_dir_all(&kernel_path).unwrap();

    let mut cmd = cargo_ksdk(["new", "-t", "kernel", KERNEL_NAME]);
    cmd.current_dir(WORKSPACE_NAME);
    let output = cmd.output().unwrap();
    depends_on_local_kstd(&manifest_path);
    assert_success(&output);
    remove_dir_all(&kernel_path).unwrap();

    remove_dir_all(WORKSPACE_NAME).unwrap();
}

#[test]
fn create_lib_in_workspace() {
    const WORKSPACE_NAME: &str = "/tmp/lib_workspace";
    if Path::new(WORKSPACE_NAME).exists() {
        remove_dir_all(WORKSPACE_NAME).unwrap();
    }
    create_workspace(WORKSPACE_NAME, &[LIB_NAME]);
    let mut cargo_ksdk = cargo_ksdk(["new", LIB_NAME]);
    cargo_ksdk.current_dir(WORKSPACE_NAME);
    let output = cargo_ksdk.output().unwrap();
    assert_success(&output);
    remove_dir_all(WORKSPACE_NAME).unwrap();
}

#[test]
fn create_two_crates_in_workspace() {
    const WORKSPACE_NAME: &str = "/tmp/my_workspace";
    if Path::new(WORKSPACE_NAME).exists() {
        remove_dir_all(WORKSPACE_NAME).unwrap();
    }

    create_workspace(WORKSPACE_NAME, &[LIB_NAME]);
    // Create lib crate
    let mut command = cargo_ksdk(["new", LIB_NAME]);
    command.current_dir(WORKSPACE_NAME);
    let output = command.output().unwrap();
    assert_success(&output);

    add_member_to_workspace(WORKSPACE_NAME, KERNEL_NAME);
    // Create kernel crate
    let mut command = cargo_ksdk(["new", "--kernel", KERNEL_NAME]);
    command.current_dir(WORKSPACE_NAME);
    let output = command.output().unwrap();
    assert_success(&output);

    remove_dir_all(WORKSPACE_NAME).unwrap();
}
