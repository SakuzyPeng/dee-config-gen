use crate::config::{EncodeMode, Profile};

use super::filter::PcmDdpV1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> PcmDdpV1Filter {
    let (dialogue_intelligence, speech_threshold, drc) = match profile {
        Profile::Standard => (true, 15, "film_light"),
        Profile::Music => (false, 100, "music_light"),
    };

    PcmDdpV1Filter {
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence,
        speech_threshold,
        data_rate: match encode_mode {
            EncodeMode::Bluray => 1536,
            EncodeMode::Ddp71 => 448,
            EncodeMode::Streaming => unreachable!("pcm_ddp_v1 does not support streaming"),
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
        custom_dialnorm: 0,
        encoder_mode: match encode_mode {
            EncodeMode::Bluray => "bluray".to_string(),
            EncodeMode::Ddp71 => "ddp71".to_string(),
            EncodeMode::Streaming => unreachable!("pcm_ddp_v1 does not support streaming"),
        },
    }
}
