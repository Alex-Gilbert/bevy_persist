# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-04

### Added

- Initial release of bevy_persist
- Automatic persistence for Bevy resources with change detection
- Support for JSON and RON serialization formats
- `#[derive(Persist)]` macro for easy resource persistence
- Multiple persistence modes:
  - `dev` (default): Local file persistence for development
  - `dynamic`: Platform-specific user config directories
  - `embed`: Compile-time embedding of values into binary
  - `secure`: AES-256-GCM encrypted save data
- Auto-save on resource changes with opt-out capability
- Manual save control via `PersistManager`
- Platform-specific save paths in production mode
- Security features with optional encryption
- Examples demonstrating basic and advanced usage
- Full test coverage with integration tests
- CI/CD pipeline with GitHub Actions
- Comprehensive documentation
- MSRV (Minimum Supported Rust Version): 1.75
- Dual licensing under MIT/Apache-2.0