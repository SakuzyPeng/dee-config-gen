# dee-config-gen

[中文说明](README.md) | English README | [FFI Bridge](docs/ffi.en.md) | [JSON Output](docs/json-output.en.md) | [Developer Guide](docs/developer-guide.en.md) | [Coverage & Experiments](docs/coverage-and-experiments.en.md)

`dee-config-gen` is a Rust CLI for validating job specs, generating DEE XML/JSON configs, and optionally invoking an external runner.

It now also ships a formal Rust library API. Recommended entrypoints:
- file input: `read_job -> generate_config`
- in-memory input: `parse_job_str -> generate_config`
- staged orchestration: `resolve_job -> render_config -> run_with_runner`

Good fit when you want to:
- describe jobs in YAML or JSON and generate stable DEE XML/JSON configs
- validate parameters locally before calling `dee`
- manage Atmos, PCM DDP, TrueHD Atmos-input, and TrueHD WAV-input templates in one tool

## Quick Overview

Current support:
- Templates: `ac4_ims_atmos_v1`, `ac4_ims_pcm_v1`, `atmos_ec3_v1`, `pcm_ddp_v1`, `thd_v1`, `thd_wav_v1`, `thd_wav_list_v1`, `thd_atmos_wav_v1`, `thd_atmos_wav_list_v1`
- Input: YAML, JSON
- Commands: `validate`, `generate`, `run`
- Output formats: default `xml`; `json` is currently supported for `ac4_ims_atmos_v1`, `ac4_ims_pcm_v1`, `atmos_ec3_v1`, `pcm_ddp_v1`, and all TrueHD templates; the AC-4 lanes also support `output.container=ac4|mp4` on the JSON path
- FFI (C ABI v1): stateless `validate/generate` with stable status codes plus UTF-8 error messages
- AC-4 mode: `ac4`
- Atmos modes: `streaming`, `bluray`
- PCM modes: `dd`, `ddp`, `ddp71`, `bluray`
- TrueHD mode: `mlp`

### Support Matrix At A Glance

| Template | XML Output | JSON Output | Real DEE runtime |
| --- | --- | --- | --- |
| `ac4_ims_atmos_v1` | Supported | Supported | Verified |
| `ac4_ims_pcm_v1` | Supported | Supported | Verified |
| `atmos_ec3_v1` | Supported | Supported | Verified |
| `pcm_ddp_v1` | Supported | Supported | Verified |
| `thd_v1` | Supported | Supported | Verified |
| `thd_wav_v1` | Supported | Supported | Verified |
| `thd_wav_list_v1` | Supported | Supported | Verified |
| `thd_atmos_wav_v1` | Supported | Supported | Verified |
| `thd_atmos_wav_list_v1` | Supported | Supported | Verified |

For parameter-level coverage and edge cases, see:
- [`docs/coverage_matrix.full.yaml`](docs/coverage_matrix.full.yaml)
- [`docs/json-output.en.md`](docs/json-output.en.md)

## 5-Minute Start

Build:

```bash
cargo build
```

Validate a spec:

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
```

Generate XML:

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml -o job.xml
```

Generate JSON:

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml --format json -o job.json
```

Generate XML and call an external runner:

```bash
cargo run -- run \
  -i examples/atmos_ec3_single.streaming.yaml \
  --runner-cmd "dee" \
  --keep-config
```

Notes:
- `run` does not embed container or Wine logic
- priority order is `--runner-cmd` -> `DEE_RUNNER_CMD` -> `dee`
- `--format json` automatically injects `--json` into the runner

## As A Library

Read from a file and generate XML:

```rust
use dee_config_gen::{GenerateOptions, generate_config, read_job};

let spec = read_job("examples/atmos_ec3_single.streaming.yaml".as_ref())?;
let generated = generate_config(spec, &GenerateOptions::default())?;
assert!(generated.rendered.starts_with("<?xml version=\"1.0\"?>"));
# Ok::<(), anyhow::Error>(())
```

Parse from a string and generate JSON:

```rust
use dee_config_gen::{GenerateOptions, RenderFormat, generate_config, parse_job_str};

