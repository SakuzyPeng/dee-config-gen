# AC-4 Official Notes

This page summarizes the local `DolbyEncodingEngineDocumentation_5.2.1` evidence used to implement `ac4_ims_atmos_v1` and `ac4_ims_pcm_v1`.

## Official pages

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

## Current repository conclusions

- The official workflow name is `encode_to_ims_ac4`.
- The input family splits into at least two lanes:
  - `atmos_mezz`: Dolby Atmos master file set, ADM BWF, MXF IAB
  - `pcm`: a single WAV or a mono WAV stem list
- The AC-4 immersive stereo input page explicitly requires a single WAV input to be 5.1 in `L, R, C, LFE, Ls, Rs` order.
- The generic `wav_list` input page allows `mono / stereo / 5.1 / 7.1 / auto`; the current repository phase models only the 5.1 stem path for AC-4.
- The official output page supports both `ac4` and `mp4`; the repository now exposes XML rendering through `output.container=ac4|mp4`.
- JSON output is currently out of scope for AC-4.
- Local DEE 5.2.1 runtime narrows at least one documented control: `iframe_interval=1` is rejected even though the official page describes a `0-1000` range.
- Local DEE 5.2.1 smoke runs now verify `output/mp4` on both `ac4_ims_atmos_v1` and `ac4_ims_pcm_v1`. The current template branch injects `output_format=mp4`, `override_frame_rate=no`, and `fill_video=false`.

## Official filter surface

- loudness: `metering_mode`, `dialogue_intelligence`, `speech_threshold`
- range/time: `timecode_frame_rate`, `start`, `end`, `time_base`
- silence: `prepend_silence_duration`, `append_silence_duration`
- AC-4 specific: `data_rate`, `ac4_frame_rate`, `ims_legacy_presentation`, `iframe_interval`, `language`, `encoding_profile`
- DRC: `ddp_drc_profile`, `flat_panel_drc_profile`, `home_theatre_drc_profile`, `portable_hp_drc_profile`, `portable_spkr_drc_profile`

## Interface evolution

- The interface changes and release notes still mention the historical names `pcm_to_ims_ac4` and `atmos_mezz_to_ims_ac4`.
- The current documentation surface is unified around `encode_to_ims_ac4`.
- The repository therefore splits templates by input family, while still rendering the shared `encode_to_ims_ac4` XML filter.
