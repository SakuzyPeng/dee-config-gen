# Python UniFFI phase-1 (`validate` / `generate`)

This folder is the phase-1 high-level Python integration on top of the
new UniFFI bridge crate.

Scope:
- only `validate_job` and `generate_config`
- no `run` wrapper
- errors are surfaced as UniFFI exceptions with stable variant names

## Files

- `generate_bindings.sh`: build bridge cdylib + generate Python bindings
- `demo.py`: local smoke script (`validate`, `generate`, and error branch sample)
- `generated/`: generated Python bindings and copied dynamic library (local artifact)

## Local run

Generate bindings first:

```bash
cargo install --locked uniffi --version 0.31.0 --features cli
ffi/example_python_uniffi/generate_bindings.sh
```

Run demo:

```bash
python3 ffi/example_python_uniffi/demo.py
```
