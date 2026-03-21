//! CLI integration tests for tpl

use std::{
    fs::{self, File},
    io::Write,
};

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use predicates::prelude::*;
use tempfile::TempDir;

/// Get the tpl command
fn tpl() -> Command {
    cargo_bin_cmd!("tpl")
}

#[test]
fn test_help_output() {
    tpl()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scaffold projects from GitHub"))
        .stdout(predicate::str::contains("SOURCE"))
        .stdout(predicate::str::contains("--ssh"))
        .stdout(predicate::str::contains("--verbose"));
}

#[test]
fn test_version_output() {
    tpl().arg("--version").assert().success().stdout(predicate::str::contains("tpl"));
}

#[test]
fn test_missing_source_arg() {
    tpl().assert().failure().stderr(predicate::str::contains("SOURCE"));
}

#[test]
fn test_invalid_source_format() {
    tpl()
        .arg("invalid-no-slash")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid source format"));
}

#[test]
fn test_local_template_success() {
    // Create a source template
    let src = TempDir::new().unwrap();
    fs::create_dir_all(src.path().join("src")).unwrap();
    File::create(src.path().join("README.md")).unwrap().write_all(b"# Test").unwrap();
    File::create(src.path().join("src/main.rs")).unwrap().write_all(b"fn main() {}").unwrap();

    // Create destination
    let dest = TempDir::new().unwrap();

    let source_arg = format!("file://{}", src.path().display());

    tpl()
        .arg(&source_arg)
        .arg(dest.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Done!"))
        .stdout(predicate::str::contains("Applied:"));

    // Verify files were created
    assert!(dest.path().join("README.md").exists());
    assert!(dest.path().join("src/main.rs").exists());
}

#[test]
fn test_local_template_with_existing_files() {
    let src = TempDir::new().unwrap();
    File::create(src.path().join("README.md")).unwrap().write_all(b"# New").unwrap();
    File::create(src.path().join("other.txt")).unwrap().write_all(b"other").unwrap();

    let dest = TempDir::new().unwrap();
    // Pre-create README
    File::create(dest.path().join("README.md")).unwrap().write_all(b"# Existing").unwrap();

    let source_arg = format!("file://{}", src.path().display());

    tpl()
        .arg(&source_arg)
        .arg(dest.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Skipped:"));

    // Verify existing file was not overwritten
    let content = fs::read_to_string(dest.path().join("README.md")).unwrap();
    assert_eq!(content, "# Existing");
}

#[test]
fn test_nonexistent_local_path() {
    tpl()
        .arg("file:///nonexistent/path/12345")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Local path does not exist"));
}

#[test]
fn test_verbose_flag() {
    let src = TempDir::new().unwrap();
    fs::create_dir_all(src.path().join("a/b")).unwrap();
    File::create(src.path().join("a/b/deep.txt")).unwrap().write_all(b"deep").unwrap();

    let dest = TempDir::new().unwrap();
    let source_arg = format!("file://{}", src.path().display());

    tpl()
        .arg("--verbose")
        .arg(&source_arg)
        .arg(dest.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Created:"));
}
