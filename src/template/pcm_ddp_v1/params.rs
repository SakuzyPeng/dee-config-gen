use std::{collections::BTreeMap, sync::OnceLock};

use crate::schema::{ModeAvailability, ParamRule, ParamSchema, SourceTag};

pub const DD_BITRATES: &[u16] = &[224, 256, 320, 384, 448, 512, 576, 640];
pub const DDP_BITRATES: &[u16] = &[
    192, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384, 400, 448,
    512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
];
pub const DDP71_BITRATES: &[u16] = &[384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024];
pub const BLURAY_BITRATES: &[u16] = &[768, 1024, 1280, 1536, 1664];
pub const BITRATE_HARD_MAX: u16 = 1664;

pub const VALID_PROFILES: &[&str] = &["standard", "music"];
pub const VALID_ENCODE_MODES: &[&str] = &["dd", "ddp", "bluray", "ddp71"];

const TIMECODE_FRAME_RATES: &[&str] = &[
    "not_indicated",
    "23.976",
    "24",
    "25",
    "29.97",
    "30",
    "48",
    "50",
    "59.94",
    "60",
];
const PCM_METERING_MODES: &[&str] = &["1770-1", "1770-2", "1770-3", "LeqA"];
const BITSTREAM_MODES: &[&str] = &[
    "complete_main",
    "music_and_effects",
    "visually_impaired",
    "hearing_impaired",
    "dialogue",
    "commentary",
    "emergency",
    "voice_over",
];
const DOWNMIX_CONFIGS: &[&str] = &["off", "mono", "stereo", "5.1"];
const DRC_PROFILES: &[&str] = &[
    "film_standard",
    "film_light",
    "music_standard",
    "music_light",
    "speech",
    "none",
];
const DOWNMIX_SURROUND_LEVELS: &[&str] = &["-1.5", "-3", "-4.5", "-6", "-inf"];
const DOWNMIX_CENTER_LEVELS: &[&str] = &["+3", "+1.5", "0", "-1.5", "-3", "-4.5", "-6", "-inf"];
const PREFERRED_DOWNMIX_MODES: &[&str] = &["not_indicated", "loro", "ltrt", "ltrt-pl2"];
const DOLBY_SURROUND_MODES: &[&str] = &["yes", "no", "not_indicated"];
const DOLBY_SURROUND_EX_MODES: &[&str] = &["yes", "no", "not_indicated"];
const TIME_BASE_VALUES: &[&str] = &["file_position", "embedded_timecode"];
const EMBEDDED_TIMECODE_FRAME_RATES: &[&str] = &[
    "auto", "23.976", "24", "25", "29.97", "30", "50", "59.94", "60",
];

const DEF_PCM: &[SourceTag] = &[SourceTag::DolbyOfficial, SourceTag::DeewObserved];
const DEF_DRC: &[SourceTag] = &[
    SourceTag::DolbyOfficial,
    SourceTag::DeewObserved,
    SourceTag::DeezyObserved,
];

pub const PARAM_SCHEMAS: &[ParamSchema] = &[
    ParamSchema {
        key: "metering_mode",
        rule: ParamRule::Enum(PCM_METERING_MODES),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "dialogue_intelligence",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "speech_threshold",
        rule: ParamRule::IntRange { min: 0, max: 100 },
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "data_rate",
        rule: ParamRule::DataRate,
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "bitstream_mode",
        rule: ParamRule::Enum(BITSTREAM_MODES),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "downmix_config",
        rule: ParamRule::Enum(DOWNMIX_CONFIGS),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "timecode_frame_rate",
        rule: ParamRule::Enum(TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "start",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "end",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "time_base",
        rule: ParamRule::Enum(TIME_BASE_VALUES),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "prepend_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "append_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "lfe_on",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "dolby_surround_mode",
        rule: ParamRule::Enum(DOLBY_SURROUND_MODES),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "dolby_surround_ex_mode",
        rule: ParamRule::Enum(DOLBY_SURROUND_EX_MODES),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "user_data",
        rule: ParamRule::IntRange {
            min: -1,
            max: 65535,
        },
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "line_mode_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_DRC,
    },
    ParamSchema {
        key: "rf_mode_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_DRC,
    },
    ParamSchema {
        key: "lfe_lowpass_filter",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "surround_90_degree_phase_shift",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DeewObserved, SourceTag::DeezyObserved],
    },
    ParamSchema {
        key: "surround_3db_attenuation",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DeewObserved, SourceTag::DeezyObserved],
    },
    ParamSchema {
        key: "loro_center_mix_level",
        rule: ParamRule::Enum(DOWNMIX_CENTER_LEVELS),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "loro_surround_mix_level",
        rule: ParamRule::Enum(DOWNMIX_SURROUND_LEVELS),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "ltrt_center_mix_level",
        rule: ParamRule::Enum(DOWNMIX_CENTER_LEVELS),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "ltrt_surround_mix_level",
        rule: ParamRule::Enum(DOWNMIX_SURROUND_LEVELS),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "preferred_downmix_mode",
        rule: ParamRule::Enum(PREFERRED_DOWNMIX_MODES),
        mode_availability: ModeAvailability::All,
        sources: &[SourceTag::DolbyOfficial],
    },
    ParamSchema {
        key: "allow_hybrid_downmix",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "starting_timecode",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::Only(&["dd", "bluray"]),
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "frame_rate",
        rule: ParamRule::Enum(EMBEDDED_TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "custom_dialnorm",
        rule: ParamRule::IntRange { min: -31, max: 0 },
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
    ParamSchema {
        key: "encoder_mode",
        rule: ParamRule::Enum(&["dd", "ddp", "bluray", "ddp71"]),
        mode_availability: ModeAvailability::All,
        sources: DEF_PCM,
    },
];

pub fn find_schema(key: &str) -> Option<&'static ParamSchema> {
    PARAM_SCHEMAS.iter().find(|p| p.key == key)
}

pub fn bitrate_sets() -> &'static BTreeMap<&'static str, &'static [u16]> {
    static BITRATE_SETS: OnceLock<BTreeMap<&'static str, &'static [u16]>> = OnceLock::new();
    BITRATE_SETS.get_or_init(|| {
        let mut m = BTreeMap::new();
        m.insert("dd", DD_BITRATES);
        m.insert("ddp", DDP_BITRATES);
        m.insert("bluray", BLURAY_BITRATES);
        m.insert("ddp71", DDP71_BITRATES);
        m
    })
}
