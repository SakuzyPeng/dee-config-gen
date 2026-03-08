# dee-config-gen

Rust CLI for generating, validating, and optionally running DEE XML jobs.

Current MVP scope:
- Single template ID: `atmos_ec3_v1`
- Input: YAML (primary) + JSON (compatible)
- Commands: `generate`, `validate`, `run`
- Atmos modes: `streaming` / `bluray`
- Music fixed-value policy supported via `--allow-fixed-override`

## Build

```bash
cargo build
```

## Commands

### Validate

```bash
cargo run -- validate -i examples/atmos_ec3_single.streaming.yaml
```

### Generate XML

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml -o ./job.xml
```

Write XML to stdout:

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml
```

### Generate + run external runner

`run` does not embed container logic. It invokes an external command:

- `--runner-cmd` has highest priority
- then env `DEE_RUNNER_CMD`
- fallback: `dee`

```bash
cargo run -- run \
  -i examples/atmos_ec3_single.streaming.yaml \
  --runner-cmd "IMAGE_TAG=ghcr.io/sakuzypeng/dee-box64-lab:latest /path/to/dee-win/scripts/run_dee_with_box64.sh" \
  --runner-arg --help \
  --keep-xml
```

## Input model

Required top-level fields:

- `template_id`: must be `atmos_ec3_v1`
- `profile`: `standard` or `music`
- `job_mode`: `single` or `album`
- `atmos_mode`: `streaming` or `bluray`
- `input`: `storage_path`, `file_names`
- `output`: `storage_path`, `file_names`
- `misc`: `temp_dir`, optional `clean_temp`
- optional `filter`
- optional `run` (`runner_args`, `env`)

### Path policy

Host paths are normalized to Windows-style paths in generated XML.
Default drive is `Y:` and can be changed by `--win-drive`.

## Fixed values (music profile)

By default, `profile=music` locks these fields:

- `dialogue_intelligence=false`
- `speech_threshold=100`
- `line_mode_drc_profile=music_light`
- `rf_mode_drc_profile=music_light`

To override them:

```bash
cargo run -- generate -i job.yaml --allow-fixed-override
```

## Bluray bitrate set

`atmos_mode=bluray` supports:

- `768, 1024, 1152, 1280, 1408, 1512, 1536, 1664`

Hard maximum is `1664`.

Bluray defaults automatically inject:

- `encoding_backend=atmosprocessor`
- `encoder_mode=bluray`

## Parameter governance

Source tags used in the registry:

- `dolby_official`
- `deew_observed`
- `deezy_observed`

Artifacts:

- matrix snapshot: [`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- code registry: [`src/registry.rs`](src/registry.rs)
- upstream observation report: [`docs/upstream_parameter_observations.md`](docs/upstream_parameter_observations.md)

## Upstream sync workflow (MIT repos)

The workflow clones upstream references into a local ignored directory and refreshes metadata-only observations.

```bash
scripts/sync_upstream_refs.sh
```

This creates/updates:

- `upstream/deew` (ignored)
- `upstream/DeeZy` (ignored)
- `docs/upstream_parameter_observations.md`

## Tests

```bash
cargo test
```
