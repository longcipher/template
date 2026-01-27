# Template CLI - Implementation Tasks

## Overview

This document breaks down the implementation into concrete, actionable tasks. Tasks are organized by phase and include acceptance criteria.

---

## Phase 1: Project Setup

### Task 1.1: Initialize Cargo Workspace

**Description**: Set up the Cargo workspace with proper structure following project conventions.

**Commands**:

```bash
# Create workspace directories
mkdir -p bin/tpl/src
mkdir -p crates/template-core/src

# Initialize workspace Cargo.toml
```

**Deliverables**:

- [x] Root `Cargo.toml` with workspace configuration
- [x] `bin/tpl/Cargo.toml` for CLI binary
- [x] `crates/template-core/Cargo.toml` for core library

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] `cargo build` succeeds
- [x] Workspace uses Edition 2024
- [x] Dependencies use `workspace = true` in sub-crates

---

### Task 1.2: Add Dependencies

**Description**: Add all required dependencies following the project's `cargo add` workflow.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] All dependencies in root `Cargo.toml` have number versions only
- [x] Sub-crates use `workspace = true` with features where needed
- [x] `cargo check --workspace` succeeds

---

### Task 1.3: Update Justfile

**Description**: Add template-specific development commands to Justfile.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] `just build-tpl` produces binary
- [x] `just run-tpl --help` shows CLI help

---

## Phase 2: Core Library Implementation

### Task 2.1: Implement Error Types

**File**: `crates/template-core/src/error.rs`

**Description**: Define all error types using `thiserror`.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] All variants have meaningful error messages
- [x] Implements `std::error::Error`
- [x] Unit tests for `Display` implementation

---

### Task 2.2: Implement Input Parser

**File**: `crates/template-core/src/parser.rs`

**Description**: Parse input string into `TemplateSource`.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] All test cases pass
- [x] Handles edge cases (trailing slashes, special chars in names)
- [x] Clear error messages for invalid input

---

### Task 2.3: Implement Source Fetching - Local

**File**: `crates/template-core/src/source.rs`

**Description**: Fetch template from local filesystem.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] Handles relative paths (`./templates`)
- [x] Handles absolute paths (`/home/user/templates`)
- [x] Returns error for non-existent paths
- [x] Returns error for files (not directories)

---

### Task 2.4: Implement Source Fetching - GitHub

**File**: `crates/template-core/src/source.rs`

**Description**: Clone GitHub repository to temporary directory.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:

- [x] Clones public repo successfully
- [x] Respects revision parameter
- [x] Handles subdir validation
- [x] Captures and reports stderr on failure
- [x] TempDir is cleaned up after drop

---

### Task 2.5: Implement File Application

**File**: `crates/template-core/src/apply.rs`

**Description**: Copy files from source to destination with skip-existing logic.

**Status**: ✅ COMPLETED
    DirCreated(PathBuf),
}

