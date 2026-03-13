use std::{collections::BTreeMap, sync::OnceLock};

use crate::schema::{ModeAvailability, ParamRule, ParamSchema, SourceTag};

pub const BITRATE_HARD_MAX: u16 = 0;
pub const VALID_PROFILES: &[&str] = &["standard", "music"];
pub const VALID_ENCODE_MODES: &[&str] = &["mlp"];

const CHANNEL_CONFIGURATIONS: &[&str] = &["stereo", "5.1", "7.1", "auto"];
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
const EMBEDDED_TIMECODE_FRAME_RATES: &[&str] = &["auto", "23.976", "24", "25", "29.97", "30"];
const METERING_MODES: &[&str] = &["1770-1", "1770-2", "1770-3", "1770-4", "LeqA"];
const DRC_PROFILES: &[&str] = &[
    "film_standard",
    "film_light",
    "music_standard",
    "music_light",
    "speech",
];
const SPATIAL_CLUSTERS: &[&str] = &["12", "14", "16"];
const TIME_BASE_VALUES: &[&str] = &["file_position", "embedded_timecode"];
const OFFICIAL: &[SourceTag] = &[SourceTag::DolbyOfficial];

pub const PARAM_SCHEMAS: &[ParamSchema] = &[
    ParamSchema {
        key: "channel_configuration",
        rule: ParamRule::Enum(CHANNEL_CONFIGURATIONS),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "input_timecode_frame_rate",
        rule: ParamRule::Enum(TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "offset",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "ffoa",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "metering_mode",
        rule: ParamRule::Enum(METERING_MODES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "dialogue_intelligence",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "speech_threshold",
        rule: ParamRule::IntRange { min: 0, max: 100 },
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "timecode_frame_rate",
        rule: ParamRule::Enum(TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "starting_timecode",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "frame_rate",
        rule: ParamRule::Enum(EMBEDDED_TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "start",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "end",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "time_base",
        rule: ParamRule::Enum(TIME_BASE_VALUES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "prepend_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "append_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "custom_dialnorm",
        rule: ParamRule::IntRange { min: -31, max: 0 },
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "atmos_presentation_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "spatial_clusters",
        rule: ParamRule::Enum(SPATIAL_CLUSTERS),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "legacy_authoring_compatibility",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "presentation_8ch_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "presentation_6ch_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "presentation_2ch_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
    ParamSchema {
        key: "optimize_data_rate",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: OFFICIAL,
    },
];

pub fn find_schema(key: &str) -> Option<&'static ParamSchema> {
    PARAM_SCHEMAS.iter().find(|p| p.key == key)
}

pub fn bitrate_sets() -> &'static BTreeMap<&'static str, &'static [u16]> {
    static BITRATE_SETS: OnceLock<BTreeMap<&'static str, &'static [u16]>> = OnceLock::new();
    BITRATE_SETS.get_or_init(BTreeMap::new)
}
