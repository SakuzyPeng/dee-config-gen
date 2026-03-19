# AC-4 官方整理笔记

本页整理本地 `DolbyEncodingEngineDocumentation_5.2.1` 中与 AC-4 immersive stereo 相关的官方证据，作为 `ac4_ims_atmos_v1` 和 `ac4_ims_pcm_v1` 的实现依据。

## 官方页面

- `encoding_params_c_ac4ims_workflow.html`
- `encoding_params_r_ac4ims_input.html`
- `encoding_params_r_ac4ims_filter.html`
- `encoding_params_r_ac4ims_output.html`
- `encoding_params_r_ac4ims_example_xml.html`
- `reference_r_input_atmos_mezz.html`
- `reference_r_input_wav.html`
- `reference_r_input_wav_list.html`
- `reference_r_output_ac4.html`
- `DolbyEncodingEngineInterfaceChanges.html`
- `DolbyEncodingEngineReleaseNotes.html`

## 当前实现结论

- 官方工作流名称是 `encode_to_ims_ac4`。
- 输入家族至少分成两条：
  - `atmos_mezz` 家族：Dolby Atmos master file set、ADM BWF、MXF IAB
  - `pcm` 家族：单个 WAV 或 WAV mono stem 列表
- 单个 WAV 输入在 AC-4 immersive stereo 页面里明确要求 5.1，通道顺序为 `L, R, C, LFE, Ls, Rs`。
- `wav_list` 的通用输入页允许 `mono / stereo / 5.1 / 7.1 / auto`，本仓库当前 AC-4 phase 只建模 5.1 stem 路径。
- 本地 DEE 5.2.1 runtime 进一步确认 AC-4 IMS PCM lane 只接受 6 声道输入：单个 8ch WAV 与 7.1 wav_list 都能通过输入装配，但会在 `encode_to_ims_ac4` 内部以 `ac4_ims_loudness_meter: Wav source must contain 6 channels.` 失败。
- 输出页明确支持 `ac4` 和 `mp4`；本仓库当前已开放 `output.container=ac4|mp4` 的 XML/JSON 渲染分支。
- 本地 DEE 5.2.1 runtime smoke 现已确认 AC-4 两条 lane 的原生 JSON 也可用，并且 `output.container=ac4|mp4` 都能被接受。
- 本地 DEE 5.2.1 runtime 已确认至少一个官方参数面比文档更窄：`iframe_interval=1` 会被拒绝，尽管官方页面写的是 `0-1000`。
- 本地 DEE 5.2.1 runtime smoke 已确认 `ac4_ims_atmos_v1` 和 `ac4_ims_pcm_v1` 两条 lane 都能成功写出 `output/mp4`，当前模板固定注入 `output_format=mp4`、`override_frame_rate=no`、`fill_video=false`。
- 2026-03-19 在原生 Windows（`win-pc`, `F:\\dee`）对 Atmos lane 复测，`output/ac4` 与 `output/mp4` 直出都成功，且从生成的 `.ac4` 手动 `mp4muxer` 也成功；此前在兼容层环境里出现的 `Access violation` 未复现，暂按运行环境不稳定记录，而非模板渲染错误。

## 官方 filter 参数

- loudness: `metering_mode`、`dialogue_intelligence`、`speech_threshold`
- range/time: `timecode_frame_rate`、`start`、`end`、`time_base`
- silence: `prepend_silence_duration`、`append_silence_duration`
- AC-4 specific: `data_rate`、`ac4_frame_rate`、`ims_legacy_presentation`、`iframe_interval`、`language`、`encoding_profile`
- DRC: `ddp_drc_profile`、`flat_panel_drc_profile`、`home_theatre_drc_profile`、`portable_hp_drc_profile`、`portable_spkr_drc_profile`

## 版本演进

- Interface changes / release notes 中出现过 `pcm_to_ims_ac4` 和 `atmos_mezz_to_ims_ac4` 的历史名称。
- 当前文档和模板语义统一收敛到 `encode_to_ims_ac4`。
- 因此仓库内部按输入家族拆成 `ac4_ims_atmos_v1` 与 `ac4_ims_pcm_v1`，但 XML filter 仍统一渲染为 `encode_to_ims_ac4`。
