use anyhow::{Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTag {
    DolbyOfficial,
    DeewObserved,
    DeezyObserved,
}

#[derive(Debug, Clone, Copy)]
pub enum Rule {
    Enum(&'static [&'static str]),
    IntRange { min: i64, max: i64 },
    DataRate,
    Bool,
    String,
}

#[derive(Debug, Clone, Copy)]
pub struct ParamDef {
    pub key: &'static str,
    pub rule: Rule,
    pub sources: &'static [SourceTag],
    pub notes: &'static str,
}

pub const STREAMING_BITRATES: &[u16] = &[384, 448, 576, 640, 768, 1024];
pub const BLURAY_BITRATES: &[u16] = &[768, 1024, 1152, 1280, 1408, 1512, 1536, 1664];
pub const BLURAY_HARD_MAX: u16 = 1664;

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

const METERING_MODES: &[&str] = &["1770-1", "1770-2", "1770-3", "1770-4", "LeqA"];
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
const TRIM_SURROUND_51: &[&str] = &["0", "-3", "-6", "-9", "auto"];
const TRIM_HEIGHT_51: &[&str] = &["-3", "-6", "-9", "-12", "auto"];
const TIME_BASE_VALUES: &[&str] = &["file_position", "embedded_timecode"];

const DEF_ATMOS_MODE: &[SourceTag] = &[SourceTag::DolbyOfficial, SourceTag::DeezyObserved];
const DEF_DRC: &[SourceTag] = &[
    SourceTag::DolbyOfficial,
    SourceTag::DeewObserved,
    SourceTag::DeezyObserved,
];
const DEF_BLURAY: &[SourceTag] = &[SourceTag::DeewObserved, SourceTag::DeezyObserved];

pub const PARAMS: &[ParamDef] = &[
    ParamDef {
        key: "metering_mode",
        rule: Rule::Enum(METERING_MODES),
        sources: DEF_ATMOS_MODE,
        notes: "Loudness measuring mode.",
    },
    ParamDef {
        key: "dialogue_intelligence",
        rule: Rule::Bool,
        sources: DEF_ATMOS_MODE,
        notes: "Ignored for 1770-1/LeqA.",
    },
    ParamDef {
        key: "speech_threshold",
        rule: Rule::IntRange { min: 0, max: 100 },
        sources: DEF_ATMOS_MODE,
        notes: "Speech gating threshold percentage.",
    },
    ParamDef {
        key: "data_rate",
        rule: Rule::DataRate,
        sources: &[
            SourceTag::DolbyOfficial,
            SourceTag::DeewObserved,
            SourceTag::DeezyObserved,
        ],
        notes: "Streaming and bluray bitrate sets differ.",
    },
    ParamDef {
        key: "timecode_frame_rate",
        rule: Rule::Enum(TIMECODE_FRAME_RATES),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Frame rate associated with timecodes.",
    },
    ParamDef {
        key: "start",
        rule: Rule::String,
        sources: &[SourceTag::DolbyOfficial],
        notes: "Start timecode or first_frame_of_action.",
    },
    ParamDef {
        key: "end",
        rule: Rule::String,
        sources: &[SourceTag::DolbyOfficial],
        notes: "End timecode or end_of_file.",
    },
    ParamDef {
        key: "time_base",
        rule: Rule::Enum(TIME_BASE_VALUES),
        sources: &[SourceTag::DolbyOfficial],
        notes: "How start/end are interpreted.",
    },
    ParamDef {
        key: "prepend_silence_duration",
        rule: Rule::String,
        sources: &[SourceTag::DolbyOfficial],
        notes: "Seconds or frame count syntax.",
    },
    ParamDef {
        key: "append_silence_duration",
        rule: Rule::String,
        sources: &[SourceTag::DolbyOfficial],
        notes: "Seconds or frame count syntax.",
    },
    ParamDef {
        key: "line_mode_drc_profile",
        rule: Rule::Enum(DRC_PROFILES),
        sources: DEF_DRC,
        notes: "Dynamic range control line mode.",
    },
    ParamDef {
        key: "rf_mode_drc_profile",
        rule: Rule::Enum(DRC_PROFILES),
        sources: DEF_DRC,
        notes: "Dynamic range control RF mode.",
    },
    ParamDef {
        key: "loro_center_mix_level",
        rule: Rule::Enum(DOWNMIX_CENTER_LEVELS),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Lo/Ro center downmix level.",
    },
    ParamDef {
        key: "loro_surround_mix_level",
        rule: Rule::Enum(DOWNMIX_SURROUND_LEVELS),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Lo/Ro surround downmix level.",
    },
    ParamDef {
        key: "ltrt_center_mix_level",
        rule: Rule::Enum(DOWNMIX_CENTER_LEVELS),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Lt/Rt center downmix level.",
    },
    ParamDef {
        key: "ltrt_surround_mix_level",
        rule: Rule::Enum(DOWNMIX_SURROUND_LEVELS),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Lt/Rt surround downmix level.",
    },
    ParamDef {
        key: "preferred_downmix_mode",
        rule: Rule::Enum(PREFERRED_DOWNMIX_MODES),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Stereo downmix preference.",
    },
    ParamDef {
        key: "surround_trim_5_1",
        rule: Rule::Enum(TRIM_SURROUND_51),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Custom surround trim for 5.1.",
    },
    ParamDef {
        key: "height_trim_5_1",
        rule: Rule::Enum(TRIM_HEIGHT_51),
        sources: &[SourceTag::DolbyOfficial],
        notes: "Custom height trim for 5.1.",
    },
    ParamDef {
        key: "custom_dialnorm",
        rule: Rule::IntRange { min: -31, max: 0 },
        sources: &[
            SourceTag::DolbyOfficial,
            SourceTag::DeewObserved,
            SourceTag::DeezyObserved,
        ],
        notes: "0 keeps measured dialnorm.",
    },
    ParamDef {
        key: "encoding_backend",
        rule: Rule::Enum(&["atmosprocessor"]),
        sources: DEF_BLURAY,
        notes: "Observed community bluray extension parameter.",
    },
    ParamDef {
        key: "encoder_mode",
        rule: Rule::Enum(&["bluray"]),
        sources: DEF_BLURAY,
        notes: "Observed community bluray extension parameter.",
    },
];

