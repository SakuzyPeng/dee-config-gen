from __future__ import annotations

import argparse
import importlib
import sys
from pathlib import Path
from types import ModuleType

JOB_TEXT = """\
template_id: atmos_ec3_v1
profile: standard
job_mode: single
encode_mode: streaming
input:
  storage_path: ./input
  file_names:
    - testADM.wav
output:
  storage_path: ./output
  file_names:
    - output.ec3
misc:
  temp_dir: ./tmp
"""


def load_bindings(bindings_dir: Path) -> ModuleType:
    if not bindings_dir.exists():
        raise SystemExit(f"bindings directory not found: {bindings_dir}")
    if not (bindings_dir / "dcg_uniffi.py").is_file():
        raise SystemExit(f"dcg_uniffi.py not found in: {bindings_dir}")

    sys.path.insert(0, str(bindings_dir))
    return importlib.import_module("dcg_uniffi")


def run_smoke(bindings_dir: Path) -> None:
    dcg_uniffi = load_bindings(bindings_dir)
    contract_version = dcg_uniffi.contract_version()
    assert contract_version == 1

    validated = dcg_uniffi.validate_job(JOB_TEXT, None)
    assert validated.template_id == "atmos_ec3_v1"
    assert validated.profile == "standard"
    assert validated.job_mode == "single"
    assert validated.encode_mode == "streaming"

    generated_xml = dcg_uniffi.generate_config(
        JOB_TEXT,
        dcg_uniffi.GenerateOptions(
            resolve=None,
            format=dcg_uniffi.RenderFormat.XML,
        ),
    )
    assert generated_xml.format.name == "XML"
    assert "<job_config>" in generated_xml.rendered_config

    generated_json = dcg_uniffi.generate_config(
        JOB_TEXT,
        dcg_uniffi.GenerateOptions(
            resolve=None,
            format=dcg_uniffi.RenderFormat.JSON,
        ),
    )
    assert generated_json.format.name == "JSON"
    assert '"job_config"' in generated_json.rendered_config

    try:
        dcg_uniffi.validate_job("template_id: [", None)
        raise AssertionError("parse error branch was expected")
    except Exception as exc:  # noqa: BLE001 - generated exception hierarchy is runtime-defined
        if not isinstance(exc, dcg_uniffi.BridgeError.ParseError):
            raise AssertionError(f"unexpected error type: {exc.__class__.__name__}") from exc

    print(
        "uniffi python smoke ok:",
        {
            "bindings_dir": str(bindings_dir),
            "contract_version": contract_version,
            "template_id": validated.template_id,
            "json_bytes": len(generated_json.rendered_config.encode("utf-8")),
        },
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Run UniFFI Python smoke tests.")
    parser.add_argument(
        "--bindings-dir",
        type=Path,
        default=Path(__file__).resolve().parent / "generated",
        help="directory containing dcg_uniffi.py and the dynamic library",
    )
    args = parser.parse_args()
    run_smoke(args.bindings_dir.resolve())


if __name__ == "__main__":
    main()
