# Contributing to bevy_persist

Thank you for your interest in contributing to bevy_persist! This document provides guidelines and information for contributors.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/bevy_persist.git`
3. Create a new branch: `git checkout -b my-feature`
4. Make your changes
5. Run tests: `cargo test --all`
6. Commit your changes: `git commit -m "feat: add new feature"`
7. Push to your fork: `git push origin my-feature`
8. Open a Pull Request

## Development Setup

### Prerequisites

- Rust 1.75.0 or later
- Cargo

### Running Tests

```bash
# Run all tests
cargo test --all

# Run with all features
cargo test --all-features

# Run specific test
cargo test test_persist_data
```

### Code Quality Checks

Before submitting a PR, ensure your code passes all checks:

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --no-deps --all-features

# Check for security vulnerabilities
cargo audit
```

## Pull Request Process

1. **PR Title**: Use conventional commit format:
   - `feat:` for new features
   - `fix:` for bug fixes
   - `docs:` for documentation
   - `chore:` for maintenance
   - `test:` for tests
   - `refactor:` for refactoring

2. **Description**: Clearly describe what the PR does and why

3. **Tests**: Add tests for new functionality

4. **Documentation**: Update documentation for API changes

5. **CI Checks**: Ensure all CI checks pass:
   - Formatting (rustfmt)
   - Linting (clippy)
   - Tests on multiple platforms
   - Documentation build
   - MSRV compatibility

## Code Style

- Follow Rust standard naming conventions
- Use `cargo fmt` for formatting
- Keep functions small and focused
- Add documentation comments for public APIs
- Write tests for new functionality

## Testing

- Unit tests go in the same file as the code
- Integration tests go in `tests/` directory
- Use descriptive test names
- Test both success and failure cases

## Documentation

- Add rustdoc comments for all public items
- Include examples in documentation
- Update README if adding new features
- Update CHANGELOG.md for notable changes

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

## Release Process

Releases are automated via GitHub Actions:

1. Update version numbers in Cargo.toml files
2. Update CHANGELOG.md
3. Create and push a tag: `git tag v0.1.1 && git push --tags`
4. GitHub Actions will:
   - Run all CI checks
   - Create a GitHub release
   - Publish to crates.io

## Getting Help

- Open an issue for bugs or feature requests
- Start a discussion for questions
- Check existing issues before creating new ones

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.