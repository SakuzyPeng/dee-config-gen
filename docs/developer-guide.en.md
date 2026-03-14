# Developer Guide

[User README](../README.en.md) | [中文](developer-guide.zh.md) | [JSON Output](json-output.en.md) | [Coverage & Experiments](coverage-and-experiments.en.md)

This page is for maintainers and contributors. It focuses on architecture, repository layout, developer commands, and doc maintenance rules.

## Architecture Overview

Current flow:

1. `spec` handles input serde and path normalization
2. `resolve` handles defaults, validation, and constraints
3. `template` owns template registration, schema, and XML/JSON structure
4. `render` emits XML or JSON based on `RenderFormat`
5. `runner` invokes the external runtime command with the generated config file
6. `media` probes input media for input-sensitive validation

## Template System

Current production templates:
- `ac4_ims_atmos_v1`
  - mode: `ac4`
- `ac4_ims_pcm_v1`
  - mode: `ac4`
- `atmos_ec3_v1`
  - modes: `streaming`, `bluray`
- `pcm_ddp_v1`
  - modes: `dd`, `ddp`, `ddp71`, `bluray`
- `thd_v1`
- `thd_wav_v1`
- `thd_wav_list_v1`
- `thd_atmos_wav_v1`
- `thd_atmos_wav_list_v1`
  - mode: `mlp`

Each template owns:
- parameter schema
- defaults
- constraints
- template-specific filter struct
- XML structure generation
- JSON structure generation when that template supports JSON output
- runtime compatibility guards when needed

Current JSON output policy:
- `RenderFormat::Xml` and `RenderFormat::Json` run in parallel
- XML remains the default
- native JSON output is currently implemented for `atmos_ec3_v1`, `pcm_ddp_v1`, and all TrueHD templates
- `ac4_ims_atmos_v1` and `ac4_ims_pcm_v1` intentionally remain XML-only for now

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
- [`../src/template/ac4_ims_shared.rs`](../src/template/ac4_ims_shared.rs)
- [`../src/template/ac4_ims_atmos_v1/`](../src/template/ac4_ims_atmos_v1)
- [`../src/template/ac4_ims_pcm_v1/`](../src/template/ac4_ims_pcm_v1)
- [`../src/template/atmos_ec3_v1/`](../src/template/atmos_ec3_v1)
- [`../src/template/pcm_ddp_v1/`](../src/template/pcm_ddp_v1)
- [`../src/template/thd_v1/`](../src/template/thd_v1)
- [`../docs/parameter_matrix.ac4_ims_atmos_v1.yaml`](../docs/parameter_matrix.ac4_ims_atmos_v1.yaml)
- [`../docs/parameter_matrix.ac4_ims_pcm_v1.yaml`](../docs/parameter_matrix.ac4_ims_pcm_v1.yaml)
- [`../docs/ac4-official-notes.en.md`](../docs/ac4-official-notes.en.md)
- [`../docs/parameter_matrix.atmos_ec3_v1.yaml`](../docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`../docs/parameter_matrix.pcm_ddp_v1.yaml`](../docs/parameter_matrix.pcm_ddp_v1.yaml)
- [`../docs/parameter_matrix.thd_v1.yaml`](../docs/parameter_matrix.thd_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_v1.yaml`](../docs/parameter_matrix.thd_wav_v1.yaml)
- [`../docs/parameter_matrix.thd_wav_list_v1.yaml`](../docs/parameter_matrix.thd_wav_list_v1.yaml)
- [`../docs/parameter_matrix.thd_atmos_wav_v1.yaml`](../docs/parameter_matrix.thd_atmos_wav_v1.yaml)
- [`../docs/parameter_matrix.thd_atmos_wav_list_v1.yaml`](../docs/parameter_matrix.thd_atmos_wav_list_v1.yaml)

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

### Template Truth Sources

When adding or changing a production template, keep these files in sync:
- `docs/parameter_matrix.<template>.yaml`
- `docs/coverage_matrix.full.yaml`
- `tests/fixtures/upstream_knowledge.json`
- `docs/json-output.zh.md` / `docs/json-output.en.md` when JSON support changes
- at least one `examples/` spec
- the matching XSD contract fixture and smoke test

Add capability-specific evidence as needed:
- if the template has real runtime conclusions, add `#[ignore]` runtime coverage
- if runtime exposes a mismatch or restriction, add an `upstream_pitfalls.*.json` fixture and its test

Status rules:
- `covered`
  - requires schema / XSD / runtime / knowledge evidence
- `conservative_gap`
  - requires at least schema + knowledge
  - use only when runtime coverage is partial, or the local model intentionally stays narrower or wider than runtime
- `unsupported_or_hidden`
  - requires knowledge evidence
  - if runtime explicitly rejects the capability, also record it in a pitfall or equivalent runtime note

The repository now includes a template-level consistency test that checks:
- every template registered in `src/template/mod.rs` also exists in the coverage matrix, parameter matrix, XSD contract fixtures, examples, and knowledge
- every `unsupported_or_hidden` parameter has a matching description in `upstream_knowledge.json`
