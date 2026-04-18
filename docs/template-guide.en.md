# Template Selection Guide

[中文](template-guide.zh.md)

## `ac4_ims_atmos_v1`

Use it when you need:
- official `encode_to_ims_ac4` XML that takes `inputs.atmos_mezz` and emits `.ac4` or `.mp4`
- an Atmos mezzanine / ADM / IAB input family
- XML/JSON AC-4 immersive stereo output, with `output.container=ac4|mp4` runtime-smoked locally

Examples:
- [`examples/ac4_ims_atmos_single.ac4.yaml`](../examples/ac4_ims_atmos_single.ac4.yaml)
- [`examples/ac4_ims_atmos_single.mp4.yaml`](../examples/ac4_ims_atmos_single.mp4.yaml)

## `ac4_ims_pcm_v1`

Use it when you need:
- official `encode_to_ims_ac4` XML that takes `inputs.wav` or `inputs.wav_list` and emits `.ac4` or `.mp4`
- a PCM-family AC-4 immersive stereo workflow
- XML/JSON output, with `output.container=ac4|mp4` runtime-smoked locally

Examples:
- [`examples/ac4_ims_pcm_single.ac4.yaml`](../examples/ac4_ims_pcm_single.ac4.yaml)
- [`examples/ac4_ims_pcm_single.mp4.yaml`](../examples/ac4_ims_pcm_single.mp4.yaml)

## `atmos_ec3_v1`

Use it when you need:
- Atmos DDP XML
- `streaming` or `bluray` Atmos workflows
- DEE JSON output is currently supported

Examples:
- [`examples/atmos_ec3_single.streaming.yaml`](../examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](../examples/atmos_ec3_single.bluray.yaml)

## `pcm_ddp_v1`

Use it when you need:
- `pcm_to_ddp` XML for DD or DDP workflows
- one of `dd`, `ddp`, `ddp71`, or `bluray`
- DEE JSON output support for the same workflow

Examples:
- [`examples/pcm_ddp_single.dd.yaml`](../examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](../examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](../examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](../examples/pcm_ddp_single.bluray.yaml)

## `thd_v1`

Use it when you need:
- official `encode_to_dthd` XML/JSON for TrueHD MLP output
- `atmos_mezz` input and `mlp` output

Examples:
- [`examples/thd_single.mlp.yaml`](../examples/thd_single.mlp.yaml)

## `thd_wav_v1`

Use it when you need:
- single-file `wav -> mlp` TrueHD jobs
- one WAV input instead of a stem list

Examples:
- [`examples/thd_wav_single.mlp.yaml`](../examples/thd_wav_single.mlp.yaml)

## `thd_wav_list_v1`

Use it when you need:
- ordered mono stems on the `wav_list -> mlp` TrueHD path
- fixed slot order in `input.file_names`
- the currently supported runtime-aligned layouts: `stereo`, `5.1`, and `7.1`

Examples:
- [`examples/thd_wav_list_single.mlp.yaml`](../examples/thd_wav_list_single.mlp.yaml)

## `thd_atmos_wav_v1`

Use it when you need:
- the mixed-input `atmos_mezz + wav -> mlp` TrueHD path
- separate `inputs.atmos_mezz` and `inputs.wav` groups in one job

Examples:
- [`examples/thd_atmos_wav_single.mlp.yaml`](../examples/thd_atmos_wav_single.mlp.yaml)

## `thd_atmos_wav_list_v1`

Use it when you need:
- the mixed-input `atmos_mezz + wav_list -> mlp` TrueHD path
- separate `inputs.atmos_mezz` and `inputs.wav_list` groups in one job

Examples:
- [`examples/thd_atmos_wav_list_single.mlp.yaml`](../examples/thd_atmos_wav_list_single.mlp.yaml)