let spec = parse_job_str(r#"{
  "template_id": "atmos_ec3_v1",
  "profile": "standard",
  "job_mode": "single",
  "encode_mode": "streaming",
  "input": {"storage_path": "./input", "file_names": ["testADM.wav"]},
  "output": {"storage_path": "./output", "file_names": ["output.ec3"]},
  "misc": {"temp_dir": "./tmp"}
}"#)?;

let generated = generate_config(
    spec,
    &GenerateOptions {
        format: RenderFormat::Json,
        ..GenerateOptions::default()
    },
)?;
assert!(generated.rendered.contains("\"job_config\""));
# Ok::<(), anyhow::Error>(())
```

## As FFI (C ABI v1)

FFI v1 entrypoints:
- `dcg_validate_job`
- `dcg_generate_config`

Protocol guarantees:
- status codes: `OK / INVALID_ARGUMENT / PARSE_ERROR / RESOLVE_ERROR / RENDER_ERROR / INTERNAL_ERROR / PANIC`
- error text is UTF-8 and diagnostic only; branch by status code
- returned strings are Rust-owned; callers must release with `dcg_free_*`

See:
- [`docs/ffi.en.md`](docs/ffi.en.md)
- [`include/dee_config_gen_ffi.h`](include/dee_config_gen_ffi.h)
- CMake consumer example: [`ffi/example_cmake`](ffi/example_cmake)
- Python pilot wrapper: [`ffi/example_python`](ffi/example_python)
- Release assets (`v*` tags): GitHub Releases publishes 3-platform dynamic library bundles plus `SHA256SUMS.txt`

## Minimal Input Example

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

## Which Template Should I Use?

### `ac4_ims_atmos_v1`

Use it when you need:
- official `encode_to_ims_ac4` XML that takes `inputs.atmos_mezz` and emits `.ac4` or `.mp4`
- an Atmos mezzanine / ADM / IAB input family
- XML/JSON AC-4 immersive stereo output, with `output.container=ac4|mp4` runtime-smoked locally

Examples:
- [`examples/ac4_ims_atmos_single.ac4.yaml`](examples/ac4_ims_atmos_single.ac4.yaml)
- [`examples/ac4_ims_atmos_single.mp4.yaml`](examples/ac4_ims_atmos_single.mp4.yaml)

### `ac4_ims_pcm_v1`

Use it when you need:
- official `encode_to_ims_ac4` XML that takes `inputs.wav` or `inputs.wav_list` and emits `.ac4` or `.mp4`
- a PCM-family AC-4 immersive stereo workflow
- XML/JSON output, with `output.container=ac4|mp4` runtime-smoked locally

Examples:
- [`examples/ac4_ims_pcm_single.ac4.yaml`](examples/ac4_ims_pcm_single.ac4.yaml)
- [`examples/ac4_ims_pcm_single.mp4.yaml`](examples/ac4_ims_pcm_single.mp4.yaml)

### `atmos_ec3_v1`

Use it when you need:
- Atmos DDP XML
- `streaming` or `bluray` Atmos workflows
- DEE JSON output is currently supported

Examples:
- [`examples/atmos_ec3_single.streaming.yaml`](examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](examples/atmos_ec3_single.bluray.yaml)

### `pcm_ddp_v1`

Use it when you need:
- `pcm_to_ddp` XML for DD or DDP workflows
- one of `dd`, `ddp`, `ddp71`, or `bluray`
- DEE JSON output support for the same workflow

Examples:
- [`examples/pcm_ddp_single.dd.yaml`](examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](examples/pcm_ddp_single.bluray.yaml)

### `thd_v1`

Use it when you need:
- official `encode_to_dthd` XML/JSON for TrueHD MLP output
- `atmos_mezz` input and `mlp` output

Examples:
- [`examples/thd_single.mlp.yaml`](examples/thd_single.mlp.yaml)

### `thd_wav_v1`

Use it when you need:
- single-file `wav -> mlp` TrueHD jobs
- one WAV input instead of a stem list

Examples:
- [`examples/thd_wav_single.mlp.yaml`](examples/thd_wav_single.mlp.yaml)

### `thd_wav_list_v1`

Use it when you need:
- ordered mono stems on the `wav_list -> mlp` TrueHD path
- fixed slot order in `input.file_names`
- the currently supported runtime-aligned layouts: `stereo`, `5.1`, and `7.1`

Examples:
- [`examples/thd_wav_list_single.mlp.yaml`](examples/thd_wav_list_single.mlp.yaml)

### `thd_atmos_wav_v1`

Use it when you need:
- the mixed-input `atmos_mezz + wav -> mlp` TrueHD path
- separate `inputs.atmos_mezz` and `inputs.wav` groups in one job

Examples:
- [`examples/thd_atmos_wav_single.mlp.yaml`](examples/thd_atmos_wav_single.mlp.yaml)

### `thd_atmos_wav_list_v1`

Use it when you need:
- the mixed-input `atmos_mezz + wav_list -> mlp` TrueHD path
- separate `inputs.atmos_mezz` and `inputs.wav_list` groups in one job

Examples:
- [`examples/thd_atmos_wav_list_single.mlp.yaml`](examples/thd_atmos_wav_list_single.mlp.yaml)

## Common Notes

- `profile=music` locks a fixed set of values by default; use `--allow-fixed-override` only when you really need it
- generated XML normalizes paths to Windows-style paths; default drive is `Y:` and can be changed with `--win-drive`
- `ac4_v1` has been removed; migrate to `ac4_ims_atmos_v1` or `ac4_ims_pcm_v1`
- `ac4_ims_atmos_v1`, `ac4_ims_pcm_v1`, `atmos_ec3_v1`, `pcm_ddp_v1`, `thd_v1`, `thd_wav_v1`, `thd_wav_list_v1`, `thd_atmos_wav_v1`, and `thd_atmos_wav_list_v1` are separate templates with separate parameter sets

## Documentation

User-facing references:
- [`docs/parameter_matrix.ac4_ims_atmos_v1.yaml`](docs/parameter_matrix.ac4_ims_atmos_v1.yaml)
- [`docs/parameter_matrix.ac4_ims_pcm_v1.yaml`](docs/parameter_matrix.ac4_ims_pcm_v1.yaml)
- [`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`docs/parameter_matrix.pcm_ddp_v1.yaml`](docs/parameter_matrix.pcm_ddp_v1.yaml)
- [`docs/parameter_matrix.thd_v1.yaml`](docs/parameter_matrix.thd_v1.yaml)
- [`docs/parameter_matrix.thd_wav_v1.yaml`](docs/parameter_matrix.thd_wav_v1.yaml)
- [`docs/parameter_matrix.thd_wav_list_v1.yaml`](docs/parameter_matrix.thd_wav_list_v1.yaml)
- [`docs/parameter_matrix.thd_atmos_wav_v1.yaml`](docs/parameter_matrix.thd_atmos_wav_v1.yaml)
- [`docs/parameter_matrix.thd_atmos_wav_list_v1.yaml`](docs/parameter_matrix.thd_atmos_wav_list_v1.yaml)
- [`docs/ac4-official-notes.en.md`](docs/ac4-official-notes.en.md)

Developer references:
- [`docs/developer-guide.en.md`](docs/developer-guide.en.md)
- [`docs/coverage-and-experiments.en.md`](docs/coverage-and-experiments.en.md)
- [`docs/json-output.en.md`](docs/json-output.en.md)

Chinese docs:
- [`README.md`](README.md)
- [`docs/developer-guide.zh.md`](docs/developer-guide.zh.md)
- [`docs/coverage-and-experiments.zh.md`](docs/coverage-and-experiments.zh.md)
- [`docs/json-output.zh.md`](docs/json-output.zh.md)

## License

This repository follows its own license terms.

Runtime dependencies such as Dolby DEE, `dee-win`, upstream fixtures, and third-party tooling have their own licenses and usage restrictions.
