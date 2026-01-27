# Default recipe to display help
default:
  @just --list

# Format all code
format:
  rumdl fmt .
  taplo fmt
  cargo +nightly fmt --all

# Auto-fix linting issues
fix:
  rumdl check --fix .

# Run all lints
lint:
  typos
  rumdl check .
  taplo fmt --check
  cargo +nightly fmt --all -- --check
  cargo +nightly clippy --all -- -D warnings
  cargo machete

# Run tests
test:
  cargo test --all-features

# Run tests with coverage
test-coverage:
  cargo tarpaulin --all-features --workspace --timeout 300

# Check for Chinese characters
check-cn:
  rg --line-number --column "\p{Han}"

# Full CI check
ci: lint test

# Generate docs
docs:
  cargo doc --no-deps --open

# Check all targets compile
check:
  cargo check --all-targets --all-features

# ============================================================
# Template CLI Commands
# ============================================================

# Build the tpl binary in release mode
build-tpl:
  cargo build -p tpl --release

# Run tpl with arguments
run-tpl *ARGS:
  cargo run -p tpl -- {{ARGS}}

# Install tpl locally
install-tpl:
  cargo install --path bin/tpl

# Show where tpl is installed
which-tpl:
  which tpl || echo "tpl not found in PATH"