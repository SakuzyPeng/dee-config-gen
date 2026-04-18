# 模板选择指南

[English](template-guide.en.md)

## `ac4_ims_atmos_v1`

适合：
- 使用官方 `encode_to_ims_ac4` 路径，从 `inputs.atmos_mezz` 生成 `.ac4` 或 `.mp4`
- 输入家族固定为 Atmos mezzanine / ADM / IAB 一类沉浸式源
- 当前同时支持 XML/JSON；`output.container=ac4|mp4` 两条分支都已做本地 DEE runtime smoke

样例：
- [`examples/ac4_ims_atmos_single.ac4.yaml`](../examples/ac4_ims_atmos_single.ac4.yaml)
- [`examples/ac4_ims_atmos_single.mp4.yaml`](../examples/ac4_ims_atmos_single.mp4.yaml)

## `ac4_ims_pcm_v1`

适合：
- 使用官方 `encode_to_ims_ac4` 路径，从 `inputs.wav` 或 `inputs.wav_list` 生成 `.ac4` 或 `.mp4`
- `inputs.wav` 与 `inputs.wav_list` 二选一，不能混用
- 当前同时支持 XML/JSON；`output.container=ac4|mp4` 两条分支都已做本地 DEE runtime smoke

样例：
- [`examples/ac4_ims_pcm_single.ac4.yaml`](../examples/ac4_ims_pcm_single.ac4.yaml)
- [`examples/ac4_ims_pcm_single.mp4.yaml`](../examples/ac4_ims_pcm_single.mp4.yaml)

## `atmos_ec3_v1`

适合：
- 生成 Atmos DDP XML
- 使用 `streaming` 或 `bluray` 两种 Atmos 路径
- 当前支持 DEE JSON 输出

样例：
- [`examples/atmos_ec3_single.streaming.yaml`](../examples/atmos_ec3_single.streaming.yaml)
- [`examples/atmos_ec3_single.bluray.yaml`](../examples/atmos_ec3_single.bluray.yaml)

## `pcm_ddp_v1`

适合：
- 使用 `pcm_to_ddp` 路径生成 DD / DDP XML
- 管理 `dd`、`ddp`、`ddp71`、`bluray` 四种 PCM 编码模式
- 当前也支持 DEE JSON 输出

样例：
- [`examples/pcm_ddp_single.dd.yaml`](../examples/pcm_ddp_single.dd.yaml)
- [`examples/pcm_ddp_single.ddp.yaml`](../examples/pcm_ddp_single.ddp.yaml)
- [`examples/pcm_ddp_single.ddp71.yaml`](../examples/pcm_ddp_single.ddp71.yaml)
- [`examples/pcm_ddp_single.bluray.yaml`](../examples/pcm_ddp_single.bluray.yaml)

## `thd_v1`

适合：
- 使用官方 `encode_to_dthd` 路径生成 TrueHD MLP XML/JSON
- 输入是 `atmos_mezz`，输出是 `mlp`

样例：
- [`examples/thd_single.mlp.yaml`](../examples/thd_single.mlp.yaml)

## `thd_wav_v1`

适合：
- 使用单文件 `wav -> mlp` 的 TrueHD 路线
- 输入是一个 WAV 文件，不是 stem 列表

样例：
- [`examples/thd_wav_single.mlp.yaml`](../examples/thd_wav_single.mlp.yaml)

## `thd_wav_list_v1`

适合：
- 使用 ordered mono stems 的 `wav_list -> mlp` TrueHD 路线
- `input.file_names` 采用固定顺序槽位
- 当前正式支持 `stereo`、`5.1`、`7.1`，不支持 `mono` 和 `-` 占位

样例：
- [`examples/thd_wav_list_single.mlp.yaml`](../examples/thd_wav_list_single.mlp.yaml)

## `thd_atmos_wav_v1`

适合：
- 使用 mixed-input 的 `atmos_mezz + wav -> mlp` TrueHD 路线
- 输入通过 `inputs.atmos_mezz` 和 `inputs.wav` 两组分别声明

样例：
- [`examples/thd_atmos_wav_single.mlp.yaml`](../examples/thd_atmos_wav_single.mlp.yaml)

## `thd_atmos_wav_list_v1`

适合：
- 使用 mixed-input 的 `atmos_mezz + wav_list -> mlp` TrueHD 路线
- 输入通过 `inputs.atmos_mezz` 和 `inputs.wav_list` 两组分别声明

样例：
- [`examples/thd_atmos_wav_list_single.mlp.yaml`](../examples/thd_atmos_wav_list_single.mlp.yaml)
