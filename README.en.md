# dee-config-gen

[![CI](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml/badge.svg)](https://github.com/SakuzyPeng/dee-config-gen/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/github/license/SakuzyPeng/dee-config-gen)](LICENSE)

[中文说明](README.md) | English

Turn YAML/JSON job descriptions into [Dolby Encoding Engine](https://professional.dolby.com/) (DEE) XML/JSON configs, and optionally invoke an external runner to execute encoding. Stop hand-editing XML templates — describe the encoding task with structured parameters, and let the tool handle path conversion, parameter validation, and default injection.

## Installation

```bash
# Build from source
cargo build --release

# Or install directly
cargo install --path .
```

## Quick Start

```bash
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

## Notes

- `profile=music` locks a fixed set of parameter values; use `--allow-fixed-override` to override
- Generated XML paths are converted to Windows format (default `Y:` drive, change with `--win-drive`)
- Runner priority: `--runner-cmd` > `DEE_RUNNER_CMD` env var > `dee`
- `--format json` automatically injects `--json` into the runner

## Documentation

| Category | 中文 | English |
|---|---|---|
| Template Guide | [template-guide.zh.md](docs/template-guide.zh.md) | [template-guide.en.md](docs/template-guide.en.md) |
| Developer Guide | [developer-guide.zh.md](docs/developer-guide.zh.md) | [developer-guide.en.md](docs/developer-guide.en.md) |
| FFI Bridge | [ffi.zh.md](docs/ffi.zh.md) | [ffi.en.md](docs/ffi.en.md) |
| JSON Output | [json-output.zh.md](docs/json-output.zh.md) | [json-output.en.md](docs/json-output.en.md) |
| Coverage & Experiments | [coverage-and-experiments.zh.md](docs/coverage-and-experiments.zh.md) | [coverage-and-experiments.en.md](docs/coverage-and-experiments.en.md) |
| Parameter Matrix | [`docs/parameter_matrix.*.yaml`](docs/) | |
| Coverage Matrix | [`docs/coverage_matrix.full.yaml`](docs/coverage_matrix.full.yaml) | |

## License

[MIT](LICENSE)

Runtime dependencies such as Dolby DEE, `dee-win`, upstream fixtures, and third-party tooling have their own licenses and usage restrictions.
