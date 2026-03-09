# dee-config-gen

Rust CLI for generating, validating, and optionally running DEE XML jobs.

Current MVP scope:
- Single template ID: `atmos_ec3_v1`
- Input: YAML (primary) + JSON (compatible)
- Commands: `generate`, `validate`, `run`
- Encode modes: `streaming` / `bluray` / `ddp71`
- Music fixed-value policy supported via `--allow-fixed-override`

## Architecture

- `config`: only input serde model and load/path normalization helpers.
- `resolve`: generic resolve pipeline (`defaults -> merge overrides -> constraint evaluation`).
- `schema`: shared `ParamRule`/`Constraint` and validation engine.
- `template`: template registry + template-specific modules (`atmos_ec3_v1`).
- `render`: generic `XmlNode` tree renderer.

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
- `encode_mode`: `streaming` or `bluray` or `ddp71`
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

`encode_mode=bluray` supports:

- `1152, 1280, 1408, 1512, 1536, 1664`

Hard maximum is `1664`.

Bluray defaults automatically inject:

- `encoding_backend=atmosprocessor`
- `encoder_mode=bluray`

## DDP 7.1 mode set

`encode_mode=ddp71` supports:

- `384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024, 1280, 1536, 1664`

`ddp71` defaults:

- `encoder_mode=ddp71`
- `surround_trim_7_1=auto`

`ddp71` constraints:

- `encoding_backend` is not allowed
- mode is mutually exclusive with `streaming` and `bluray` by `encode_mode` enum itself

## Parameter governance

Source tags used in the registry:

- `dolby_official`
- `deew_observed`
- `deezy_observed`

Artifacts:

- matrix snapshot: [`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- template schema: [`src/template/atmos_ec3_v1/params.rs`](src/template/atmos_ec3_v1/params.rs)
- xsd raw fixtures: [`tests/fixtures/xsd/raw/`](tests/fixtures/xsd/raw)
- xsd structured contract: [`tests/fixtures/xsd/contract.atmos_ec3_v1.json`](tests/fixtures/xsd/contract.atmos_ec3_v1.json)
- upstream observation report: [`docs/upstream_parameter_observations.md`](docs/upstream_parameter_observations.md)
- channel-based experiment (EN): [`docs/channel_based_mode_experiment.md`](docs/channel_based_mode_experiment.md)
- channel-based experiment (ZH): [`docs/channel_based_mode_experiment.zh.md`](docs/channel_based_mode_experiment.zh.md)

### XSD contract extraction

After exporting XSD template(s) from DEE into `tests/fixtures/xsd/raw/`, regenerate the structured contract:

```bash
python3 scripts/extract_xsd_contract.py \
  --template-id atmos_ec3_v1 \
  --dee-version unknown \
  --exported-at 2026-03-09T00:00:00Z \
  --raw-dir tests/fixtures/xsd/raw \
  --output tests/fixtures/xsd/contract.atmos_ec3_v1.json
```

## Runtime experiment helper

To reproduce channel-based runtime behavior (with `dee-win` as runtime base):

```bash
scripts/experiment_channel_based_profiles.sh --dee-win-root /path/to/dee-win
```

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

Run XSD snapshot consistency check (regenerates contract in temp file and compares with committed JSON):

```bash
cargo test --test xsd_contract_snapshot -- --ignored
```

Run schema + XSD driven matrix tests manually:

```bash
cargo test --test matrix_params -- --ignored --nocapture
```
