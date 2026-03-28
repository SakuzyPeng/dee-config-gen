from __future__ import annotations

import sys
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent
GENERATED_DIR = BASE_DIR / "generated"

if not GENERATED_DIR.exists():
    raise SystemExit(
        "missing generated bindings. run ffi/example_python_uniffi/generate_bindings.sh first."
    )

sys.path.insert(0, str(GENERATED_DIR))

import dcg_uniffi  # type: ignore  # generated module

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


def main() -> None:
    validated = dcg_uniffi.validate_job(JOB_TEXT, None)
    print(
        "validated:",
        {
            "template_id": validated.template_id,
            "profile": validated.profile,
            "job_mode": validated.job_mode,
            "encode_mode": validated.encode_mode,
            "output_container": validated.output_container,
        },
    )

    generated = dcg_uniffi.generate_config(
        JOB_TEXT,
        dcg_uniffi.GenerateOptions(
            resolve=None,
            format=dcg_uniffi.RenderFormat.JSON,
        ),
    )
    print(
        "generated:",
        {
            "format": generated.format.name,
            "template_id": generated.template_id,
            "output_container": generated.output_container,
            "rendered_bytes": len(generated.rendered_config.encode("utf-8")),
        },
    )

    try:
        dcg_uniffi.validate_job("template_id: [", None)
    except Exception as exc:  # noqa: BLE001 - generated exception hierarchy is runtime-defined
        code = exc.__class__.__name__
        if isinstance(exc, dcg_uniffi.BridgeError.ParseError):
            print("error branch:", {"code": code, "message": str(exc)})
        else:
            print("error branch (unexpected):", {"code": code, "message": str(exc)})


if __name__ == "__main__":
    main()
