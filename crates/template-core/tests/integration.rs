//! Integration tests for template-core
//!
//! Note: Some tests require network access and are marked with #[ignore].
//! Run them with: cargo test -- --ignored

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use tempfile::TempDir;
use template_core::{
    ApplyEvent, SourceKind, apply_template, fetch_github, fetch_local, parse_source,
};

/// Helper to create a test template directory
fn create_test_template(dir: &Path) {
    fs::create_dir_all(dir.join("src")).unwrap();
    File::create(dir.join("README.md")).unwrap().write_all(b"# Test Template\n").unwrap();
    File::create(dir.join("src/main.rs"))
        .unwrap()
        .write_all(b"fn main() { println!(\"Hello\"); }\n")
        .unwrap();
    File::create(dir.join(".gitignore")).unwrap().write_all(b"/target\n").unwrap();
}

#[test]
fn test_full_local_workflow() {
    // Create source template
    let src = TempDir::new().unwrap();
    create_test_template(src.path());

    // Create destination
    let dest = TempDir::new().unwrap();

    // Parse source
    let source_str = format!("file://{}", src.path().display());
    let source = parse_source(&source_str).unwrap();

    // Fetch
    let source_path = match source.kind {
        SourceKind::Local { path } => fetch_local(&path).unwrap(),
        _ => panic!("Expected local source"),
    };

    // Apply
    let mut events = Vec::new();
    let result = apply_template(&source_path, dest.path(), true, |e| events.push(e)).unwrap();

    // Verify (with gitignore, .gitignore itself is copied but ignored patterns are not)
    assert_eq!(result.applied.len(), 3); // README.md, src/main.rs, .gitignore
    assert!(dest.path().join("README.md").exists());
    assert!(dest.path().join("src/main.rs").exists());
    assert!(dest.path().join(".gitignore").exists());
}

#[test]
fn test_local_with_existing_files() {
    let src = TempDir::new().unwrap();
    create_test_template(src.path());

    let dest = TempDir::new().unwrap();
    // Pre-create a file that should be skipped
    File::create(dest.path().join("README.md")).unwrap().write_all(b"Existing content").unwrap();

    let source_str = format!("file://{}", src.path().display());
    let source = parse_source(&source_str).unwrap();
    let source_path = match source.kind {
        SourceKind::Local { path } => fetch_local(&path).unwrap(),
        _ => panic!("Expected local source"),
    };

    let result = apply_template(&source_path, dest.path(), true, |_| {}).unwrap();

    assert_eq!(result.applied.len(), 2); // src/main.rs and .gitignore
    assert_eq!(result.skipped.len(), 1); // README.md

    // Verify existing file was NOT overwritten
    let content = fs::read_to_string(dest.path().join("README.md")).unwrap();
    assert_eq!(content, "Existing content");
}

#[test]
fn test_parse_and_match_github_source() {
    let source = parse_source("owner/repo#v1.0.0").unwrap();
    match source.kind {
        SourceKind::GitHub { owner, repo, revision, subdir } => {
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(revision, Some("v1.0.0".to_string()));
            assert!(subdir.is_none());
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[test]
fn test_parse_github_with_subdir() {
    let source = parse_source("owner/repo/templates/rust#main").unwrap();
    match source.kind {
        SourceKind::GitHub { owner, repo, revision, subdir } => {
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(revision, Some("main".to_string()));
            assert_eq!(subdir, Some("templates/rust".into()));
        }
        _ => panic!("Expected GitHub source"),
    }
}

#[test]
#[ignore] // Requires network access
fn test_github_clone_public_repo() {
    // Clone a small, stable public repo
    let result = fetch_github("octocat", "Hello-World", None, None, false);
    assert!(result.is_ok());

    let (temp_dir, source_path) = result.unwrap();
    assert!(source_path.join("README").exists());
    drop(temp_dir); // Cleanup
}

#[test]
#[ignore] // Requires network access
fn test_github_clone_with_branch() {
    let result = fetch_github("octocat", "Hello-World", Some("master"), None, false);
    assert!(result.is_ok());
}

#[test]
fn test_github_clone_nonexistent_fails() {
    let result =
        fetch_github("this-user-does-not-exist-12345", "nonexistent-repo-67890", None, None, false);
    assert!(result.is_err());
}

#[test]
fn test_apply_events_are_emitted() {
    let src = TempDir::new().unwrap();
    create_test_template(src.path());
    let dest = TempDir::new().unwrap();

    let source_path = fetch_local(src.path()).unwrap();

    let mut events = Vec::new();
    apply_template(&source_path, dest.path(), false, |e| events.push(e)).unwrap();

    // Should have events for directories and files
    let applied_count = events.iter().filter(|e| matches!(e, ApplyEvent::FileApplied(_))).count();
    assert_eq!(applied_count, 3);
}

#[test]
fn test_nested_directory_creation() {
    let src = TempDir::new().unwrap();

    // Create deeply nested structure
    let deep_dir = src.path().join("a/b/c/d");
    fs::create_dir_all(&deep_dir).unwrap();
    File::create(deep_dir.join("deep.txt")).unwrap().write_all(b"deep content").unwrap();

    let dest = TempDir::new().unwrap();
    let source_path = fetch_local(src.path()).unwrap();

    let result = apply_template(&source_path, dest.path(), false, |_| {}).unwrap();

    assert!(result.applied.iter().any(|p| p.ends_with("deep.txt")));
    assert!(dest.path().join("a/b/c/d/deep.txt").exists());
}

#[test]
fn test_empty_template_produces_no_files() {
    let src = TempDir::new().unwrap();
    let dest = TempDir::new().unwrap();

    let source_path = fetch_local(src.path()).unwrap();
    let result = apply_template(&source_path, dest.path(), false, |_| {}).unwrap();

    assert!(result.applied.is_empty());
    assert!(result.skipped.is_empty());
}
