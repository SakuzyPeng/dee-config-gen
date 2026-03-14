# Coverage & Experiments

[User README](../README.en.md) | [中文](coverage-and-experiments.zh.md) | [JSON Output](json-output.en.md) | [Developer Guide](developer-guide.en.md)

This page is for maintenance and research. It explains coverage semantics, manual runtime suites, XSD contracts, and the role of knowledge and pitfall fixtures.

## Coverage Semantics

Coverage matrix: [`coverage_matrix.full.yaml`](coverage_matrix.full.yaml)

Status values:
- `covered`: explicitly verified at that layer
- `conservative_gap`: intentionally more conservative than runtime, or only partially runtime-verified
- `unsupported_or_hidden`: outside the official contract, or confirmed unsupported at runtime

Rules:
- the coverage matrix is the single summary table for coverage state
- every new runtime conclusion must update the matrix
- matrix entries must be based on evidence, not informal assumptions

## Manual Runtime Suites

Atmos:

```bash
cargo test --test dee_runtime_hidden_params -- --ignored --nocapture
```

PCM:

```bash
cargo test --test dee_runtime_pcm_ddp -- --ignored --nocapture
```

TrueHD:

```bash
cargo test --test dee_runtime_thd -- --ignored --nocapture
```

TrueHD WAV:

```bash
cargo test --test dee_runtime_thd_wav -- --ignored --nocapture
```

TrueHD WAV list:

```bash
cargo test --test dee_runtime_thd_wav_list -- --ignored --nocapture
```

TrueHD mixed input (atmos_mezz + wav):

```bash
cargo test --test dee_runtime_thd_atmos_wav -- --ignored --nocapture
```

TrueHD mixed input:

```bash
cargo test --test dee_runtime_thd_atmos_wav_list -- --ignored --nocapture
```

These suites are used to:
- verify real DEE 5.2.1 behavior
- lock down hidden extensions, runtime normalization, and known gaps
- support updates to the coverage matrix and knowledge fixtures

## XSD Contracts and Fixtures

Official-contract files:
- AC-4 contract: [`../tests/fixtures/xsd/contract.ac4_v1.json`](../tests/fixtures/xsd/contract.ac4_v1.json)
- Atmos contract: [`../tests/fixtures/xsd/contract.atmos_ec3_v1.json`](../tests/fixtures/xsd/contract.atmos_ec3_v1.json)
- PCM contract: [`../tests/fixtures/xsd/contract.pcm_ddp_v1.json`](../tests/fixtures/xsd/contract.pcm_ddp_v1.json)
- TrueHD contract: [`../tests/fixtures/xsd/contract.thd_v1.json`](../tests/fixtures/xsd/contract.thd_v1.json)
- TrueHD WAV contract: [`../tests/fixtures/xsd/contract.thd_wav_v1.json`](../tests/fixtures/xsd/contract.thd_wav_v1.json)
- TrueHD WAV list contract: [`../tests/fixtures/xsd/contract.thd_wav_list_v1.json`](../tests/fixtures/xsd/contract.thd_wav_list_v1.json)
- TrueHD mixed-input WAV contract: [`../tests/fixtures/xsd/contract.thd_atmos_wav_v1.json`](../tests/fixtures/xsd/contract.thd_atmos_wav_v1.json)
- TrueHD mixed-input contract: [`../tests/fixtures/xsd/contract.thd_atmos_wav_list_v1.json`](../tests/fixtures/xsd/contract.thd_atmos_wav_list_v1.json)
- raw XSD files (local-only, do not commit): `../tests/fixtures/xsd/raw/`

Extraction script:
- [`../scripts/extract_xsd_contract.py`](../scripts/extract_xsd_contract.py)

Rule of thumb:
- XSD captures the official contract
- runtime captures actual external behavior
- exported official XSD files under `tests/fixtures/xsd/raw/` are local-only artifacts and should not be committed
- if they differ, the difference must be explicit in coverage and knowledge docs

## Knowledge / Pitfall / Experiment Index

Knowledge fixture:
- [`../tests/fixtures/upstream_knowledge.json`](../tests/fixtures/upstream_knowledge.json)

Pitfall fixtures:
- [`../tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json`](../tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json)
- [`../tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json`](../tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json)
- [`../tests/fixtures/upstream_pitfalls.thd_v1.json`](../tests/fixtures/upstream_pitfalls.thd_v1.json)

Parameter matrices:
- [`parameter_matrix.ac4_v1.yaml`](parameter_matrix.ac4_v1.yaml)
- [`parameter_matrix.atmos_ec3_v1.yaml`](parameter_matrix.atmos_ec3_v1.yaml)
- [`parameter_matrix.pcm_ddp_v1.yaml`](parameter_matrix.pcm_ddp_v1.yaml)
- [`parameter_matrix.thd_v1.yaml`](parameter_matrix.thd_v1.yaml)
- [`parameter_matrix.thd_wav_v1.yaml`](parameter_matrix.thd_wav_v1.yaml)
- [`parameter_matrix.thd_wav_list_v1.yaml`](parameter_matrix.thd_wav_list_v1.yaml)
- [`parameter_matrix.thd_atmos_wav_v1.yaml`](parameter_matrix.thd_atmos_wav_v1.yaml)
- [`parameter_matrix.thd_atmos_wav_list_v1.yaml`](parameter_matrix.thd_atmos_wav_list_v1.yaml)
- [`json-output.en.md`](json-output.en.md)

Experiment records:
- [`upstream_parameter_observations.md`](upstream_parameter_observations.md)
- [`channel_based_mode_experiment.zh.md`](channel_based_mode_experiment.zh.md)
- [`channel_based_mode_experiment.md`](channel_based_mode_experiment.md)

## Current Maintenance Policy

- High-value main-path parameters should trend toward `covered`
- Low-value tail parameters may remain `conservative_gap`, but only when the gap is already known and explained
- If runtime evidence proves current production modeling generates reliably failing XML, code should be tightened; otherwise record the difference first
