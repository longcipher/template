//! Input parsing for template sources.

use std::path::PathBuf;

use crate::error::TemplateError;

/// Represents a parsed template source location
#[derive(Debug, Clone, PartialEq)]
pub struct TemplateSource {
    pub kind: SourceKind,
}

/// The type of template source
#[derive(Debug, Clone, PartialEq)]
pub enum SourceKind {
    /// Local filesystem path
    Local { path: PathBuf },
    /// GitHub repository
    GitHub {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// Subdirectory within the repository
        subdir: Option<PathBuf>,
        /// Branch, tag, or commit reference
        revision: Option<String>,
    },
}

/// Parse an input string into a TemplateSource
pub fn parse_source(input: &str) -> Result<TemplateSource, TemplateError> {
    let input = input.trim();

    if input.is_empty() {
        return Err(TemplateError::InvalidSource("input cannot be empty".to_string()));
    }

    let kind = if input.starts_with("file://") {
        parse_local_source(input)?
    } else {
        parse_github_source(input)?
    };

    Ok(TemplateSource { kind })
}

/// Parse a local file:// source
fn parse_local_source(input: &str) -> Result<SourceKind, TemplateError> {
    // Strip the "file://" prefix
    let path_str = input
        .strip_prefix("file://")
        .ok_or_else(|| TemplateError::InvalidSource("invalid file:// URL".to_string()))?;

    if path_str.is_empty() {
        return Err(TemplateError::InvalidSource("file:// URL must include a path".to_string()));
    }

    Ok(SourceKind::Local { path: PathBuf::from(path_str) })
}

/// Parse a GitHub source (owner/repo/subdir#revision)
fn parse_github_source(input: &str) -> Result<SourceKind, TemplateError> {
    // Split on '#' to extract revision
    let (path_part, revision) = match input.split_once('#') {
        Some((path, rev)) => {
            let rev = rev.trim();
            if rev.is_empty() { (path, None) } else { (path, Some(rev.to_string())) }
        }
        None => (input, None),
    };

    // Remove trailing slashes for consistent parsing
    let path_part = path_part.trim_end_matches('/');

    // Split path into segments
    let segments: Vec<&str> = path_part.split('/').filter(|s| !s.is_empty()).collect();

    // Need at least owner and repo
    if segments.len() < 2 {
        return Err(TemplateError::InvalidSource(format!(
            "GitHub source requires owner/repo format, got: {}",
            input
        )));
    }

    let owner = segments[0].to_string();
    let repo = segments[1].to_string();

    // Validate owner and repo are not empty
    if owner.is_empty() || repo.is_empty() {
        return Err(TemplateError::InvalidSource("owner and repo cannot be empty".to_string()));
    }

    // Remaining segments form the subdir
    let subdir = if segments.len() > 2 {
        let subdir_path = segments[2..].join("/");
        Some(PathBuf::from(subdir_path))
    } else {
        None
    };

    Ok(SourceKind::GitHub { owner, repo, subdir, revision })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_owner_repo() {
        let result = parse_source("user/repo").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: None,
                revision: None,
            }
        );
    }

    #[test]
    fn test_parse_github_with_revision() {
        let result = parse_source("user/repo#dev").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: None,
                revision: Some("dev".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_github_with_subdir() {
        let result = parse_source("user/repo/templates/rust").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: Some(PathBuf::from("templates/rust")),
                revision: None,
            }
        );
    }

    #[test]
    fn test_parse_github_with_subdir_and_revision() {
        let result = parse_source("user/repo/path#v1.0").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: Some(PathBuf::from("path")),
                revision: Some("v1.0".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_local_absolute_path() {
        let result = parse_source("file:///abs/path").unwrap();
        assert_eq!(result.kind, SourceKind::Local { path: PathBuf::from("/abs/path") });
    }

    #[test]
    fn test_parse_local_relative_path() {
        let result = parse_source("file://./rel/path").unwrap();
        assert_eq!(result.kind, SourceKind::Local { path: PathBuf::from("./rel/path") });
    }

    #[test]
    fn test_parse_invalid_missing_repo() {
        let result = parse_source("invalid");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TemplateError::InvalidSource(_)));
    }

    #[test]
    fn test_parse_invalid_only_user() {
        let result = parse_source("user");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TemplateError::InvalidSource(_)));
    }

    #[test]
    fn test_parse_empty_input() {
        let result = parse_source("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_input() {
        let result = parse_source("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_trailing_slash() {
        let result = parse_source("user/repo/").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: None,
                revision: None,
            }
        );
    }

    #[test]
    fn test_parse_github_empty_revision() {
        let result = parse_source("user/repo#").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: None,
                revision: None,
            }
        );
    }

    #[test]
    fn test_parse_local_empty_path() {
        let result = parse_source("file://");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_github_complex_subdir() {
        let result = parse_source("owner/repo/deep/nested/path#main").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "owner".to_string(),
                repo: "repo".to_string(),
                subdir: Some(PathBuf::from("deep/nested/path")),
                revision: Some("main".to_string()),
            }
        );
    }

    #[test]
    fn test_parse_github_special_chars_in_revision() {
        let result = parse_source("user/repo#feature/branch-name").unwrap();
        assert_eq!(
            result.kind,
            SourceKind::GitHub {
                owner: "user".to_string(),
                repo: "repo".to_string(),
                subdir: None,
                revision: Some("feature/branch-name".to_string()),
            }
        );
    }
}
