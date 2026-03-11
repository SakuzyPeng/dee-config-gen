# Developer Guide

[User README](../README.en.md) | [中文](developer-guide.zh.md) | [Coverage & Experiments](coverage-and-experiments.en.md)

This page is for maintainers and contributors. It focuses on architecture, repository layout, developer commands, and doc maintenance rules.

## Architecture Overview

Current flow:

1. `config` handles input serde and path normalization
2. `resolve` handles defaults, validation, and constraints
3. `template` owns template registration, schema, and XML structure
4. `render` converts `XmlNode` trees into XML text
5. `runner` invokes the external runtime command
6. `media` probes input media for input-sensitive validation

## Template System

Current production templates:
- `atmos_ec3_v1`
  - modes: `streaming`, `bluray`
- `pcm_ddp_v1`
  - modes: `dd`, `ddp`, `ddp71`, `bluray`

Each template owns:
- parameter schema
- defaults
- constraints
- template-specific filter struct
- XML structure generation
- runtime compatibility guards when needed

Primary entry points:
- [`../src/template/mod.rs`](../src/template/mod.rs)
- [`../src/resolve.rs`](../src/resolve.rs)

## Repository Layout

Key directories:
- `src/`: CLI, resolve pipeline, templates, renderer, runner
- `examples/`: runnable example specs
- `docs/`: parameter matrices, coverage matrix, experiments, developer docs
- `tests/`: schema/XSD/example/runtime/pitfall layers
- `scripts/`: XSD extraction, upstream sync, experiment helpers

Useful starting points:
- [`../src/template/atmos_ec3_v1/`](../src/template/atmos_ec3_v1)
- [`../src/template/pcm_ddp_v1/`](../src/template/pcm_ddp_v1)
- [`../docs/parameter_matrix.atmos_ec3_v1.yaml`](../docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`../docs/parameter_matrix.pcm_ddp_v1.yaml`](../docs/parameter_matrix.pcm_ddp_v1.yaml)

## Common Developer Commands

Build:

```bash
cargo build
```

Default test suite:

```bash
cargo test
```

Pre-commit checks:

```bash
scripts/precommit_checks.sh
```

Quick command smoke:

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
cargo run -- generate -i examples/pcm_ddp_single.dd.yaml -o job.xml
```

## Test Layers

The repository currently uses five layers:
- schema / resolve
- XSD contract
- example regression
- runtime regression
- pitfall / knowledge

Rules:
- schema/XSD/example tests belong in default `cargo test`
- real DEE runtime tests stay `#[ignore]` and are triggered manually
- pitfall and knowledge fixtures preserve known differences so they do not live only in memory

Runtime entry points are documented in:
- [`coverage-and-experiments.en.md`](coverage-and-experiments.en.md)

## Documentation Maintenance Rules

Homepages and developer docs have different jobs:
- `README.md` and `README.en.md` are user-facing only
- development, coverage, and experiment details live under `docs/`

When updating docs:
- template schema and matrix documents remain the parameter source of truth
- every new runtime conclusion must update both the coverage matrix and the knowledge fixture
- the homepage should only answer what the tool is, how to use it, and where to go deeper
