# Template (tpl)

A lightweight CLI tool for scaffolding projects from GitHub repositories or local directories.

## Features

- **🚀 Simple**: Single binary, no runtime dependencies
- **🔒 Private Repo Support**: Uses system git for SSH/credential authentication
- **📁 Skip-Existing**: Never overwrites your existing files
- **🎯 Monorepo Friendly**: Supports subdirectory extraction
- **📊 Transparent**: Shows exactly which files are applied or skipped

## Installation

### From Source

```bash
cargo install --path bin/tpl
```

### Using Just

```bash
just install-tpl
```

### Alias Setup (Optional)

The binary is named `tpl` for brevity. If you prefer `template`, add an alias:

**Bash/Zsh** (add to `~/.bashrc` or `~/.zshrc`):

```bash
alias template='tpl'
```

**Fish** (add to `~/.config/fish/config.fish`):

```fish
alias template='tpl'
```

**PowerShell** (add to `$PROFILE`):

```powershell
Set-Alias -Name template -Value tpl
```

## Usage

### Basic Usage

```bash
# Clone from GitHub (default branch)
tpl user/repo

# Specify a branch or tag
tpl user/repo#main
tpl user/repo#v1.0.0

# Extract a subdirectory (great for monorepos)
tpl user/repo/templates/rust
tpl user/repo/packages/starter#main

# Use a local directory as template
tpl file://./my-templates/rust-cli
tpl file:///absolute/path/to/template

# Specify destination (default: current directory)
tpl user/repo ./my-new-project
```

### Options

| Flag | Description |
|------|-------------|
| `-s, --ssh` | Use SSH for GitHub clone (`git@github.com:...`) |
| `-v, --verbose` | Show detailed output including directory creation |
| `-h, --help` | Print help information |
| `-V, --version` | Print version |

### Examples

```bash
# Create a new Rust project from a template
tpl longcipher/rust-template ./my-app

# Use SSH for private repositories
tpl -s company/private-template

# Verbose output to see all operations
tpl -v user/repo

# Extract just the frontend from a monorepo
tpl my-org/monorepo/packages/frontend
```

## How It Works

1. **Parse**: Parses input to determine source type (GitHub or local)
2. **Fetch**: Clones repository (shallow, depth=1) or locates local directory
3. **Apply**: Copies files to destination, skipping existing ones
4. **Report**: Shows summary of applied and skipped files

### Skip-Existing Behavior

Template never overwrites existing files. This is by design:

```text
Applying template to ./my-project...
  ✓ src/main.rs
  ✓ Cargo.toml
  ⊘ .gitignore (exists)
  ⊘ README.md (exists)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✓ Done!
  Applied: 2 files
  Skipped: 2 files (already exist)
```

This allows you to safely apply templates to existing projects without losing your customizations.

## Comparison with Similar Tools

| Feature | tpl | degit | tiged |
|---------|-----|-------|-------|
| Private repos | ✅ (via system git) | ❌ | ❌ |
| SSH support | ✅ | ❌ | ❌ |
| Subdirectories | ✅ | ✅ | ✅ |
| Skip existing | ✅ | ❌ | ❌ |
| Template engine | ❌ | ❌ | ❌ |
| Single binary | ✅ | ❌ (Node.js) | ❌ (Node.js) |

## Development

```bash
# Format code
just format

# Run lints
just lint

# Run tests
just test

# Build release binary
just build-tpl

# Run with arguments
just run-tpl user/repo
```
