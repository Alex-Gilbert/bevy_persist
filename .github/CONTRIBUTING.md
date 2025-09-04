# Contributing to bevy_persist

Thank you for your interest in contributing to bevy_persist! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/bevy_persist.git`
3. Create a new branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test --workspace --all-features`
6. Commit your changes: `git commit -m "feat: add new feature"`
7. Push to your fork: `git push origin my-feature`
8. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75.0 or later (check `rust-version` in Cargo.toml)
- Cargo

### Running Tests

```bash
# Run all tests with default features
cargo test --workspace

# Run with all features (including secure encryption)
cargo test --workspace --all-features

# Run tests in production mode
cargo test --workspace --no-default-features --features prod

# Run tests with secure feature
cargo test --workspace --no-default-features --features prod,secure

# Run specific test
cargo test test_persist_data
```

### Building Examples

```bash
# Development mode (default)
cargo run --example basic
cargo run --example advanced

# Production mode
cargo run --example basic --no-default-features --features prod
cargo run --example advanced --no-default-features --features prod,secure
```

### Code Quality Checks

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -W warnings

# Build documentation
cargo doc --no-deps --all-features

# Check for security vulnerabilities
cargo audit

# Verify MSRV (Minimum Supported Rust Version: 1.75)
rustup run 1.75.0 cargo check --all-features --workspace
```

## Features

bevy_persist has several feature flags:

- `dev` (default): Development mode with local file persistence
- `prod`: Production mode with platform-specific paths
- `secure`: Adds AES-256-GCM encryption support for save data

When testing changes, ensure they work with different feature combinations:

```bash
# Test default (dev) features
cargo test

# Test production mode
cargo test --no-default-features --features prod

# Test with encryption
cargo test --no-default-features --features prod,secure
```

## Pull Request Process

1. **PR Title**: Use conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `chore:` for maintenance
   - `test:` for tests
   - `refactor:` for refactoring
   - `perf:` for performance improvements

2. **Description**: Clearly describe what the PR does and why

3. **Tests**: Add tests for new functionality

4. **Documentation**: Update documentation for API changes

5. **CI Checks**: Ensure all CI checks pass:
   - Formatting (rustfmt)
   - Linting (clippy)
   - Tests on multiple platforms (Linux, Windows, macOS)
   - Documentation build
   - MSRV compatibility (1.75)
   - Security audit

## Code Style

- Follow Rust standard naming conventions
- Use `cargo fmt` for formatting
- Keep functions small and focused
- Add documentation comments for public APIs
- Write tests for new functionality
- Avoid deprecated Bevy APIs (e.g., use `EventWriter::write` instead of `send`)

## Testing

- Unit tests go in the same file as the code (in `#[cfg(test)]` modules)
- Integration tests go in `tests/` directory
- Use descriptive test names
- Test both success and failure cases
- Test with different feature flag combinations

## Documentation

- Add rustdoc comments for all public items
- Include examples in documentation
- Update README if adding new features
- Update CHANGELOG.md for notable changes
- Ensure examples compile and run correctly

## Commit Messages

Follow conventional commits:

```
<type>(<scope>): <subject>

<body>

<footer>
```

Example:
```
feat(persist): add support for TOML format

Add TOML as a third serialization format option alongside JSON and RON.
This allows users to choose their preferred configuration format.

Closes #123
```

## Workspace Structure

This is a Cargo workspace with two crates:

- `bevy_persist`: Main crate with persistence functionality
- `bevy_persist_derive`: Procedural macro for the `#[derive(Persist)]` attribute

When making changes that affect both crates, ensure version numbers stay in sync.

## Release Process

Releases are automated via GitHub Actions:

1. Update version numbers in:
   - `bevy_persist/Cargo.toml`
   - `bevy_persist_derive/Cargo.toml`
   - Update dependency version in `bevy_persist/Cargo.toml`

2. Update CHANGELOG.md with the new version and changes

3. Create and push a tag: `git tag v0.1.1 && git push --tags`

4. GitHub Actions will:
   - Run all CI checks
   - Create a GitHub release
   - Publish to crates.io (first derive crate, then main crate)

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Check existing issues before creating new ones
- Join the Bevy Discord for general Bevy-related questions

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.