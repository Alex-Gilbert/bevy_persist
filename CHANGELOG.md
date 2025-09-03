# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2025-01-03

### Added

- Initial release of bevy_persist
- Automatic persistence for Bevy resources with change detection
- Support for JSON and RON serialization formats
- `#[derive(Persist)]` macro for easy resource persistence
- Auto-save on resource changes with opt-out capability
- Manual save control via `PersistManager`
- Configurable save paths per resource type
- Examples demonstrating basic and advanced usage
- Full documentation with inline examples
- Dual licensing under MIT/Apache-2.0