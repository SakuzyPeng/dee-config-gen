use crate::config::{EncodeMode, Profile};

use super::filter::PcmDdpV1Filter;

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> PcmDdpV1Filter {
    let (dialogue_intelligence, speech_threshold, drc) = match profile {
        Profile::Standard => (true, 15, "film_light"),
        Profile::Music => (false, 100, "music_light"),
    };

    let (data_rate, encoder_mode_value, downmix_config) = match encode_mode {
        EncodeMode::Dd => (384, "dd", "5.1"),
        EncodeMode::Ddp => (768, "ddp", "5.1"),
        EncodeMode::Bluray => (1536, "bluray", "off"),
        EncodeMode::Ddp71 => (448, "ddp71", "off"),
        EncodeMode::Streaming => unreachable!("pcm_ddp_v1 does not support streaming"),
    };

    PcmDdpV1Filter {
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence,
        speech_threshold,
        data_rate,
        bitstream_mode: "complete_main".to_string(),
        downmix_config: downmix_config.to_string(),
        timecode_frame_rate: "not_indicated".to_string(),
        start: "first_frame_of_action".to_string(),
        end: "end_of_file".to_string(),
        time_base: "file_position".to_string(),
        prepend_silence_duration: "0.0".to_string(),
        append_silence_duration: "0.0".to_string(),
        lfe_on: true,
        dolby_surround_mode: "not_indicated".to_string(),
        dolby_surround_ex_mode: "no".to_string(),
        user_data: -1,
        line_mode_drc_profile: drc.to_string(),
        rf_mode_drc_profile: drc.to_string(),
        lfe_lowpass_filter: true,
        surround_90_degree_phase_shift: true,
        surround_3db_attenuation: true,
        loro_center_mix_level: "-3".to_string(),
        loro_surround_mix_level: "-3".to_string(),
        ltrt_center_mix_level: "-3".to_string(),
        ltrt_surround_mix_level: "-3".to_string(),
        preferred_downmix_mode: "loro".to_string(),
        allow_hybrid_downmix: false,
        starting_timecode: "off".to_string(),
        frame_rate: "auto".to_string(),
        custom_dialnorm: 0,
        encoder_mode: encoder_mode_value.to_string(),
    }
}
