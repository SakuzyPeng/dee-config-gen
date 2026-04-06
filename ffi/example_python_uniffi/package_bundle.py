from __future__ import annotations

import argparse
from pathlib import Path
import zipfile


def find_library(bindings_dir: Path) -> Path:
    for name in ("libuniffi.so", "libuniffi.dylib", "uniffi.dll"):
        candidate = bindings_dir / name
        if candidate.is_file():
            return candidate
    raise FileNotFoundError(
        f"no UniFFI runtime library found in {bindings_dir}; expected one of "
        "libuniffi.so/libuniffi.dylib/uniffi.dll"
    )


def ensure_exists(path: Path, description: str) -> None:
    if not path.is_file():
        raise FileNotFoundError(f"{description} not found: {path}")


def main() -> None:
    parser = argparse.ArgumentParser(description="Package UniFFI Python release bundle.")
    parser.add_argument("--bindings-dir", type=Path, required=True)
    parser.add_argument("--output-zip", type=Path, required=True)
    parser.add_argument("--readme", type=Path, required=True)
    parser.add_argument("--smoke-script", type=Path, required=True)
    args = parser.parse_args()

    bindings_dir = args.bindings_dir.resolve()
    output_zip = args.output_zip.resolve()
    readme = args.readme.resolve()
    smoke_script = args.smoke_script.resolve()

    module_path = bindings_dir / "dcg_uniffi.py"
    library_path = find_library(bindings_dir)
    ensure_exists(module_path, "generated module")
    ensure_exists(readme, "bundle README")
    ensure_exists(smoke_script, "smoke script")

    output_zip.parent.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(output_zip, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.write(module_path, "dcg_uniffi.py")
        zf.write(library_path, library_path.name)
        zf.write(readme, "README.md")
        zf.write(smoke_script, "smoke.py")

    print(f"wrote bundle: {output_zip}")


if __name__ == "__main__":
    main()
