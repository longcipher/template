# Template CLI - Design Document

## 1. Overview

**Template** is a lightweight CLI tool for scaffolding projects from GitHub repositories or local directories. Unlike degit/tiged, it focuses on simplicity and transparency:

- **No tarball optimization** - Uses `git clone` for full compatibility with private repos
- **No template engine** - Pure file copying, preserves binary files
- **Skip-existing strategy** - Never overwrites user files
- **Transparent output** - Clearly shows applied and skipped files

## 2. Project Metadata

| Item | Value |
|------|-------|
| **Project Name** | template |
| **Binary Name** | `tpl` (primary), `template` (alias) |
| **Min Rust Version** | 1.85+ (Edition 2024) |
| **License** | MIT / Apache-2.0 |

## 3. Core Dependencies

Following the project's dependency guidelines:

| Crate | Purpose | Notes |
|-------|---------|-------|
| `clap` | CLI argument parsing | With `derive` feature |
| `eyre` | Error handling | App-level errors (NOT anyhow) |
| `tracing` | Structured logging | NOT log crate |
| `tracing-subscriber` | Log output formatting | Console output |
| `walkdir` | Directory traversal | Recursive file iteration |
| `tempfile` | Temporary directories | For git clone target |
| `console` | Terminal styling | Colors, spinners |
| `indicatif` | Progress reporting | Progress bars |

## 4. Input Syntax Specification

### 4.1 Syntax Grammar

```text
INPUT        := LOCAL_SOURCE | REMOTE_SOURCE
LOCAL_SOURCE := "file://" PATH
REMOTE_SOURCE:= REPO_SPEC ["#" REVISION]
REPO_SPEC    := OWNER "/" REPO ["/" SUBDIR_PATH]
OWNER        := [a-zA-Z0-9_-]+
REPO         := [a-zA-Z0-9_.-]+
SUBDIR_PATH  := PATH_SEGMENT ("/" PATH_SEGMENT)*
REVISION     := BRANCH | TAG
```

### 4.2 Examples

| Input | Owner | Repo | Subdir | Revision |
|-------|-------|------|--------|----------|
| `user/repo` | user | repo | None | None (default branch) |
| `user/repo#dev` | user | repo | None | dev |
| `user/repo#v1.0.0` | user | repo | None | v1.0.0 |
| `user/repo/templates/rust` | user | repo | templates/rust | None |
| `user/repo/src/starter#main` | user | repo | src/starter | main |
| `file:///path/to/template` | N/A | N/A | N/A | N/A |
| `file://./relative/path` | N/A | N/A | N/A | N/A |

### 4.3 Parsing Rules

1. **Local Detection**: If input starts with `file://`, treat as local source
2. **Revision Extraction**: Split on `#` - right part is revision (if present)
3. **Repo Parsing**: First two `/`-separated segments are `owner/repo`
4. **Subdir Extraction**: Remaining path segments after `owner/repo` form subdir

## 5. Data Structures

### 5.1 Template Source

```rust
/// Represents a parsed template source location
#[derive(Debug, Clone)]
pub struct TemplateSource {
    /// Source type determines fetching strategy
    pub kind: SourceKind,
}

#[derive(Debug, Clone)]
pub enum SourceKind {
    /// Local filesystem path
    Local {
        path: PathBuf,
    },
    /// GitHub repository
    GitHub {
        /// Full clone URL (https or ssh)
        clone_url: String,
        /// Subdirectory within the repository
        subdir: Option<PathBuf>,
        /// Branch, tag, or commit reference
        revision: Option<String>,
    },
}
```

### 5.2 Application Result

```rust
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
```

### 5.3 CLI Arguments

```rust
#[derive(Parser)]
#[command(name = "tpl", version, about = "Scaffold projects from templates")]
pub struct Cli {
    /// Template source (e.g., user/repo, user/repo#branch, file://path)
    #[arg(value_name = "SOURCE")]
    pub source: String,

    /// Target directory (default: current directory)
    #[arg(value_name = "DEST", default_value = ".")]
    pub destination: PathBuf,

    /// Use SSH for git clone (git@github.com:...)
    #[arg(long, short = 's')]
    pub ssh: bool,

    /// Verbose output
    #[arg(long, short = 'v')]
    pub verbose: bool,
}
```

## 6. Core Logic Flow

### 6.1 High-Level Flow

```text
┌─────────────────┐
│  Parse Input    │
│  (source arg)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Fetch Source   │
│  (clone/locate) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Apply Template │
│  (copy files)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Report Results │
│  (summary)      │
└─────────────────┘
```

### 6.2 Source Fetching

#### Remote (GitHub)

```text
1. Create temporary directory via tempfile::TempDir
2. Construct git clone command:
   - Base: git clone --depth 1
   - If revision: --branch <revision>
   - URL: https://github.com/{owner}/{repo}.git (or SSH variant)
   - Target: temp_dir path
3. Execute via std::process::Command
4. If subdir specified: source_root = temp_dir/subdir
5. Validate source_root exists
```

