# AGENTS.md - Development Guide for rules_rust

## Overview

This is a Bazel rules repository for building Rust projects. Uses bzlmod (MODULE.bazel) by default, with legacy WORKSPACE support available.

## Build Commands

```bash
# Build everything
bazel build //...

# Build with clippy linting enabled
bazel build --config=clippy //...

# Build with rustfmt checking enabled
bazel build --config=rustfmt //...

# Build with both linting and formatting
bazel build --config=clippy --config=rustfmt //...

# Build with nightly toolchain
bazel build --config=nightly //...

# Build in opt mode
bazel build --compilation_mode=opt //...

# Build without bzlmod (legacy WORKSPACE mode)
bazel build --config=no-bzlmod //...
```

## Test Commands

```bash
# Run all tests
bazel test //...

# Run tests with aspects (clippy + rustfmt)
bazel test --config=clippy --config=rustfmt //...

# Run specific test
bazel test //test/unit/...:all

# Run coverage
bazel coverage //...
```

## process_wrapper

The process_wrapper is a Rust binary in `util/process_wrapper/` that wraps rustc invocations.

```bash
# Build process_wrapper
bazel build //util/process_wrapper:process_wrapper

# Run process_wrapper tests
bazel test //util/process_wrapper:process_wrapper_test
```

## Linting and Formatting

### Rust

Clippy and rustfmt are run via Bazel aspects:

```bash
# Check formatting
bazel build --config=rustfmt //...

# Run clippy
bazel build --config=clippy //...
```

Config files:
- `.rustfmt.toml`: rustfmt configuration (uses `skip_children = true`)
- `.clippy.toml`: clippy configuration

### Starlark/Bazel Files

Uses buildifier for `.bzl` and `BUILD.bazel` files:

```bash
# Via pre-commit (recommended)
pre-commit run buildifier --all-files
pre-commit run buildifier-lint --all-files
```

### Other Linters

Pre-commit hooks configured in `.pre-commit-config.yaml`:
- **buildifier**: Bazel file formatting (v8.2.0)
- **typos**: Spell checking
- **black**: Python formatting
- **isort**: Python import sorting
- **clang-format**: C/C++ formatting

```bash
# Run all pre-commit hooks
pre-commit run --all-files

# Install pre-commit hooks
pre-commit install
```

## Code Style Conventions

### Rust
- Edition: 2018 (for core tools like process_wrapper)
- See `.rustfmt.toml` for formatting rules

### Starlark
- Follow Bazel style guide
- Use buildifier for formatting with all warnings enabled

### JavaScript/TypeScript
- 4-space indentation
- Trailing commas (see `.prettierrc.toml`)

## Directory Structure

- `//rust`: Core rules (rust_binary, rust_library, etc.)
- `//rust/private`: Internal implementation (do not depend on externally)
- `//cargo`: Cargo integration rules
- `//crate_universe`: Crate dependency management
- `//test`: Unit and integration tests
- `//test/unit`: Unit tests
- `//examples`: Usage examples (separate local repository)
- `//util/process_wrapper`: Rustc process wrapper
- `//extensions`: Optional extensions (bindgen, prost, protobuf, pyo3, wasm_bindgen, mdbook)
- `//docs`: Documentation

## CI Configuration

CI runs on Buildkite, configured in `.bazelci/presubmit.yml`.

Key CI configurations:
- Minimum Bazel version: 7.4.1
- Tests run on: Ubuntu, macOS, Windows
- Tests run with and without bzlmod
- Aspects (clippy + rustfmt) tested separately

## Useful Targets

```bash
# Generate rust-analyzer project
bazel run //tools/rust_analyzer:gen_rust_project

# Generate VS Code launch.json
bazel run //tools/vscode:gen_launch_json
```

## Testing Examples

Examples are in a separate local repository:

```bash
# Test crate_universe examples
cd examples/crate_universe
bazel test //...

# Test FFI examples
cd examples/ffi
bazel test //...

# Test hello_world with bzlmod
cd examples/hello_world
bazel test //...
```

## Extensions

Each extension has its own workspace in `extensions/`:

```bash
# Build/test bindgen extension
cd extensions/bindgen
bazel test //...

# Build/test prost extension
cd extensions/prost
bazel test //...
```
