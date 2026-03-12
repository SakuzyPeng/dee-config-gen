# dee-config-gen

[中文说明](README.md) | English README | [Developer Guide](docs/developer-guide.en.md) | [Coverage & Experiments](docs/coverage-and-experiments.en.md)

`dee-config-gen` is a Rust CLI for validating job specs, generating DEE XML, and optionally invoking an external runner.

Good fit when you want to:
- describe jobs in YAML or JSON and generate stable DEE XML
- validate parameters locally before calling `dee`
- manage Atmos, PCM DDP, and TrueHD templates in one tool

## Quick Overview

Current support:
- Templates: `atmos_ec3_v1`, `pcm_ddp_v1`, `thd_v1`
- Input: YAML, JSON
- Commands: `validate`, `generate`, `run`
- Atmos modes: `streaming`, `bluray`
- PCM modes: `dd`, `ddp`, `ddp71`, `bluray`
- TrueHD mode: `mlp`

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

Generate XML and call an external runner:

```bash
cargo run -- run \
  -i examples/atmos_ec3_single.streaming.yaml \
  --runner-cmd "dee" \
  --keep-xml
```

Notes:
- `run` does not embed container or Wine logic
- priority order is `--runner-cmd` -> `DEE_RUNNER_CMD` -> `dee`

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

### `atmos_ec3_v1`

Use it when you need:
- Atmos DDP XML
- `streaming` or `bluray` Atmos workflows

Examples:
- [`examples/atmos_ec3_single.streaming.yaml`](examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](examples/atmos_ec3_single.bluray.yaml)

### `pcm_ddp_v1`

Use it when you need:
- `pcm_to_ddp` XML for DD or DDP workflows
- one of `dd`, `ddp`, `ddp71`, or `bluray`

Examples:
- [`examples/pcm_ddp_single.dd.yaml`](examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](examples/pcm_ddp_single.bluray.yaml)

### `thd_v1`

Use it when you need:
- official `encode_to_dthd` XML for TrueHD MLP output
- `atmos_mezz` input and `mlp` output

Examples:
- [`examples/thd_single.mlp.yaml`](examples/thd_single.mlp.yaml)

## Common Notes

- `profile=music` locks a fixed set of values by default; use `--allow-fixed-override` only when you really need it
- generated XML normalizes paths to Windows-style paths; default drive is `Y:` and can be changed with `--win-drive`
- `atmos_ec3_v1`, `pcm_ddp_v1`, and `thd_v1` are separate templates with separate parameter sets

## Documentation

User-facing references:
- [`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- [`docs/parameter_matrix.pcm_ddp_v1.yaml`](docs/parameter_matrix.pcm_ddp_v1.yaml)
- [`docs/parameter_matrix.thd_v1.yaml`](docs/parameter_matrix.thd_v1.yaml)

Developer references:
- [`docs/developer-guide.en.md`](docs/developer-guide.en.md)
- [`docs/coverage-and-experiments.en.md`](docs/coverage-and-experiments.en.md)

Chinese docs:
- [`README.md`](README.md)
- [`docs/developer-guide.zh.md`](docs/developer-guide.zh.md)
- [`docs/coverage-and-experiments.zh.md`](docs/coverage-and-experiments.zh.md)

## License

This repository follows its own license terms.

Runtime dependencies such as Dolby DEE, `dee-win`, upstream fixtures, and third-party tooling have their own licenses and usage restrictions.
