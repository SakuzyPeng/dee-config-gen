# dee-config-gen

[![CI](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/SakuzyPeng/dee-config-gen)](LICENSE)

[中文说明](README.md) | English

Turn YAML/JSON job descriptions into Dolby Encoding Engine (DEE) XML/JSON configs, and optionally invoke an external runner to execute encoding. Stop hand-editing XML templates — describe the encoding task with structured parameters, and let the tool handle path conversion, parameter validation, and default injection.

## Installation

```bash
# Install from crates.io
cargo install dee-config-gen

# Or build from source
cargo build --release

# You can also install directly from the repository
cargo install --path .
```

GitHub Releases also publishes prebuilt CLI bundles for all three platforms:
- `dee-config-gen-cli-linux.zip`
- `dee-config-gen-cli-macos.zip`
- `dee-config-gen-cli-windows.zip`

Notes:
- the bundle only includes the CLI executable, `LICENSE`, and a short README
- it does not include Dolby DEE, `dee-win`, `ffmpeg`, or proprietary sample assets
- `validate` and `generate` work out of the box; `run` still requires your own external runner setup

## Quick Start

```bash
# List supported templates
dee-config-gen templates list

# Inspect one template's parameters and examples
dee-config-gen templates show atmos_ec3_v1

# Write a starter job.yaml
dee-config-gen init -o job.yaml

# Validate job parameters
dee-config-gen validate -i job.yaml

# Generate DEE XML config
dee-config-gen generate -i job.yaml -o job.xml

# Generate JSON config
dee-config-gen generate -i job.yaml --format json -o job.json

# Generate config and invoke runner
dee-config-gen run -i job.yaml --runner-cmd "dee" --keep-config
```

Minimal input example (`job.yaml`):

```yaml
template_id: atmos_ec3_v1
profile: standard
job_mode: single
encode_mode: streaming
input:
  storage_path: ./input
  file_names:
    - testADM.wav
output:
  storage_path: ./output
  file_names:
    - output.ec3
misc:
  temp_dir: ./tmp
```

More runnable examples: [`examples/`](examples)

## Support Matrix

| Template | Purpose | Encode Modes |
|---|---|---|
| `ac4_ims_atmos_v1` | AC-4 IMS (Atmos immersive input) | `ac4` |
| `ac4_ims_pcm_v1` | AC-4 IMS (PCM wav/wav_list input) | `ac4` |
| `atmos_ec3_v1` | Atmos DDP | `streaming` `bluray` |
| `pcm_ddp_v1` | PCM to DD/DDP | `dd` `ddp` `ddp71` `bluray` |
| `thd_v1` | TrueHD (Atmos mezz input) | `mlp` |
| `thd_wav_v1` | TrueHD (single WAV input) | `mlp` |
| `thd_wav_list_v1` | TrueHD (mono stems input) | `mlp` |
| `thd_atmos_wav_v1` | TrueHD (Atmos + WAV mixed input) | `mlp` |
| `thd_atmos_wav_list_v1` | TrueHD (Atmos + stems mixed input) | `mlp` |

All 9 templates support both XML and JSON output, verified against real DEE runtime.

Not sure which template to use? See the [Template Selection Guide](docs/template-guide.en.md).

## Integration

Beyond the CLI:

- **Rust Library API** — `read_job` / `parse_job_str` -> `generate_config`, see [API docs](https://docs.rs/dee-config-gen)
- **C ABI (FFI v1)** — `dcg_validate_job` / `dcg_generate_config` with stable status codes, see [FFI docs](docs/ffi.en.md)
- **UniFFI Python bundle** — cross-platform downloadable bundles, see [`ffi/example_python_uniffi`](ffi/example_python_uniffi)

## Versioning and Compatibility

- The crate is still in the `0.x` phase; the CLI, Rust library API, and template semantics continue to evolve and are not frozen at the same level as the FFI contracts
- C ABI v1 and UniFFI v1 remain explicitly maintained stable contracts, and breaking changes there require version bumps
- Every breaking change must be called out in [CHANGELOG.md](CHANGELOG.md) and the matching release notes

## Contributing and Open-Source Boundaries

- Contribution workflow, default verification commands, PR expectations, and doc sync rules live in [CONTRIBUTING.md](CONTRIBUTING.md)
- Distribution boundaries, the relationship between default `cargo test` and ignored runtime suites, and the `testfiles/` / `dee-win` / Dolby DEE requirements are documented in the [Open-Source Guide](docs/open-source-guide.en.md)
- Report security issues through [SECURITY.md](SECURITY.md), and follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for community interactions

## Notes

- `profile=music` locks a fixed set of parameter values; use `--allow-fixed-override` to override
- Generated XML paths are converted to Windows format (default `Y:` drive, change with `--win-drive`)
- Runner priority: `--runner-cmd` > `DEE_RUNNER_CMD` env var > `dee`
- `--format json` automatically injects `--json` into the runner
- Default `cargo test --workspace --quiet` does not require a local Dolby runtime; see the open-source guide for ignored runtime suites and proprietary fixture boundaries

## Documentation

| Category | 中文 | English |
|---|---|---|
| Template Guide | [template-guide.zh.md](docs/template-guide.zh.md) | [template-guide.en.md](docs/template-guide.en.md) |
| Open-Source Guide | [open-source-guide.zh.md](docs/open-source-guide.zh.md) | [open-source-guide.en.md](docs/open-source-guide.en.md) |
| Developer Guide | [developer-guide.zh.md](docs/developer-guide.zh.md) | [developer-guide.en.md](docs/developer-guide.en.md) |
| FFI Bridge | [ffi.zh.md](docs/ffi.zh.md) | [ffi.en.md](docs/ffi.en.md) |
| JSON Output | [json-output.zh.md](docs/json-output.zh.md) | [json-output.en.md](docs/json-output.en.md) |
| Coverage & Experiments | [coverage-and-experiments.zh.md](docs/coverage-and-experiments.zh.md) | [coverage-and-experiments.en.md](docs/coverage-and-experiments.en.md) |
| Parameter Matrix | [`docs/parameter_matrix.*.yaml`](docs/) | |
| Coverage Matrix | [`docs/coverage_matrix.full.yaml`](docs/coverage_matrix.full.yaml) | |
| Changelog | [CHANGELOG.md](CHANGELOG.md) | [CHANGELOG.md](CHANGELOG.md) |

## License

[MIT](LICENSE)

Runtime dependencies such as Dolby DEE, `dee-win`, upstream fixtures, and third-party tooling have their own licenses and usage restrictions.
