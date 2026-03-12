use crate::config::{EncodeMode, Profile};

use super::filter::AtmosEc3V1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> AtmosEc3V1Filter {
    let (dialogue_intelligence, speech_threshold, drc) = match profile {
        Profile::Standard => (true, 15, "film_light"),
        Profile::Music => (false, 100, "music_light"),
    };

    AtmosEc3V1Filter {
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence,
        speech_threshold,
        data_rate: match encode_mode {
            EncodeMode::Streaming => 448,
            EncodeMode::Dd | EncodeMode::Ddp => {
                unreachable!("atmos_ec3_v1 does not support dd/ddp")
            }
            EncodeMode::Mlp => unreachable!("atmos_ec3_v1 does not support mlp"),
            EncodeMode::Bluray => 1280,
            EncodeMode::Ddp71 => unreachable!("atmos_ec3_v1 does not support ddp71"),
        },
        timecode_frame_rate: "not_indicated".to_string(),
        start: "first_frame_of_action".to_string(),
        end: "end_of_file".to_string(),
        time_base: "file_position".to_string(),
        prepend_silence_duration: "0.0".to_string(),
        append_silence_duration: "0.0".to_string(),
        line_mode_drc_profile: drc.to_string(),
        rf_mode_drc_profile: drc.to_string(),
        loro_center_mix_level: "-3".to_string(),
        loro_surround_mix_level: "-3".to_string(),
        ltrt_center_mix_level: "-3".to_string(),
        ltrt_surround_mix_level: "-3".to_string(),
        preferred_downmix_mode: "loro".to_string(),
        surround_trim_5_1: "auto".to_string(),
        height_trim_5_1: "auto".to_string(),
        custom_dialnorm: 0,
        encoding_backend: match encode_mode {
            EncodeMode::Streaming => None,
            EncodeMode::Dd | EncodeMode::Ddp => {
                unreachable!("atmos_ec3_v1 does not support dd/ddp")
            }
            EncodeMode::Mlp => unreachable!("atmos_ec3_v1 does not support mlp"),
            EncodeMode::Bluray => Some("atmosprocessor".to_string()),
            EncodeMode::Ddp71 => unreachable!("atmos_ec3_v1 does not support ddp71"),
        },
        encoder_mode: match encode_mode {
            EncodeMode::Streaming => None,
            EncodeMode::Dd | EncodeMode::Ddp => {
                unreachable!("atmos_ec3_v1 does not support dd/ddp")
            }
            EncodeMode::Mlp => unreachable!("atmos_ec3_v1 does not support mlp"),
            EncodeMode::Bluray => Some("bluray".to_string()),
            EncodeMode::Ddp71 => unreachable!("atmos_ec3_v1 does not support ddp71"),
        },
    }
}
