use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during template operations
#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Invalid source format: {0}")]
    InvalidSource(String),

    #[error("Local path does not exist: {0}")]
    LocalPathNotFound(PathBuf),

    #[error("Subdirectory not found in repository: {0}")]
    SubdirNotFound(PathBuf),

    #[error("Git clone failed: {message}")]
    GitCloneFailed { message: String, stderr: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path processing error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_source_display() {
        let err = TemplateError::InvalidSource("bad input".to_string());
        assert_eq!(err.to_string(), "Invalid source format: bad input");
    }

    #[test]
    fn test_local_path_not_found_display() {
        let err = TemplateError::LocalPathNotFound(PathBuf::from("/nonexistent"));
        assert_eq!(err.to_string(), "Local path does not exist: /nonexistent");
    }

    #[test]
    fn test_git_clone_failed_display() {
        let err = TemplateError::GitCloneFailed {
            message: "auth failed".to_string(),
            stderr: "Permission denied".to_string(),
        };
        assert_eq!(err.to_string(), "Git clone failed: auth failed");
    }
}
