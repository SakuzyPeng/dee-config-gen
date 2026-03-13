# DEE JSON Output
[User README](../README.en.md) | [中文](json-output.zh.md) | [Developer Guide](developer-guide.en.md)

## Overview
`dee-config-gen` can now generate Dolby DEE JSON configuration files directly.

Current behavior:
- XML remains the default output format
- Use `--format json` to switch to JSON
- `validate`, `generate`, and `run` all support JSON
- `run --format json` automatically injects `--json` for the DEE runner

## Supported Templates
- `atmos_ec3_v1`
- `pcm_ddp_v1`
- `thd_v1`
- `thd_wav_v1`
- `thd_wav_list_v1`
- `thd_atmos_wav_v1`
- `thd_atmos_wav_list_v1`

If a template does not support JSON output, the tool fails explicitly instead of pretending partial support.

## Quick Examples
Generate JSON:

```bash
cargo run -- generate -i examples/atmos_ec3_single.streaming.yaml --format json -o job.json
```

Validate the JSON render path:

```bash
cargo run -- validate -i examples/pcm_ddp_single.dd.yaml --format json
```

Run DEE with JSON:

```bash
cargo run -- run -i examples/thd_single.mlp.yaml --format json --runner-cmd "dee"
```

## Output Semantics
- Validation, defaults, and template constraints still come from the existing `resolve` flow
- JSON is only another DEE config serialization format, not a second parameter system
- XML remains the default output format; JSON is a parallel capability

## Relationship to XML
- Structurally, JSON maps to the same `job_config` as XML
- Final acceptance is defined by local `DEE 5.2.1` runtime behavior
- If JSON runtime later diverges from XML for a template, that difference will be recorded in:
  - `tests/fixtures/upstream_knowledge.json`
  - and, if needed, the matching pitfall fixture

## Related Docs
- Coverage matrix: [`coverage_matrix.full.yaml`](coverage_matrix.full.yaml)
- Coverage & experiments: [`coverage-and-experiments.en.md`](coverage-and-experiments.en.md)
- Developer guide: [`developer-guide.en.md`](developer-guide.en.md)
