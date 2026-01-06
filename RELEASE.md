# Release Process

This document describes how to create a new release for Scratchpad.

## Versioning

This project follows [Semantic Versioning](https://semver.org/):
- **MAJOR** (x.0.0): Breaking changes
- **MINOR** (0.x.0): New features, backwards compatible
- **PATCH** (0.0.x): Bug fixes, backwards compatible

## Release Steps

### 1. Prepare the release branch

Ensure all changes for the release are merged to `main`:

```bash
git checkout main
git pull origin main
git merge <feature-branch>
```

### 2. Update version numbers

Update the version in `Cargo.toml` in two places:

```toml
[package]
version = "X.Y.Z"

[package.metadata.bundle]
version = "X.Y.Z"
```

### 3. Commit the version bump

```bash
git add Cargo.toml
git commit -m "Bump version to X.Y.Z"
```

### 4. Create an annotated tag

```bash
git tag -a vX.Y.Z -m "vX.Y.Z: Brief description of changes"
```

### 5. Push to remote

```bash
git push origin main
git push origin vX.Y.Z
```

## Building Release Artifacts

### Linux

```bash
cargo build --release
```

The binary will be at `target/release/scratchpad`.

### macOS

```bash
cargo build --release
cargo bundle --release
```

The app bundle will be at `target/release/bundle/osx/Scratchpad.app`.

## Post-Release

After pushing the tag, consider:
- Creating a GitHub release with release notes
- Updating any distribution channels
