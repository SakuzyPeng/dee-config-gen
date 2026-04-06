# dee-config-gen UniFFI Python Bundle

This bundle is a self-contained Python consumer artifact for the stable
`dee-config-gen` UniFFI v1 contract.

## Included

- `dcg_uniffi.py`
- platform dynamic library:
  - `libuniffi.so` (Linux) or
  - `libuniffi.dylib` (macOS) or
  - `uniffi.dll` (Windows)
- `smoke.py` (minimal validate/generate/error-branch smoke)

## Quick smoke

Linux/macOS:

```bash
python3 smoke.py --bindings-dir .
```

Windows:

```bash
python smoke.py --bindings-dir .
```

The smoke script verifies `contract_version()`, `validate_job`,
`generate_config`, and a parse-error branch.

## Error handling contract

- branch by error type (`BridgeError.InvalidArgument`, `BridgeError.ParseError`, etc.)
- exception message is diagnostic only

## Compatibility

- `contract_version()` currently returns `1`
- the stable UniFFI v1 surface is `contract_version`, `validate_job`, and `generate_config`
- this bundle is Python-specific, but the UniFFI contract is maintained as a language-neutral baseline
