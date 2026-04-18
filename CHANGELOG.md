# Changelog

All notable changes to this project should be documented in this file.

The format is inspired by Keep a Changelog, and versioning follows the repository's documented compatibility policy:
- the crate, CLI, Rust library API, and template semantics remain in the `0.x` phase
- C ABI v1 and UniFFI v1 are maintained as explicit stable contracts
- breaking changes must be called out in release notes

## [Unreleased]

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
