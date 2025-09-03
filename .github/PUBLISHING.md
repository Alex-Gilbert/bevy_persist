# Publishing Guide

This document describes how to publish new releases of bevy_persist to crates.io.

## Prerequisites

1. **Crates.io Account**: You need publish rights for both crates:
   - `bevy_persist_derive`
   - `bevy_persist`

2. **GitHub Secrets**: The repository needs these secrets configured:
   - `CARGO_REGISTRY_TOKEN`: Your crates.io API token

## Automated Release Process

The recommended way to publish is using the automated GitHub Actions workflow:

### Method 1: Tag-based Release

1. Update version numbers in both `Cargo.toml` files
2. Update `CHANGELOG.md` with release notes
3. Commit changes: `git commit -m "chore: prepare v0.1.1 release"`
4. Create tag: `git tag v0.1.1`
5. Push changes and tag: `git push && git push --tags`

The workflow will automatically:
- Run all CI checks
- Create a GitHub release
- Publish `bevy_persist_derive` to crates.io
- Wait for it to be available
- Publish `bevy_persist` to crates.io
- Create a PR to update documentation

### Method 2: Manual Workflow Dispatch

1. Go to Actions → Release workflow
2. Click "Run workflow"
3. Enter the version number (e.g., `0.1.1`)
4. Click "Run workflow"

## Manual Release Process

If you need to publish manually:

### 1. Pre-release Checklist

```bash
# Run all checks
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo doc --no-deps --all-features

# Verify package contents
cd bevy_persist_derive
cargo package --list
cd ../bevy_persist
cargo package --list
```

### 2. Update Versions

Update version in both `Cargo.toml` files:
```toml
# bevy_persist_derive/Cargo.toml
version = "0.1.1"

# bevy_persist/Cargo.toml
version = "0.1.1"
bevy_persist_derive = { version = "0.1.1", path = "../bevy_persist_derive" }
```

### 3. Update Documentation

- Update version in README.md
- Update CHANGELOG.md
- Commit changes

### 4. Publish Order (Important!)

The crates must be published in this specific order:

```bash
# 1. First publish the derive crate
cd bevy_persist_derive
cargo publish

# 2. Wait for it to be available (usually 1-2 minutes)
# Check with:
cargo search bevy_persist_derive

# 3. Then publish the main crate
cd ../bevy_persist
cargo publish
```

### 5. Create GitHub Release

1. Go to GitHub releases page
2. Click "Create a new release"
3. Tag: `v0.1.1`
4. Title: `Release v0.1.1`
5. Copy release notes from CHANGELOG.md
6. Publish release

## Version Numbering

Follow Semantic Versioning:
- MAJOR.MINOR.PATCH (e.g., 0.1.0)
- MAJOR: Breaking API changes
- MINOR: New features, backwards compatible
- PATCH: Bug fixes, backwards compatible

Pre-1.0 versions (0.x.y) may have breaking changes in minor versions.

## Troubleshooting

### "Package not found" Error

If publishing `bevy_persist` fails with a dependency error:
- Wait 1-2 minutes for crates.io to index `bevy_persist_derive`
- Try again with `cargo publish --no-verify`

### "Already published" Error

- Check the current published version: `cargo search bevy_persist`
- Ensure you've incremented the version number
- Check that you haven't already published this version

### CI Failures

Before publishing, ensure all CI checks pass:
- Format check: `cargo fmt --all`
- Clippy: `cargo clippy --all-targets --all-features -- -D warnings`
- Tests: `cargo test --all`
- Docs: `cargo doc --no-deps`

## Post-Release

After successful release:
1. Announce on relevant channels (Discord, Reddit, etc.)
2. Update any example repositories
3. Monitor for issues
4. Consider writing a blog post for significant releases