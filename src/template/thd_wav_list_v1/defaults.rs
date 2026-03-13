use crate::spec::{EncodeMode, Profile};

use super::filter::ThdWavListV1Filter;

pub fn defaults(_profile: Profile, encode_mode: EncodeMode) -> ThdWavListV1Filter {
    match encode_mode {
        EncodeMode::Mlp => {}
        other => unreachable!("thd_wav_list_v1 does not support {}", other.as_str()),
    }

    ThdWavListV1Filter {
        channel_configuration: "auto".to_string(),
        input_timecode_frame_rate: "not_indicated".to_string(),
        offset: "auto".to_string(),
        ffoa: "auto".to_string(),
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence: true,
        speech_threshold: 15,
        timecode_frame_rate: "not_indicated".to_string(),
        starting_timecode: "off".to_string(),
        frame_rate: "auto".to_string(),
        start: "first_frame_of_action".to_string(),
        end: "end_of_file".to_string(),
        time_base: "file_position".to_string(),
        prepend_silence_duration: "0".to_string(),
        append_silence_duration: "0".to_string(),
        custom_dialnorm: 0,
        atmos_presentation_drc_profile: "film_light".to_string(),
        spatial_clusters: "12".to_string(),
        legacy_authoring_compatibility: true,
        presentation_8ch_drc_profile: "film_light".to_string(),
        presentation_6ch_drc_profile: "film_light".to_string(),
        presentation_2ch_drc_profile: "film_light".to_string(),
        optimize_data_rate: false,
    }
}
