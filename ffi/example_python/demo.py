from __future__ import annotations

from dcg_ffi import DcgFfiClient

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
    client = DcgFfiClient()

    validated = client.validate(JOB_TEXT)
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

    generated = client.generate(JOB_TEXT, format="xml")
    print(
        "generated:",
        {
            "format": generated.format,
            "template_id": generated.template_id,
            "output_container": generated.output_container,
            "rendered_bytes": len(generated.rendered_config.encode("utf-8")),
        },
    )


if __name__ == "__main__":
    main()
