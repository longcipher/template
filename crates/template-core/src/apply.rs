//! Template application - copies files from source to destination.

use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use walkdir::WalkDir;

use crate::error::TemplateError;

/// Events emitted during template application
#[derive(Debug, Clone, PartialEq)]
pub enum ApplyEvent {
    /// A file was successfully copied
    FileApplied(PathBuf),
    /// A file was skipped because it already exists
    FileSkipped(PathBuf),
    /// A directory was created
    DirCreated(PathBuf),
}

/// Result of applying a template
#[derive(Debug, Default)]
pub struct ApplyResult {
    /// Files successfully copied
    pub applied: Vec<PathBuf>,
    /// Files skipped (already exist)
    pub skipped: Vec<PathBuf>,
    /// Directories created
    pub dirs_created: Vec<PathBuf>,
}

/// Apply a template from source to destination
///
/// Copies all files from source to dest, skipping files that already exist.
/// Never overwrites existing files.
///
/// # Arguments
/// * `source` - Source directory containing template files
/// * `dest` - Destination directory
/// * `respect_gitignore` - If true, skip files matching .gitignore patterns
/// * `callback` - Called for each file event (applied/skipped/dir created)
///
/// # Errors
/// Returns `TemplateError` if file operations fail.
pub fn apply_template<F>(
    source: &Path,
    dest: &Path,
    respect_gitignore: bool,
    mut callback: F,
) -> Result<ApplyResult, TemplateError>
where
    F: FnMut(ApplyEvent),
{
    let mut result = ApplyResult::default();

    if respect_gitignore {
        // Use ignore crate's WalkBuilder which respects .gitignore
        let walker = WalkBuilder::new(source)
            .hidden(false) // Don't skip hidden files (except those in .gitignore)
            .git_ignore(true) // Respect .gitignore
            .git_global(false) // Don't use global gitignore
            .git_exclude(false) // Don't require .git/info/exclude
            .require_git(false) // Process .gitignore even without .git directory
            .build();

        for entry in walker.filter_map(|e| e.ok()) {
            let path = entry.path();

            // Skip directories - we create them on demand
            if path.is_dir() {
                continue;
            }

            // Skip .git directory contents
            if path.components().any(|c| c.as_os_str() == ".git") {
                continue;
            }

            // Calculate relative and target paths
            let relative = path.strip_prefix(source)?;
            let target = dest.join(relative);

            if target.exists() {
                // Skip existing files
                result.skipped.push(relative.to_path_buf());
                callback(ApplyEvent::FileSkipped(relative.to_path_buf()));
            } else {
                // Create parent directories if needed
                if let Some(parent) = target.parent()
                    && !parent.exists()
                {
                    fs::create_dir_all(parent)?;
                    result.dirs_created.push(parent.to_path_buf());
                    callback(ApplyEvent::DirCreated(parent.to_path_buf()));
                }
                // Copy the file
                fs::copy(path, &target)?;
                result.applied.push(relative.to_path_buf());
                callback(ApplyEvent::FileApplied(relative.to_path_buf()));
            }
        }
    } else {
        // Original walkdir behavior for non-gitignore sources
        for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Skip directories - we create them on demand
            if path.is_dir() {
                continue;
            }

            // Skip .git directory contents
            if path.components().any(|c| c.as_os_str() == ".git") {
                continue;
            }

            // Calculate relative and target paths
            let relative = path.strip_prefix(source)?;
            let target = dest.join(relative);

            if target.exists() {
                // Skip existing files
                result.skipped.push(relative.to_path_buf());
                callback(ApplyEvent::FileSkipped(relative.to_path_buf()));
            } else {
                // Create parent directories if needed
                if let Some(parent) = target.parent()
                    && !parent.exists()
                {
                    fs::create_dir_all(parent)?;
                    result.dirs_created.push(parent.to_path_buf());
                    callback(ApplyEvent::DirCreated(parent.to_path_buf()));
                }
                // Copy the file
                fs::copy(path, &target)?;
                result.applied.push(relative.to_path_buf());
                callback(ApplyEvent::FileApplied(relative.to_path_buf()));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    fn create_test_template(dir: &Path) {
        fs::create_dir_all(dir.join("src")).unwrap();
        File::create(dir.join("README.md")).unwrap().write_all(b"# Test").unwrap();
        File::create(dir.join("src/main.rs")).unwrap().write_all(b"fn main() {}").unwrap();
    }

    #[test]
    fn test_apply_to_empty_dir() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();
        create_test_template(src.path());

        let result = apply_template(src.path(), dest.path(), false, |_| {}).unwrap();

        assert_eq!(result.applied.len(), 2);
        assert!(result.skipped.is_empty());
        assert!(dest.path().join("README.md").exists());
        assert!(dest.path().join("src/main.rs").exists());
    }

    #[test]
    fn test_skip_existing_files() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();
        create_test_template(src.path());

        // Create existing file in dest
        File::create(dest.path().join("README.md")).unwrap().write_all(b"existing").unwrap();

        let result = apply_template(src.path(), dest.path(), false, |_| {}).unwrap();

        assert_eq!(result.applied.len(), 1); // Only src/main.rs
        assert_eq!(result.skipped.len(), 1); // README.md skipped

        // Verify existing file was NOT overwritten
        let content = fs::read_to_string(dest.path().join("README.md")).unwrap();
        assert_eq!(content, "existing");
    }

    #[test]
    fn test_skip_git_directory() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        // Create .git directory with some files
        fs::create_dir_all(src.path().join(".git/objects")).unwrap();
        File::create(src.path().join(".git/config")).unwrap();
        File::create(src.path().join("README.md")).unwrap();

        let result = apply_template(src.path(), dest.path(), false, |_| {}).unwrap();

        assert_eq!(result.applied.len(), 1); // Only README.md
        assert!(!dest.path().join(".git").exists());
    }

    #[test]
    fn test_callback_events() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();
        create_test_template(src.path());
        File::create(dest.path().join("README.md")).unwrap();

        let mut events = Vec::new();
        apply_template(src.path(), dest.path(), false, |e| events.push(e)).unwrap();

        assert!(events.iter().any(|e| matches!(e, ApplyEvent::FileApplied(_))));
        assert!(events.iter().any(|e| matches!(e, ApplyEvent::FileSkipped(_))));
    }

    #[test]
    fn test_creates_nested_directories() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        fs::create_dir_all(src.path().join("a/b/c")).unwrap();
        File::create(src.path().join("a/b/c/deep.txt")).unwrap();

        let result = apply_template(src.path(), dest.path(), false, |_| {}).unwrap();

        assert_eq!(result.applied.len(), 1);
        assert!(dest.path().join("a/b/c/deep.txt").exists());
    }

    #[test]
    fn test_respects_gitignore() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        // Create template files
        File::create(src.path().join("README.md")).unwrap().write_all(b"# Test").unwrap();
        File::create(src.path().join("keep.txt")).unwrap().write_all(b"keep").unwrap();

        // Create files that should be ignored
        fs::create_dir_all(src.path().join("target")).unwrap();
        File::create(src.path().join("target/debug.bin")).unwrap();
        fs::create_dir_all(src.path().join("node_modules")).unwrap();
        File::create(src.path().join("node_modules/pkg.js")).unwrap();
        File::create(src.path().join(".env")).unwrap();

        // Create .gitignore
        File::create(src.path().join(".gitignore"))
            .unwrap()
            .write_all(b"target/\nnode_modules/\n.env\n")
            .unwrap();

        let result = apply_template(src.path(), dest.path(), true, |_| {}).unwrap();

        // Should only copy README.md, keep.txt, and .gitignore
        assert_eq!(result.applied.len(), 3);
        assert!(dest.path().join("README.md").exists());
        assert!(dest.path().join("keep.txt").exists());
        assert!(dest.path().join(".gitignore").exists());

        // Should NOT copy ignored files
        assert!(!dest.path().join("target").exists());
        assert!(!dest.path().join("node_modules").exists());
        assert!(!dest.path().join(".env").exists());
    }

    #[test]
    fn test_no_gitignore_copies_all() {
        let src = TempDir::new().unwrap();
        let dest = TempDir::new().unwrap();

        // Create template files
        File::create(src.path().join("README.md")).unwrap();
        fs::create_dir_all(src.path().join("target")).unwrap();
        File::create(src.path().join("target/debug.bin")).unwrap();

        // Create .gitignore that would ignore target/
        File::create(src.path().join(".gitignore")).unwrap().write_all(b"target/\n").unwrap();

        // With respect_gitignore = false, should copy everything
        let result = apply_template(src.path(), dest.path(), false, |_| {}).unwrap();

        assert_eq!(result.applied.len(), 3); // README.md, .gitignore, target/debug.bin
        assert!(dest.path().join("target/debug.bin").exists());
    }
}
