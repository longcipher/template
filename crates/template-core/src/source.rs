//! Template source fetching implementations.

use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

use crate::error::TemplateError;

/// Fetch and validate a local template source path
///
/// # Arguments
/// * `path` - The path to the local template directory
///
/// # Returns
/// * `Ok(PathBuf)` - The canonicalized absolute path to the template
/// * `Err(TemplateError)` - If the path doesn't exist or isn't a directory
pub fn fetch_local(path: &Path) -> Result<PathBuf, TemplateError> {
    // Check if path exists
    if !path.exists() {
        return Err(TemplateError::LocalPathNotFound(path.to_path_buf()));
    }

    // Canonicalize to absolute path (resolves symlinks)
    let canonical =
        path.canonicalize().map_err(|_| TemplateError::LocalPathNotFound(path.to_path_buf()))?;

    // Verify it's a directory
    if !canonical.is_dir() {
        return Err(TemplateError::LocalPathNotFound(path.to_path_buf()));
    }

    Ok(canonical)
}

/// Fetch a GitHub repository by cloning to a temporary directory
///
/// # Arguments
/// * `owner` - Repository owner
/// * `repo` - Repository name
/// * `revision` - Optional branch or tag to checkout
/// * `subdir` - Optional subdirectory within the repo
/// * `use_ssh` - Whether to use SSH instead of HTTPS
///
/// # Returns
/// * `Ok((TempDir, PathBuf))` - The temp directory (for RAII cleanup) and path to template root
/// * `Err(TemplateError)` - If clone fails or subdir doesn't exist
pub fn fetch_github(
    owner: &str,
    repo: &str,
    revision: Option<&str>,
    subdir: Option<&Path>,
    use_ssh: bool,
) -> Result<(TempDir, PathBuf), TemplateError> {
    // 1. Create TempDir
    let temp_dir = TempDir::new()?;

    // 2. Build clone URL
    let clone_url = if use_ssh {
        format!("git@github.com:{}/{}.git", owner, repo)
    } else {
        format!("https://github.com/{}/{}.git", owner, repo)
    };

    // 3. Build and execute git clone command
    let mut cmd = Command::new("git");
    cmd.arg("clone").arg("--depth").arg("1");

    if let Some(rev) = revision {
        cmd.arg("--branch").arg(rev);
    }

    cmd.arg(&clone_url).arg(temp_dir.path());

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(TemplateError::GitCloneFailed {
            message: format!("Failed to clone {}/{}", owner, repo),
            stderr,
        });
    }

    // 4. Determine source root (with optional subdir)
    let source_root = match subdir {
        Some(sub) => {
            let path = temp_dir.path().join(sub);
            if !path.exists() || !path.is_dir() {
                return Err(TemplateError::SubdirNotFound(sub.to_path_buf()));
            }
            path
        }
        None => temp_dir.path().to_path_buf(),
    };

    Ok((temp_dir, source_root))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn test_fetch_local_existing_dir() {
        let temp = TempDir::new().unwrap();
        let result = fetch_local(temp.path());
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    fn test_fetch_local_nonexistent_path() {
        let result = fetch_local(Path::new("/nonexistent/path/12345"));
        assert!(matches!(result, Err(TemplateError::LocalPathNotFound(_))));
    }

    #[test]
    fn test_fetch_local_file_not_dir() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("file.txt");
        fs::write(&file_path, "content").unwrap();
        let result = fetch_local(&file_path);
        assert!(matches!(result, Err(TemplateError::LocalPathNotFound(_))));
    }

    #[test]
    fn test_fetch_local_relative_path() {
        // Use current directory which always exists
        let result = fetch_local(Path::new("."));
        assert!(result.is_ok());
        assert!(result.unwrap().is_absolute());
    }

    #[test]
    #[ignore] // Requires network access
    fn test_fetch_github_public_repo() {
        // Clone a small known public repo
        let result = fetch_github("octocat", "Hello-World", None, None, false);
        assert!(result.is_ok());
        let (temp_dir, source_root) = result.unwrap();
        assert!(source_root.exists());
        assert!(source_root.join("README").exists());
        drop(temp_dir); // Explicit cleanup
    }

    #[test]
    fn test_fetch_github_nonexistent_repo() {
        let result =
            fetch_github("nonexistent-user-12345", "nonexistent-repo-67890", None, None, false);
        assert!(matches!(result, Err(TemplateError::GitCloneFailed { .. })));
    }

    #[test]
    fn test_clone_url_construction() {
        // Test that URLs are constructed correctly (via a helper or inline)
        let https_url = format!("https://github.com/{}/{}.git", "owner", "repo");
        assert_eq!(https_url, "https://github.com/owner/repo.git");

        let ssh_url = format!("git@github.com:{}/{}.git", "owner", "repo");
        assert_eq!(ssh_url, "git@github.com:owner/repo.git");
    }
}