**Why `git clone` over tarball API:**

- Automatic SSH key and credential helper support
- Works with private repositories without token configuration
- Simpler implementation, fewer edge cases

#### Local (file://)

```text
1. Strip "file://" prefix
2. Resolve path (handle relative paths)
3. Validate path exists and is directory
4. source_root = resolved_path
```

### 6.3 File Application Strategy

```rust
fn apply_template(source: &Path, dest: &Path) -> Result<ApplyResult> {
    let mut result = ApplyResult::default();

    for entry in WalkDir::new(source) {
        let entry = entry?;
        let path = entry.path();

        // Skip directories (we create them on-demand)
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
            // SKIP: File already exists
            result.skipped.push(relative.to_path_buf());
        } else {
            // APPLY: Copy file
            if let Some(parent) = target.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                    result.dirs_created.push(parent.to_path_buf());
                }
            }
            fs::copy(path, &target)?;
            result.applied.push(relative.to_path_buf());
        }
    }

    Ok(result)
}
```

**Key Design Decisions:**

1. **Skip `.git` directory**: Never copy version control metadata
2. **On-demand directory creation**: Only create dirs when copying files
3. **Existence check before copy**: Never overwrite (core requirement)
4. **Preserve file permissions**: `fs::copy` preserves metadata on Unix

## 7. Output Design

### 7.1 Normal Operation

```text
$ tpl user/awesome-template

⠋ Cloning user/awesome-template...
✓ Cloned to temporary directory

Applying template...
  ✓ Applied: src/main.rs
  ✓ Applied: Cargo.toml
  ✓ Applied: README.md
  ⊘ Skipped: .gitignore (exists)
  ⊘ Skipped: .env (exists)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Done!
  Applied: 3 files
  Skipped: 2 files (already exist)
```

### 7.2 Verbose Mode (-v)

Additional output includes:

- Full git clone command
- Source and destination paths for each file
- Directory creation events
- Timing information

### 7.3 Error Cases

```text
$ tpl user/nonexistent-repo

✗ Error: Failed to clone repository
  Command: git clone --depth 1 https://github.com/user/nonexistent-repo.git /tmp/xxx
  Exit code: 128
  Stderr: Repository not found
```

## 8. Error Handling Strategy

### 8.1 Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Invalid source format: {0}")]
    InvalidSource(String),

    #[error("Local path does not exist: {0}")]
    LocalPathNotFound(PathBuf),

    #[error("Subdirectory not found in repository: {0}")]
    SubdirNotFound(PathBuf),

    #[error("Git clone failed: {message}")]
    GitCloneFailed {
        message: String,
        stderr: String,
    },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Path processing error: {0}")]
    StripPrefix(#[from] std::path::StripPrefixError),
}
```

### 8.2 Error Recovery

| Error | Recovery |
|-------|----------|
| Git clone fails | Show full stderr, suggest SSH flag if HTTPS fails |
| Subdir not found | List available top-level directories |
| Permission denied | Suggest running with appropriate permissions |

## 9. Security Considerations

1. **Path Traversal**: Validate that resolved paths don't escape intended directories
2. **Symlink Handling**: Follow symlinks in source, but validate targets
3. **Git Credential Safety**: Use system git, never store credentials
4. **Temp Directory Cleanup**: Use `TempDir` RAII for automatic cleanup

## 10. Testing Strategy

### 10.1 Unit Tests

- Input parsing (various formats)
- Path resolution
- Git URL construction

### 10.2 Integration Tests

- Clone public repository
- Apply to empty directory
- Apply with existing files (skip behavior)
- Local source application

### 10.3 Manual Testing Matrix

| Source Type | Revision | Subdir | Expected |
|-------------|----------|--------|----------|
| Public repo | None | None | Clone default branch |
| Public repo | Branch | None | Clone specific branch |
| Public repo | Tag | Subdir | Clone tag, use subdir |
| Private repo (SSH) | None | None | Use SSH credentials |
| Local path | N/A | N/A | Direct copy |

## 11. Future Considerations (Out of Scope)

These features are explicitly **NOT** in the initial scope:

- Template engine / variable substitution
- Interactive prompts
- Post-install hooks
- Template caching
- Registry / template discovery
- Tarball download optimization

## 12. Project Structure

```text
template/
├── Cargo.toml              # Workspace root
├── Justfile                # Task runner
├── docs/
│   ├── design.md           # This document
│   └── tasks.md            # Implementation tasks
├── bin/
│   └── tpl/
│       ├── Cargo.toml      # Binary crate
│       └── src/
│           └── main.rs     # Entry point
└── crates/
    └── template-core/
        ├── Cargo.toml      # Library crate
        └── src/
            ├── lib.rs
            ├── parser.rs   # Input parsing
            ├── source.rs   # Source fetching
            ├── apply.rs    # File application
            └── error.rs    # Error types
```
