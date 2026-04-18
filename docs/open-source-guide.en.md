# Open-Source Guide

[User README](../README.en.md) | [中文](open-source-guide.zh.md) | [Contributing](../CONTRIBUTING.md) | [Security](../SECURITY.md)

This page explains the current open-source distribution boundaries for the repository and what outside contributors should be able to reproduce without local Dolby tooling.

## Default Reproducible Path

Public contributors should be able to run the following baseline without a local Dolby runtime:

```bash
cargo build
cargo test --workspace --quiet
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
cargo run -- generate -i examples/pcm_ddp_single.dd.yaml -o job.xml
```

That path should work without Dolby DEE, `dee-win`, or private audio sample assets.

## Manual Runtime Suite Boundary

The repository still keeps real-runtime maintenance paths, but they are intentionally outside the default public workflow:

- `tests/dee_runtime_*.rs` real DEE runtime suites
- local experiment helpers such as `scripts/ac4_native_win_probe.sh`
- runtime evidence recorded under `docs/coverage-and-experiments.*`

Rules:
- real runtime tests stay `#[ignore]`
- they are typically invoked with `cargo test --test <name> -- --ignored --nocapture`
- they may require local `dee`, `ffmpeg`, and `DEE_WORKSPACE_ROOT`
- AC-4 MP4 coverage may additionally require an AC-4 package and native mp4 muxing support

Examples:

```bash
cargo test --test dee_runtime_pcm_ddp -- --ignored --nocapture
cargo test --test dee_runtime_json -- --ignored --nocapture
```

## What Is Not Distributed With the Repository

The following are treated as local or proprietary dependencies and are not expected to ship with the public repository:

- Dolby DEE / `dee-win` runtimes and official templates
- large local audio samples under `testfiles/`
- locally exported raw XSD files, experiment caches, temp logs, and temp output directories
- internal collaboration notes or drafts that are not part of the public user/developer surface

Current repository policy:
- default tests should prefer checked-in fixtures or generated WAV/stem inputs
- a small number of manual runtime checks still depend on local Atmos sample assets such as `testfiles/testADM.wav`
- crate packaging explicitly excludes internal collaboration docs and local temp/log directories

## Contribution Expectations

- Do not commit proprietary binaries, Dolby documents, private fixtures, temp logs, or local absolute paths.
- If you change template behavior, update the matching examples, parameter matrix, coverage matrix, and knowledge/pitfall fixtures alongside the code.
- If you add a new real-runtime conclusion, prefer recording it in ignored runtime coverage or public docs instead of leaving it only in chat or local notes.
- If your change touches C ABI v1 or UniFFI v1, evaluate outward compatibility explicitly.

## Further Reading

- [Contributing](../CONTRIBUTING.md)
- [Developer Guide](developer-guide.en.md)
- [Coverage & Experiments](coverage-and-experiments.en.md)
- [FFI Bridge](ffi.en.md)
