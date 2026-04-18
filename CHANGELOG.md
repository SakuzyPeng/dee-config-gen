# Changelog

All notable changes to this project should be documented in this file.

The format is inspired by Keep a Changelog, and versioning follows the repository's documented compatibility policy:
- the crate, CLI, Rust library API, and template semantics remain in the `0.x` phase
- C ABI v1 and UniFFI v1 are maintained as explicit stable contracts
- breaking changes must be called out in release notes

## [Unreleased]

### Added

- community files for contribution, security, and conduct guidance
- GitHub issue and pull request templates
- an open-source guide that documents public distribution boundaries and reproducible local workflows

### Changed

- top-level README files now link contribution, security, compatibility, and changelog entry points
- crate packaging now excludes internal collaboration docs and local temp/log directories

## [0.1.0] - 2026-04-18

### Initial Public Baseline

- CLI support for `validate`, `generate`, and `run`
- nine production templates across Atmos, PCM DDP, AC-4 IMS, and TrueHD workflows
- Rust library entrypoints for parse, validate, generate, and run flows
- stable C ABI v1 for `validate` / `generate`
- stable UniFFI v1 surface for `contract_version`, `validate_job`, and `generate_config`, currently shipped as Python bundles
