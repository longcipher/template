//! # Template Core
//!
//! A library for scaffolding projects from GitHub repositories or local directories.
//!
//! ## Features
//!
//! - Parse template source specifications (GitHub repos, local paths)
//! - Clone GitHub repositories (public and private via system git)
//! - Apply templates with skip-existing file logic
//!
//! ## Example
//!
//! ```no_run
//! use template_core::{parse_source, SourceKind, fetch_github, fetch_local, apply_template};
//! use std::path::Path;
//!
//! // Parse a GitHub source
//! let source = parse_source("owner/repo#main").unwrap();
//!
//! // Fetch and apply based on source kind
//! match source.kind {
//!     SourceKind::GitHub { owner, repo, subdir, revision } => {
//!         let (_temp, src_path) = fetch_github(
//!             &owner,
//!             &repo,
//!             revision.as_deref(),
//!             subdir.as_deref(),
//!             false,
//!         ).unwrap();
//!         // GitHub clones don't need gitignore filtering (already clean)
//!         apply_template(&src_path, Path::new("."), false, |event| {
//!             println!("{:?}", event);
//!         }).unwrap();
//!     }
//!     SourceKind::Local { path } => {
//!         let src_path = fetch_local(&path).unwrap();
//!         // Local templates respect .gitignore
//!         apply_template(&src_path, Path::new("."), true, |event| {
//!             println!("{:?}", event);
//!         }).unwrap();
//!     }
//! }
//! ```

pub mod apply;
pub mod error;
pub mod parser;
pub mod source;

// Re-export main types for convenience
pub use apply::{ApplyEvent, ApplyResult, apply_template};
pub use error::TemplateError;
pub use parser::{SourceKind, TemplateSource, parse_source};
pub use source::{fetch_github, fetch_local};

/// Prelude module for convenient imports
///
/// ```
/// use template_core::prelude::*;
/// ```
pub mod prelude {
    pub use crate::apply::{ApplyEvent, ApplyResult, apply_template};
    pub use crate::error::TemplateError;
    pub use crate::parser::{SourceKind, TemplateSource, parse_source};
    pub use crate::source::{fetch_github, fetch_local};
}
