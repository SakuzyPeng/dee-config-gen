# Python FFI pilot (`ctypes`)

This folder is a minimal high-level-language pilot on top of the stable C ABI v1.

Scope:
- only `validate` and `generate`
- no `run` wrapper
- no extra protocol fields beyond current FFI v1

## Files

- `dcg_ffi.py`: thin `ctypes` wrapper for `dcg_validate_job` and `dcg_generate_config`
- `demo.py`: local smoke script

## Local run

Build the dynamic library first:

```bash
cargo build --release
```

Run the demo:

```bash
python3 ffi/example_python/demo.py
```

Optional: explicitly pass a library path:

```bash
DCG_FFI_LIB="$PWD/target/release/libdee_config_gen.dylib" \
  python3 ffi/example_python/demo.py
```

Linux use `libdee_config_gen.so`; Windows use `dee_config_gen.dll`.