```text

**Logic**:
1. Walk source directory recursively
2. Skip `.git` directory
3. For each file:
   - Calculate relative path
   - Calculate target path
   - If exists: skip, emit event
   - Else: create parent dirs, copy, emit event
4. Return accumulated result

**Edge Cases**:
- Empty source directory
- Deeply nested directories
- Files with special characters in names
- Symlinks (follow in source)

**Acceptance Criteria**:
- [x] Never overwrites existing files
- [x] Creates necessary parent directories
- [x] Skips `.git` directory contents
- [x] Handles symlinks correctly
- [x] All events emitted in order

---

### Task 2.6: Library Public API

**File**: `crates/template-core/src/lib.rs`

**Description**: Define public API surface.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Clean public API
- [x] Internal details hidden
- [x] Documentation comments on all public items

---

## Phase 3: CLI Implementation

### Task 3.1: Define CLI Structure

**File**: `bin/tpl/src/main.rs`

**Description**: Set up clap CLI structure.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] `--help` shows usage
- [x] `--version` shows version
- [x] Arguments parsed correctly

---

### Task 3.2: Implement Main Flow

**File**: `bin/tpl/src/main.rs`

**Description**: Wire together CLI and core library.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Clean error messages (no panics)
- [x] Progress shown during clone
- [x] Summary printed at end

---

### Task 3.3: Implement Output Formatting

**File**: `bin/tpl/src/output.rs`

**Description**: Pretty console output using `console` and `indicatif`.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Colors work in terminal
- [x] Spinner animates
- [x] No colors when not TTY

---

## Phase 4: Testing

### Task 4.1: Unit Tests for Parser

**File**: `crates/template-core/src/parser.rs`

**Status**: ✅ COMPLETED (implemented in Task 2.2)

**Acceptance Criteria**:
- [x] 100% coverage of parse logic (15+ test cases)
- [x] Edge cases documented

---

### Task 4.2: Unit Tests for Apply Logic

**File**: `crates/template-core/src/apply.rs`

**Status**: ✅ COMPLETED (implemented in Task 2.5)

**Acceptance Criteria**:
- [x] Uses `tempfile` for test directories
- [x] Cleans up after tests
- [x] Tests file content correctness

---

### Task 4.3: Integration Tests

**File**: `crates/template-core/tests/integration.rs`

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Tests can be skipped in offline mode (#[ignore])
- [x] Uses small test repos (octocat/Hello-World)
- [x] 10 integration tests covering full workflow

---

### Task 4.4: CLI Tests

**File**: `bin/tpl/tests/cli.rs`

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Tests CLI binary via `assert_cmd`
- [x] Error messages tested
- [x] 8 CLI tests covering help, version, errors, and template application

---

## Phase 5: Documentation & Polish

### Task 5.1: README.md

**Description**: Comprehensive README with examples.

**Sections**:
- Installation
- Quick Start
- Usage Examples
- How It Works
- Comparison with degit/tiged

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Examples are tested and work
- [x] Clear and concise

---

### Task 5.2: CLI Help Text

**Description**: Ensure `--help` is informative.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Examples in help text
- [x] Clear parameter descriptions
- [x] Common use cases shown

---

### Task 5.3: Binary Alias Setup

**Description**: Support both `tpl` and `template` commands.

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Documented shell alias approach for Bash/Zsh, Fish, PowerShell
- [x] Single binary approach

---

## Phase 6: Release

### Task 6.1: Release Build Optimization

**File**: Root `Cargo.toml`

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Binary size < 5MB (achieved: 736 KB)
- [x] No debug symbols in release (strip = true)
- [x] LTO enabled for optimization

---

### Task 6.2: CI/CD Setup

**File**: `.github/workflows/ci.yml`

**Status**: ✅ COMPLETED

**Acceptance Criteria**:
- [x] Lint job (format + clippy)
- [x] Test job on all platforms (Ubuntu, macOS, Windows)
- [x] Build for 4 targets (Linux x64, macOS x64/arm64, Windows x64)
- [x] Release job on version tags

---

## Task Dependencies

```

```text
Phase 1 (Setup)
  1.1 ─► 1.2 ─► 1.3

Phase 2 (Core Library)
  2.1 ─┬─► 2.2
       ├─► 2.3
       ├─► 2.4
       └─► 2.5
  2.2 ─► 2.6
  2.3 ─► 2.6
  2.4 ─► 2.6
  2.5 ─► 2.6

Phase 3 (CLI)
  2.6 ─► 3.1 ─► 3.2 ─► 3.3

Phase 4 (Testing)
  2.6 ─► 4.1, 4.2, 4.3
  3.3 ─► 4.4

Phase 5 (Docs)
  4.* ─► 5.1, 5.2, 5.3

Phase 6 (Release)
  5.* ─► 6.1, 6.2
```

---

## Estimated Effort

| Phase | Tasks | Estimated Hours |
|-------|-------|-----------------|
| Phase 1: Setup | 3 | 2h |
| Phase 2: Core Library | 6 | 8h |
| Phase 3: CLI | 3 | 4h |
| Phase 4: Testing | 4 | 4h |
| Phase 5: Documentation | 3 | 2h |
| Phase 6: Release | 2 | 2h |
| **Total** | **21** | **22h** |

---

## Checklist Before Each Task Completion

- [ ] Code compiles (`cargo check`)
- [ ] Tests pass (`cargo test`)
- [ ] Lints pass (`just lint`)
- [ ] Format applied (`just format`)
- [ ] No Chinese comments
- [ ] Documentation updated if public API changed
