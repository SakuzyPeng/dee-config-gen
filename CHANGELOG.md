# Changelog

All notable changes to this project should be documented in this file.

The format is inspired by Keep a Changelog, and versioning follows the repository's documented compatibility policy:
- the crate, CLI, Rust library API, and template semantics remain in the `0.x` phase
- C ABI v1 and UniFFI v1 are maintained as explicit stable contracts
- breaking changes must be called out in release notes

## [Unreleased]

## [0.1.4] - 2026-05-02

### Added

- Added `templates list`, `templates show`, and `init` commands for template discovery and starter job generation.
- Added human-readable and JSON output for template discovery commands.

### Changed

- Expanded CLI help text for existing `generate`, `validate`, and `run` options.
- Top-level CLI errors now include the underlying error chain when available.

## [0.1.3] - 2026-05-02

### Fixed

- The CLI now exposes the standard top-level `--version` flag.

## [0.1.2] - 2026-05-01

### Added

- README installation instructions now include the crates.io `cargo install dee-config-gen` path

### Fixed

- GitHub release checksum publishing now keeps CLI, C ABI, and UniFFI bundle checksums in one combined `SHA256SUMS.txt`

### Changed

- Crate package metadata is aligned with the public release version so crates.io publishes can match GitHub tags and source archives

## [0.1.1] - 2026-04-19

### Fixed

- macOS C ABI release bundles now rewrite `libdee_config_gen.dylib` to use `@rpath/libdee_config_gen.dylib` instead of a GitHub Actions workspace absolute path
- macOS packaged-consumer verification now fails fast if a release bundle leaks the builder workspace path into linked binaries

### Changed

- GitHub release workflows now use Node 24-native versions of `actions/upload-artifact`, `actions/download-artifact`, and `softprops/action-gh-release`, removing Node 20 deprecation warnings during release runs

## [0.1.0] - 2026-04-18

### Initial Public Baseline

- CLI support for `validate`, `generate`, and `run`
- nine production templates across Atmos, PCM DDP, AC-4 IMS, and TrueHD workflows
- Rust library entrypoints for parse, validate, generate, and run flows
- stable C ABI v1 for `validate` / `generate`
- stable UniFFI v1 surface for `contract_version`, `validate_job`, and `generate_config`, currently shipped as Python bundles

### Added

- community files for contribution, security, and conduct guidance
- GitHub issue and pull request templates
- an open-source guide that documents public distribution boundaries and reproducible local workflows
- automated GitHub Actions packaging and release publishing for CLI binaries on Linux, macOS, and Windows

### Changed

- top-level README files now link contribution, security, compatibility, and changelog entry points
- crate packaging now excludes internal collaboration docs and local temp/log directories