pub fn find_param(key: &str) -> Option<&'static ParamDef> {
    PARAMS.iter().find(|p| p.key == key)
}

pub fn canonicalize_enum(key: &str, value: &str) -> Result<String> {
    let def = find_param(key).ok_or_else(|| anyhow::anyhow!("unknown parameter: {key}"))?;
    match def.rule {
        Rule::Enum(allowed) => {
            let lower = value.to_ascii_lowercase();
            allowed
                .iter()
                .find(|candidate| candidate.to_ascii_lowercase() == lower)
                .map(|found| (*found).to_string())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "invalid value '{value}' for {key}; allowed: {}",
                        allowed.join(", ")
                    )
                })
        }
        _ => bail!("parameter {key} is not enum"),
    }
}

pub fn validate_int(key: &str, value: i64) -> Result<()> {
    let def = find_param(key).ok_or_else(|| anyhow::anyhow!("unknown parameter: {key}"))?;
    match def.rule {
        Rule::IntRange { min, max } => {
            if (min..=max).contains(&value) {
                Ok(())
            } else {
                bail!("invalid value '{value}' for {key}; expected {min}..{max}")
            }
        }
        _ => bail!("parameter {key} is not integer range"),
    }
}

pub fn validate_data_rate(value: u16, atmos_mode: &str) -> Result<()> {
    if value > BLURAY_HARD_MAX {
        bail!("invalid data_rate '{value}'; hard max is {BLURAY_HARD_MAX}");
    }

    let allowed = match atmos_mode {
        "streaming" => STREAMING_BITRATES,
        "bluray" => BLURAY_BITRATES,
        _ => bail!("unsupported atmos_mode '{atmos_mode}'"),
    };

    if allowed.contains(&value) {
        Ok(())
    } else {
        bail!(
            "invalid data_rate '{value}' for mode '{atmos_mode}'; allowed: {}",
            allowed
                .iter()
                .map(u16::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{canonicalize_enum, validate_data_rate};

    #[test]
    fn validates_bluray_data_rate_upper_bound() {
        assert!(validate_data_rate(1664, "bluray").is_ok());
        assert!(validate_data_rate(1665, "bluray").is_err());
    }

    #[test]
    fn canonicalizes_case_insensitive_enums() {
        let got = canonicalize_enum("preferred_downmix_mode", "LoRo").unwrap();
        assert_eq!(got, "loro");
    }
}
