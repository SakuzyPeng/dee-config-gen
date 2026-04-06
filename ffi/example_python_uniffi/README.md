# UniFFI Python Bindings (`validate` / `generate`)

This folder is the current Python consumer package for the stable UniFFI v1
contract exposed by the new UniFFI bridge crate.

Scope:
- only `validate_job` and `generate_config`
- stable `contract_version()` query
- no `run` wrapper
- errors are surfaced as UniFFI exceptions with stable variant names

## Files

- `generate_bindings.sh`: build bridge cdylib + generate Python bindings
- `smoke.py`: CI/release friendly smoke (`contract_version`, `validate`, `generate xml/json`, parse error branch)
- `demo.py`: local entrypoint (delegates to `smoke.py`)
- `package_bundle.py`: package Python release bundle zip
- `RELEASE_BUNDLE_README.md`: README template embedded into release bundles
- `generated/`: generated Python bindings and copied dynamic library (local artifact, ignored)

Compatibility notes:
- the UniFFI v1 contract currently exposes `contract_version`, `validate_job`, and `generate_config`
- this repository currently publishes ready-to-run Python bundles for that contract
- future language bindings should reuse the same contract version semantics

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

Run smoke against an arbitrary unpacked bundle directory:

```bash
python3 ffi/example_python_uniffi/smoke.py --bindings-dir /path/to/unpacked-bundle
```
