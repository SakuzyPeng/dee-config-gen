# dee-config-gen

Rust CLI for generating, validating, and optionally running DEE XML jobs.

Current MVP scope:
- Template IDs: `atmos_ec3_v1`, `pcm_ddp_v1`
- Input: YAML (primary) + JSON (compatible)
- Commands: `generate`, `validate`, `run`
- `atmos_ec3_v1` encode modes: `streaming` / `bluray`
- `pcm_ddp_v1` encode modes: `dd` / `ddp` / `ddp71` / `bluray`
- Music fixed-value policy supported via `--allow-fixed-override`

## Architecture

- `config`: only input serde model and load/path normalization helpers.
- `resolve`: generic resolve pipeline (`defaults -> merge overrides -> constraint evaluation`).
- `schema`: shared `ParamRule`/`Constraint` and validation engine.
- `template`: template registry + template-specific modules (`atmos_ec3_v1`, `pcm_ddp_v1`).
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
  - or `pcm_ddp_v1`
- `profile`: `standard` or `music`
- `job_mode`: `single` or `album`
- `encode_mode`:
  - `atmos_ec3_v1`: `streaming` or `bluray`
  - `pcm_ddp_v1`: `dd`, `ddp`, `bluray`, or `ddp71`
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

`template_id=atmos_ec3_v1, encode_mode=bluray` supports:

- `1152, 1280, 1408, 1512, 1536, 1664`

Hard maximum is `1664`.

Bluray defaults automatically inject:

- `encoding_backend=atmosprocessor`
- `encoder_mode=bluray`

## PCM DDP mode set

`template_id=pcm_ddp_v1, encode_mode=dd` supports:

- `224, 256, 320, 384, 448, 512, 576, 640`

`template_id=pcm_ddp_v1, encode_mode=ddp` supports:

- `192, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384, 400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024`

`template_id=pcm_ddp_v1, encode_mode=ddp71` supports:

- `384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024`

`template_id=pcm_ddp_v1, encode_mode=bluray` supports:

- `768, 1024, 1280, 1536, 1664`

`pcm_ddp_v1` defaults:

- `metering_mode=1770-3`
- `encode_mode=dd` defaults `data_rate=640`
- `encode_mode=dd` injects `encoder_mode=dd`
- `encode_mode=dd` defaults `downmix_config=5.1`
- `encode_mode=ddp` defaults `data_rate=1024`
- `encode_mode=ddp` injects `encoder_mode=ddp`
- `encode_mode=ddp` defaults `downmix_config=5.1`
- `encode_mode=ddp71` defaults `data_rate=1024`
- `encode_mode=ddp71` injects `encoder_mode=ddp71`
- `encode_mode=ddp71` injects `downmix_config=off`
- `encode_mode=bluray` defaults `data_rate=1664`
- `encode_mode=bluray` injects `encoder_mode=bluray`
- `encode_mode=bluray` injects `downmix_config=off`
- `1770-4` is intentionally not allowed on `pcm_ddp_v1`; local DEE 5.2.1 runtime rejects it on the `pcm_to_ddp` path

`pcm_ddp_v1` now exposes these previously fixed-only PCM parameters:

- `bitstream_mode`
- `downmix_config`
- `lfe_on`
- `dolby_surround_mode`
- `dolby_surround_ex_mode`
- `user_data`
- `lfe_lowpass_filter`
- `surround_90_degree_phase_shift`
- `surround_3db_attenuation`
- `allow_hybrid_downmix`
- `starting_timecode`
- `frame_rate`

Runtime note:

- `starting_timecode` overrides are currently runtime-verified on `dd` and `bluray`
- `frame_rate` is intentionally modeled as free-form string on `pcm_ddp_v1`, because DEE 5.2.1 accepts arbitrary strings across all tested `pcm_to_ddp` modes while the official contract is narrower
- `dd` / `ddp` are input-sensitive:
  - `6ch` accepts `downmix_config=5.1` or `off`
  - `8ch` requires `downmix_config=5.1`
- `ddp71` / `bluray` require `downmix_config=off`
- `preferred_downmix_mode=ltrt-pl2` is runtime-verified for `ddp` / `ddp71`, but rejected for `dd` / `bluray`
- `dolby_surround_ex_mode` is valid on `pcm_to_ddp`; `bluray` normalizes `no` / `not_indicated` to `yes` at runtime
- `atmos_ec3_v1` keeps `preferred_downmix_mode=ltrt-pl2` for `streaming`, but rejects it for `bluray` to match DEE 5.2.1 runtime behavior
- `atmos_ec3_v1` does not expose `dolby_surround_mode` or `dolby_surround_ex_mode`; DEE reports them as unknown `downmix:*` properties on the Atmos path

`pcm_ddp_v1` does not support Atmos-only overrides such as:

- `encoding_backend`
- `surround_trim_5_1`
- `height_trim_5_1`

## Parameter governance

Source tags used in the registry:

- `dolby_official`
- `deew_observed`
- `deezy_observed`

Artifacts:

- matrix snapshot: [`docs/parameter_matrix.atmos_ec3_v1.yaml`](docs/parameter_matrix.atmos_ec3_v1.yaml)
- matrix snapshot: [`docs/parameter_matrix.pcm_ddp_v1.yaml`](docs/parameter_matrix.pcm_ddp_v1.yaml)
- template schema: [`src/template/atmos_ec3_v1/params.rs`](src/template/atmos_ec3_v1/params.rs)
- template schema: [`src/template/pcm_ddp_v1/params.rs`](src/template/pcm_ddp_v1/params.rs)
- xsd raw fixtures: [`tests/fixtures/xsd/raw/`](tests/fixtures/xsd/raw)
- xsd structured contract: [`tests/fixtures/xsd/contract.atmos_ec3_v1.json`](tests/fixtures/xsd/contract.atmos_ec3_v1.json)
- xsd structured contract: [`tests/fixtures/xsd/contract.pcm_ddp_v1.json`](tests/fixtures/xsd/contract.pcm_ddp_v1.json)
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

```bash
python3 scripts/extract_xsd_contract.py \
  --template-id pcm_ddp_v1 \
  --dee-version unknown \
  --exported-at 2026-03-09T00:00:00Z \
  --raw-dir tests/fixtures/xsd/raw \
  --filter-path-prefix /job_config/filter/audio/pcm_to_ddp \
  --output tests/fixtures/xsd/contract.pcm_ddp_v1.json
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

Run PCM template example tests:

```bash
cargo test --test pcm_ddp_examples
```

Run PCM template matrix tests:

```bash
cargo test --test pcm_ddp_matrix
```

Run PCM XSD smoke tests:

```bash
cargo test --test pcm_ddp_xsd_contract_smoke
```

Run local pre-commit checks:

```bash
scripts/precommit_checks.sh
```
