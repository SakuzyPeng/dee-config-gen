use std::{collections::BTreeMap, sync::OnceLock};

use anyhow::{Result, bail};
use serde_json::{Map, Value as JsonValue, json};

use crate::{
    render::XmlNode,
    resolve::{ResolvedInputGroups, ResolvedIo, ResolvedJob, ResolvedOutput},
    schema::{
        ModeAvailability, ParamRule, ParamSchema, SourceTag, Value,
        validate::{ParamValue, ValidationContext, validate_mode_availability, validate_value},
    },
    spec::{EncodeMode, FilterOverrides, JobMode, OutputContainer, Profile},
    template::thd_json,
};

pub const AC4_BITRATES: &[u16] = &[64, 72, 112, 144, 256, 320];
pub const BITRATE_HARD_MAX: u16 = 320;

pub const VALID_PROFILES: &[&str] = &["standard", "music"];
pub const VALID_ENCODE_MODES: &[&str] = &["ac4"];

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
const LOUDNESS_METERING_MODES: &[&str] = &["1770-1", "1770-2", "1770-3", "1770-4", "LeqA"];
const TIME_BASE_VALUES: &[&str] = &["file_position", "embedded_timecode"];
const AC4_FRAME_RATES: &[&str] = &["native", "23.976", "24", "25", "29.97"];
const ENCODING_PROFILES: &[&str] = &["ims", "ims_music"];
const DRC_PROFILES: &[&str] = &[
    "film_standard",
    "film_light",
    "music_standard",
    "music_light",
    "speech",
    "none",
];
const DEF_OFFICIAL: &[SourceTag] = &[SourceTag::DolbyOfficial];

#[derive(Debug, Clone)]
pub struct Ac4ImsFilter {
    pub input_timecode_frame_rate: String,
    pub offset: String,
    pub ffoa: String,
    pub metering_mode: String,
    pub dialogue_intelligence: bool,
    pub speech_threshold: u8,
    pub data_rate: u16,
    pub timecode_frame_rate: String,
    pub start: String,
    pub end: String,
    pub time_base: String,
    pub prepend_silence_duration: String,
    pub append_silence_duration: String,
    pub ac4_frame_rate: String,
    pub ims_legacy_presentation: bool,
    pub iframe_interval: u16,
    pub language: String,
    pub encoding_profile: String,
    pub ddp_drc_profile: String,
    pub flat_panel_drc_profile: String,
    pub home_theatre_drc_profile: String,
    pub portable_hp_drc_profile: String,
    pub portable_spkr_drc_profile: String,
}

pub const PARAM_SCHEMAS: &[ParamSchema] = &[
    ParamSchema {
        key: "input_timecode_frame_rate",
        rule: ParamRule::Enum(TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "offset",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "ffoa",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "metering_mode",
        rule: ParamRule::Enum(LOUDNESS_METERING_MODES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "dialogue_intelligence",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "speech_threshold",
        rule: ParamRule::IntRange { min: 0, max: 100 },
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "data_rate",
        rule: ParamRule::DataRate,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "timecode_frame_rate",
        rule: ParamRule::Enum(TIMECODE_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "start",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "end",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "time_base",
        rule: ParamRule::Enum(TIME_BASE_VALUES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "prepend_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "append_silence_duration",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "ac4_frame_rate",
        rule: ParamRule::Enum(AC4_FRAME_RATES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "ims_legacy_presentation",
        rule: ParamRule::Bool,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "iframe_interval",
        rule: ParamRule::IntRange { min: 0, max: 1000 },
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "language",
        rule: ParamRule::FreeString,
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "encoding_profile",
        rule: ParamRule::Enum(ENCODING_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "ddp_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "flat_panel_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "home_theatre_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "portable_hp_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
    ParamSchema {
        key: "portable_spkr_drc_profile",
        rule: ParamRule::Enum(DRC_PROFILES),
        mode_availability: ModeAvailability::All,
        sources: DEF_OFFICIAL,
    },
];

pub fn find_schema(key: &str) -> Option<&'static ParamSchema> {
    PARAM_SCHEMAS.iter().find(|schema| schema.key == key)
}

pub fn bitrate_sets() -> &'static BTreeMap<&'static str, &'static [u16]> {
    static BITRATE_SETS: OnceLock<BTreeMap<&'static str, &'static [u16]>> = OnceLock::new();
    BITRATE_SETS.get_or_init(|| {
        let mut sets = BTreeMap::new();
        sets.insert("ac4", AC4_BITRATES);
        sets
    })
}

pub fn defaults(profile: Profile, encode_mode: EncodeMode) -> Ac4ImsFilter {
    match encode_mode {
        EncodeMode::Ac4 => Ac4ImsFilter {
            input_timecode_frame_rate: "not_indicated".to_string(),
            offset: "auto".to_string(),
            ffoa: "auto".to_string(),
            metering_mode: "1770-4".to_string(),
            dialogue_intelligence: true,
            speech_threshold: 15,
            data_rate: 256,
            timecode_frame_rate: "not_indicated".to_string(),
            start: "first_frame_of_action".to_string(),
            end: "end_of_file".to_string(),
            time_base: "file_position".to_string(),
            prepend_silence_duration: "0".to_string(),
            append_silence_duration: "0".to_string(),
            ac4_frame_rate: "native".to_string(),
            ims_legacy_presentation: false,
            iframe_interval: 0,
            language: String::new(),
            encoding_profile: match profile {
                Profile::Standard => "ims".to_string(),
                Profile::Music => "ims_music".to_string(),
            },
            ddp_drc_profile: "film_light".to_string(),
            flat_panel_drc_profile: "film_light".to_string(),
            home_theatre_drc_profile: "film_light".to_string(),
            portable_hp_drc_profile: "film_light".to_string(),
            portable_spkr_drc_profile: "film_light".to_string(),
        },
        other => unreachable!("AC-4 IMS templates do not support {}", other.as_str()),
    }
}

pub fn apply_overrides(
    filter: &mut Ac4ImsFilter,
    overrides: &FilterOverrides,
    encode_mode: EncodeMode,
) -> Result<()> {
    reject_unsupported_overrides(overrides)?;

    let ctx = ValidationContext {
        encode_mode: encode_mode.as_str(),
        bitrate_sets: bitrate_sets(),
        bitrate_hard_max: BITRATE_HARD_MAX,
    };

    if let Some(v) = &overrides.input_timecode_frame_rate {
        filter.input_timecode_frame_rate =
            validate_string_param("input_timecode_frame_rate", v, &ctx)?;
    }
    if let Some(v) = &overrides.offset {
        filter.offset = validate_input_timecode("offset", v)?;
    }
    if let Some(v) = &overrides.ffoa {
        filter.ffoa = validate_input_timecode("ffoa", v)?;
    }
    if let Some(v) = &overrides.metering_mode {
        filter.metering_mode = validate_string_param("metering_mode", v, &ctx)?;
    }
    if let Some(v) = overrides.dialogue_intelligence {
        filter.dialogue_intelligence = validate_bool_param("dialogue_intelligence", v, &ctx)?;
    }
    if let Some(v) = overrides.speech_threshold {
        let validated = validate_int_param("speech_threshold", i64::from(v), &ctx)?;
        filter.speech_threshold = u8::try_from(validated)
            .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for speech_threshold"))?;
    }
    if let Some(v) = overrides.data_rate {
        let validated = validate_int_param("data_rate", i64::from(v), &ctx)?;
        filter.data_rate = u16::try_from(validated)
            .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for data_rate"))?;
    }
    if let Some(v) = &overrides.timecode_frame_rate {
        filter.timecode_frame_rate = validate_string_param("timecode_frame_rate", v, &ctx)?;
    }
    if let Some(v) = &overrides.start {
        let value = validate_string_param("start", v, &ctx)?;
        filter.start = validate_boundary_timecode("start", &value)?;
    }
    if let Some(v) = &overrides.end {
        let value = validate_string_param("end", v, &ctx)?;
        filter.end = validate_boundary_timecode("end", &value)?;
    }
    if let Some(v) = &overrides.time_base {
        filter.time_base = validate_string_param("time_base", v, &ctx)?;
    }
    if let Some(v) = &overrides.prepend_silence_duration {
        let value = validate_string_param("prepend_silence_duration", v, &ctx)?;
        filter.prepend_silence_duration =
            validate_decimal_duration("prepend_silence_duration", &value)?;
    }
    if let Some(v) = &overrides.append_silence_duration {
        let value = validate_string_param("append_silence_duration", v, &ctx)?;
        filter.append_silence_duration =
            validate_decimal_duration("append_silence_duration", &value)?;
    }
    if let Some(v) = &overrides.ac4_frame_rate {
        filter.ac4_frame_rate = validate_string_param("ac4_frame_rate", v, &ctx)?;
    }
    if let Some(v) = overrides.ims_legacy_presentation {
        filter.ims_legacy_presentation = validate_bool_param("ims_legacy_presentation", v, &ctx)?;
    }
    if let Some(v) = overrides.iframe_interval {
        let validated = validate_int_param("iframe_interval", i64::from(v), &ctx)?;
        filter.iframe_interval = u16::try_from(validated)
            .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for iframe_interval"))?;
    }
    if let Some(v) = &overrides.language {
        let value = validate_string_param("language", v, &ctx)?;
        filter.language = validate_language_tag(&value)?;
    }
    if let Some(v) = &overrides.encoding_profile {
        filter.encoding_profile = validate_string_param("encoding_profile", v, &ctx)?;
    }
    if let Some(v) = &overrides.ddp_drc_profile {
        filter.ddp_drc_profile = validate_string_param("ddp_drc_profile", v, &ctx)?;
    }
    if let Some(v) = &overrides.flat_panel_drc_profile {
        filter.flat_panel_drc_profile = validate_string_param("flat_panel_drc_profile", v, &ctx)?;
    }
    if let Some(v) = &overrides.home_theatre_drc_profile {
        filter.home_theatre_drc_profile =
            validate_string_param("home_theatre_drc_profile", v, &ctx)?;
    }
    if let Some(v) = &overrides.portable_hp_drc_profile {
        filter.portable_hp_drc_profile = validate_string_param("portable_hp_drc_profile", v, &ctx)?;
    }
    if let Some(v) = &overrides.portable_spkr_drc_profile {
        filter.portable_spkr_drc_profile =
            validate_string_param("portable_spkr_drc_profile", v, &ctx)?;
    }

    Ok(())
}

pub fn constraint_value(filter: &Ac4ImsFilter, key: &str) -> Option<Value> {
    match key {
        "input_timecode_frame_rate" => Some(Value::Str(filter.input_timecode_frame_rate.clone())),
        "offset" => Some(Value::Str(filter.offset.clone())),
        "ffoa" => Some(Value::Str(filter.ffoa.clone())),
        "metering_mode" => Some(Value::Str(filter.metering_mode.clone())),
        "dialogue_intelligence" => Some(Value::Bool(filter.dialogue_intelligence)),
        "speech_threshold" => Some(Value::Int(i64::from(filter.speech_threshold))),
        "data_rate" => Some(Value::Int(i64::from(filter.data_rate))),
        "timecode_frame_rate" => Some(Value::Str(filter.timecode_frame_rate.clone())),
        "start" => Some(Value::Str(filter.start.clone())),
        "end" => Some(Value::Str(filter.end.clone())),
        "time_base" => Some(Value::Str(filter.time_base.clone())),
        "prepend_silence_duration" => Some(Value::Str(filter.prepend_silence_duration.clone())),
        "append_silence_duration" => Some(Value::Str(filter.append_silence_duration.clone())),
        "ac4_frame_rate" => Some(Value::Str(filter.ac4_frame_rate.clone())),
        "ims_legacy_presentation" => Some(Value::Bool(filter.ims_legacy_presentation)),
        "iframe_interval" => Some(Value::Int(i64::from(filter.iframe_interval))),
        "language" => Some(Value::Str(filter.language.clone())),
        "encoding_profile" => Some(Value::Str(filter.encoding_profile.clone())),
        "ddp_drc_profile" => Some(Value::Str(filter.ddp_drc_profile.clone())),
        "flat_panel_drc_profile" => Some(Value::Str(filter.flat_panel_drc_profile.clone())),
        "home_theatre_drc_profile" => Some(Value::Str(filter.home_theatre_drc_profile.clone())),
        "portable_hp_drc_profile" => Some(Value::Str(filter.portable_hp_drc_profile.clone())),
        "portable_spkr_drc_profile" => Some(Value::Str(filter.portable_spkr_drc_profile.clone())),
        _ => None,
    }
}

pub fn xml_structure(job: &ResolvedJob, filter: &Ac4ImsFilter, input_node: XmlNode) -> XmlNode {
    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    XmlNode::element(
        "job_config",
        vec![],
        vec![
            input_node,
            filter_node(filter),
            output_node(storage_tag, &job.output),
            misc_node(&job.misc.temp_dir, job.misc.clean_temp),
        ],
    )
}

pub fn json_structure(
    job: &ResolvedJob,
    filter: &Ac4ImsFilter,
    input_node: JsonValue,
) -> JsonValue {
    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    json!({
        "job_config": {
            "input": {
                "audio": input_node,
            },
            "filter": {
                "audio": {
                    "encode_to_ims_ac4": filter_json_node(filter),
                }
            },
            "output": output_json_node(storage_tag, &job.output),
            "misc": {
                "temp_dir": misc_json_node(&job.misc.temp_dir, job.misc.clean_temp),
            }
        }
    })
}

pub fn atmos_input_node_with_timecodes(
    storage_tag: &str,
    atmos_mezz: &ResolvedIo,
    input_timecode_frame_rate: &str,
    offset: &str,
    ffoa: &str,
) -> XmlNode {
    XmlNode::element(
        "input",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "atmos_mezz",
                vec![("version".to_string(), "1".to_string())],
                vec![
                    XmlNode::leaf("file_name", &atmos_mezz.file_names[0]),
                    XmlNode::leaf("timecode_frame_rate", input_timecode_frame_rate),
                    XmlNode::leaf("offset", offset),
                    XmlNode::leaf("ffoa", ffoa),
                    XmlNode::element(
                        "storage",
                        vec![],
                        vec![XmlNode::element(
                            storage_tag,
                            vec![],
                            vec![XmlNode::leaf("path", &atmos_mezz.storage_path)],
                        )],
                    ),
                ],
            )],
        )],
    )
}

pub fn atmos_input_json_node_with_timecodes(
    storage_tag: &str,
    atmos_mezz: &ResolvedIo,
    input_timecode_frame_rate: &str,
    offset: &str,
    ffoa: &str,
) -> JsonValue {
    json!({
        "atmos_mezz": {
            "-version": "1",
            "file_name": atmos_mezz.file_names[0],
            "timecode_frame_rate": input_timecode_frame_rate,
            "offset": offset,
            "ffoa": ffoa,
            "storage": thd_json::storage_node(storage_tag, &atmos_mezz.storage_path),
        }
    })
}

pub fn pcm_input_node_with_timecodes(
    storage_tag: &str,
    groups: &ResolvedInputGroups,
    input_timecode_frame_rate: &str,
    offset: &str,
    ffoa: &str,
) -> XmlNode {
    let mut children = Vec::new();

    if let Some(wav) = &groups.wav {
        children.push(XmlNode::element(
            "wav",
            vec![("version".to_string(), "1".to_string())],
            vec![
                XmlNode::leaf("file_name", &wav.file_names[0]),
                XmlNode::leaf("timecode_frame_rate", input_timecode_frame_rate),
                XmlNode::leaf("offset", offset),
                XmlNode::leaf("ffoa", ffoa),
                XmlNode::element(
                    "storage",
                    vec![],
                    vec![XmlNode::element(
                        storage_tag,
                        vec![],
                        vec![XmlNode::leaf("path", &wav.storage_path)],
                    )],
                ),
            ],
        ));
    }

    if let Some(wav_list) = &groups.wav_list {
        let channel_tags = [
            "file_name_L",
            "file_name_R",
            "file_name_C",
            "file_name_LFE",
            "file_name_LS",
            "file_name_RS",
        ];
        let mut wav_list_children: Vec<XmlNode> = channel_tags
            .iter()
            .zip(&wav_list.file_names)
            .map(|(tag, file_name)| XmlNode::leaf(*tag, file_name))
            .collect();
        wav_list_children.push(XmlNode::leaf("channel_configuration", "5.1"));
        wav_list_children.push(XmlNode::leaf(
            "timecode_frame_rate",
            input_timecode_frame_rate,
        ));
        wav_list_children.push(XmlNode::leaf("offset", offset));
        wav_list_children.push(XmlNode::leaf("ffoa", ffoa));
        wav_list_children.push(XmlNode::element(
            "storage",
            vec![],
            vec![XmlNode::element(
                storage_tag,
                vec![],
                vec![XmlNode::leaf("path", &wav_list.storage_path)],
            )],
        ));

        children.push(XmlNode::element(
            "wav_list",
            vec![("version".to_string(), "1".to_string())],
            wav_list_children,
        ));
    }

    XmlNode::element(
        "input",
        vec![],
        vec![XmlNode::element("audio", vec![], children)],
    )
}

pub fn pcm_input_json_node_with_timecodes(
    storage_tag: &str,
    groups: &ResolvedInputGroups,
    input_timecode_frame_rate: &str,
    offset: &str,
    ffoa: &str,
) -> JsonValue {
    let mut audio = Map::new();

    if let Some(wav) = &groups.wav {
        audio.insert(
            "wav".to_string(),
            json!({
                "-version": "1",
                "file_name": wav.file_names[0],
                "timecode_frame_rate": input_timecode_frame_rate,
                "offset": offset,
                "ffoa": ffoa,
                "storage": thd_json::storage_node(storage_tag, &wav.storage_path),
            }),
        );
    }

    if let Some(wav_list) = &groups.wav_list {
        let channel_tags = [
            "file_name_L",
            "file_name_R",
            "file_name_C",
            "file_name_LFE",
            "file_name_LS",
            "file_name_RS",
        ];
        let mut wav_list_node = Map::new();
        wav_list_node.insert("-version".to_string(), JsonValue::String("1".to_string()));
        for (tag, file_name) in channel_tags.iter().zip(&wav_list.file_names) {
            wav_list_node.insert((*tag).to_string(), JsonValue::String(file_name.clone()));
        }
        wav_list_node.insert(
            "channel_configuration".to_string(),
            JsonValue::String("5.1".to_string()),
        );
        wav_list_node.insert(
            "timecode_frame_rate".to_string(),
            JsonValue::String(input_timecode_frame_rate.to_string()),
        );
        wav_list_node.insert("offset".to_string(), JsonValue::String(offset.to_string()));
        wav_list_node.insert("ffoa".to_string(), JsonValue::String(ffoa.to_string()));
        wav_list_node.insert(
            "storage".to_string(),
            thd_json::storage_node(storage_tag, &wav_list.storage_path),
        );
        audio.insert("wav_list".to_string(), JsonValue::Object(wav_list_node));
    }

    JsonValue::Object(audio)
}

fn filter_node(filter: &Ac4ImsFilter) -> XmlNode {
    XmlNode::element(
        "filter",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "encode_to_ims_ac4",
                vec![("version".to_string(), "1".to_string())],
                vec![
                    XmlNode::leaf("timecode_frame_rate", &filter.timecode_frame_rate),
                    XmlNode::leaf("start", &filter.start),
                    XmlNode::leaf("end", &filter.end),
                    XmlNode::leaf("time_base", &filter.time_base),
                    XmlNode::leaf("prepend_silence_duration", &filter.prepend_silence_duration),
                    XmlNode::leaf("append_silence_duration", &filter.append_silence_duration),
                    XmlNode::element(
                        "loudness",
                        vec![],
                        vec![XmlNode::element(
                            "measure_only",
                            vec![],
                            vec![
                                XmlNode::leaf("metering_mode", &filter.metering_mode),
                                XmlNode::leaf(
                                    "dialogue_intelligence",
                                    if filter.dialogue_intelligence {
                                        "true"
                                    } else {
                                        "false"
                                    },
                                ),
                                XmlNode::leaf(
                                    "speech_threshold",
                                    filter.speech_threshold.to_string(),
                                ),
                            ],
                        )],
                    ),
                    XmlNode::leaf("data_rate", filter.data_rate.to_string()),
                    XmlNode::leaf("ac4_frame_rate", &filter.ac4_frame_rate),
                    XmlNode::leaf(
                        "ims_legacy_presentation",
                        if filter.ims_legacy_presentation {
                            "true"
                        } else {
                            "false"
                        },
                    ),
                    XmlNode::leaf("iframe_interval", filter.iframe_interval.to_string()),
                    XmlNode::leaf("language", &filter.language),
                    XmlNode::leaf("encoding_profile", &filter.encoding_profile),
                    XmlNode::element(
                        "drc",
                        vec![],
                        vec![
                            XmlNode::leaf("ddp_drc_profile", &filter.ddp_drc_profile),
                            XmlNode::leaf("flat_panel_drc_profile", &filter.flat_panel_drc_profile),
                            XmlNode::leaf(
                                "home_theatre_drc_profile",
                                &filter.home_theatre_drc_profile,
                            ),
                            XmlNode::leaf(
                                "portable_hp_drc_profile",
                                &filter.portable_hp_drc_profile,
                            ),
                            XmlNode::leaf(
                                "portable_spkr_drc_profile",
                                &filter.portable_spkr_drc_profile,
                            ),
                        ],
                    ),
                ],
            )],
        )],
    )
}

fn filter_json_node(filter: &Ac4ImsFilter) -> JsonValue {
    json!({
        "-version": "1",
        "timecode_frame_rate": filter.timecode_frame_rate,
        "start": filter.start,
        "end": filter.end,
        "time_base": filter.time_base,
        "prepend_silence_duration": filter.prepend_silence_duration,
        "append_silence_duration": filter.append_silence_duration,
        "loudness": {
            "measure_only": {
                "metering_mode": filter.metering_mode,
                "dialogue_intelligence": filter.dialogue_intelligence,
                "speech_threshold": filter.speech_threshold,
            }
        },
        "data_rate": filter.data_rate,
        "ac4_frame_rate": filter.ac4_frame_rate,
        "ims_legacy_presentation": filter.ims_legacy_presentation,
        "iframe_interval": filter.iframe_interval,
        "language": filter.language,
        "encoding_profile": filter.encoding_profile,
        "drc": {
            "ddp_drc_profile": filter.ddp_drc_profile,
            "flat_panel_drc_profile": filter.flat_panel_drc_profile,
            "home_theatre_drc_profile": filter.home_theatre_drc_profile,
            "portable_hp_drc_profile": filter.portable_hp_drc_profile,
            "portable_spkr_drc_profile": filter.portable_spkr_drc_profile,
        }
    })
}

fn output_node(storage_tag: &str, output: &ResolvedOutput) -> XmlNode {
    let storage = XmlNode::element(
        "storage",
        vec![],
        vec![XmlNode::element(
            storage_tag,
            vec![],
            vec![XmlNode::leaf("path", &output.storage_path)],
        )],
    );

    let child = match output.container {
        OutputContainer::Ac4 => XmlNode::element(
            "ac4",
            vec![("version".to_string(), "1".to_string())],
            vec![XmlNode::leaf("file_name", &output.file_names[0]), storage],
        ),
        OutputContainer::Mp4 => XmlNode::element(
            "mp4",
            vec![("version".to_string(), "1".to_string())],
            vec![
                XmlNode::leaf("file_name", &output.file_names[0]),
                XmlNode::leaf("output_format", "mp4"),
                XmlNode::leaf("override_frame_rate", "no"),
                XmlNode::leaf("fill_video", "false"),
                storage,
            ],
        ),
    };

    XmlNode::element("output", vec![], vec![child])
}

fn output_json_node(storage_tag: &str, output: &ResolvedOutput) -> JsonValue {
    let (tag, node) = match output.container {
        OutputContainer::Ac4 => (
            "ac4",
            json!({
                "-version": "1",
                "file_name": output.file_names[0],
                "storage": thd_json::storage_node(storage_tag, &output.storage_path),
            }),
        ),
        OutputContainer::Mp4 => (
            "mp4",
            json!({
                "-version": "1",
                "output_format": "mp4",
                "override_frame_rate": "no",
                "fill_video": false,
                "file_name": output.file_names[0],
                "storage": thd_json::storage_node(storage_tag, &output.storage_path),
            }),
        ),
    };

    let mut output_map = Map::new();
    output_map.insert(tag.to_string(), node);
    JsonValue::Object(output_map)
}

fn misc_node(temp_dir: &str, clean_temp: bool) -> XmlNode {
    XmlNode::element(
        "misc",
        vec![],
        vec![XmlNode::element(
            "temp_dir",
            vec![],
            vec![
                XmlNode::leaf("clean_temp", if clean_temp { "true" } else { "false" }),
                XmlNode::leaf("path", temp_dir),
            ],
        )],
    )
}

fn misc_json_node(temp_dir: &str, clean_temp: bool) -> JsonValue {
    json!({
        "clean_temp": thd_json::bool_string(clean_temp),
        "path": thd_json::quoted_path(temp_dir),
    })
}

fn reject_unsupported_overrides(overrides: &FilterOverrides) -> Result<()> {
    for key in [
        "channel_configuration",
        "bitstream_mode",
        "downmix_config",
        "line_mode_drc_profile",
        "rf_mode_drc_profile",
        "lfe_on",
        "dolby_surround_mode",
        "dolby_surround_ex_mode",
        "user_data",
        "lfe_lowpass_filter",
        "surround_90_degree_phase_shift",
        "surround_3db_attenuation",
        "loro_center_mix_level",
        "loro_surround_mix_level",
        "ltrt_center_mix_level",
        "ltrt_surround_mix_level",
        "preferred_downmix_mode",
        "allow_hybrid_downmix",
        "starting_timecode",
        "frame_rate",
        "surround_trim_5_1",
        "height_trim_5_1",
        "custom_dialnorm",
        "encoding_backend",
        "encoder_mode",
        "atmos_presentation_drc_profile",
        "spatial_clusters",
        "legacy_authoring_compatibility",
        "presentation_8ch_drc_profile",
        "presentation_6ch_drc_profile",
        "presentation_2ch_drc_profile",
        "optimize_data_rate",
    ] {
        if override_is_set(overrides, key) {
            bail!("parameter '{key}' is not supported by AC-4 IMS templates");
        }
    }
    Ok(())
}

fn override_is_set(overrides: &FilterOverrides, key: &str) -> bool {
    match key {
        "channel_configuration" => overrides.channel_configuration.is_some(),
        "bitstream_mode" => overrides.bitstream_mode.is_some(),
        "downmix_config" => overrides.downmix_config.is_some(),
        "line_mode_drc_profile" => overrides.line_mode_drc_profile.is_some(),
        "rf_mode_drc_profile" => overrides.rf_mode_drc_profile.is_some(),
        "lfe_on" => overrides.lfe_on.is_some(),
        "dolby_surround_mode" => overrides.dolby_surround_mode.is_some(),
        "dolby_surround_ex_mode" => overrides.dolby_surround_ex_mode.is_some(),
        "user_data" => overrides.user_data.is_some(),
        "lfe_lowpass_filter" => overrides.lfe_lowpass_filter.is_some(),
        "surround_90_degree_phase_shift" => overrides.surround_90_degree_phase_shift.is_some(),
        "surround_3db_attenuation" => overrides.surround_3db_attenuation.is_some(),
        "loro_center_mix_level" => overrides.loro_center_mix_level.is_some(),
        "loro_surround_mix_level" => overrides.loro_surround_mix_level.is_some(),
        "ltrt_center_mix_level" => overrides.ltrt_center_mix_level.is_some(),
        "ltrt_surround_mix_level" => overrides.ltrt_surround_mix_level.is_some(),
        "preferred_downmix_mode" => overrides.preferred_downmix_mode.is_some(),
        "allow_hybrid_downmix" => overrides.allow_hybrid_downmix.is_some(),
        "starting_timecode" => overrides.starting_timecode.is_some(),
        "frame_rate" => overrides.frame_rate.is_some(),
        "surround_trim_5_1" => overrides.surround_trim_5_1.is_some(),
        "height_trim_5_1" => overrides.height_trim_5_1.is_some(),
        "custom_dialnorm" => overrides.custom_dialnorm.is_some(),
        "encoding_backend" => overrides.encoding_backend.is_some(),
        "encoder_mode" => overrides.encoder_mode.is_some(),
        "atmos_presentation_drc_profile" => overrides.atmos_presentation_drc_profile.is_some(),
        "spatial_clusters" => overrides.spatial_clusters.is_some(),
        "legacy_authoring_compatibility" => overrides.legacy_authoring_compatibility.is_some(),
        "presentation_8ch_drc_profile" => overrides.presentation_8ch_drc_profile.is_some(),
        "presentation_6ch_drc_profile" => overrides.presentation_6ch_drc_profile.is_some(),
        "presentation_2ch_drc_profile" => overrides.presentation_2ch_drc_profile.is_some(),
        "optimize_data_rate" => overrides.optimize_data_rate.is_some(),
        _ => false,
    }
}

fn validate_string_param(key: &str, value: &str, ctx: &ValidationContext<'_>) -> Result<String> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Str(value.to_string()), ctx)? {
        ParamValue::Str(v) => Ok(v),
        _ => bail!("parameter {key} returned non-string value"),
    }
}

fn validate_int_param(key: &str, value: i64, ctx: &ValidationContext<'_>) -> Result<i64> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Int(value), ctx)? {
        ParamValue::Int(v) => Ok(v),
        _ => bail!("parameter {key} returned non-integer value"),
    }
}

fn validate_bool_param(key: &str, value: bool, ctx: &ValidationContext<'_>) -> Result<bool> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Bool(value), ctx)? {
        ParamValue::Bool(v) => Ok(v),
        _ => bail!("parameter {key} returned non-bool value"),
    }
}

fn find_schema_required(key: &str) -> Result<&'static ParamSchema> {
    find_schema(key).ok_or_else(|| anyhow::anyhow!("unknown parameter: {key}"))
}

fn validate_boundary_timecode(key: &str, value: &str) -> Result<String> {
    let special = match key {
        "start" => "first_frame_of_action",
        "end" => "end_of_file",
        other => bail!("unsupported boundary timecode key '{other}'"),
    };

    if value == special || is_valid_timecode(value) {
        Ok(value.to_string())
    } else {
        bail!(
            "invalid value '{value}' for {key}; expected {special}, HH:MM:SS:FF[df], or HH:MM:SS.xx"
        )
    }
}

fn validate_decimal_duration(key: &str, value: &str) -> Result<String> {
    if is_valid_decimal_duration(value) {
        Ok(value.to_string())
    } else {
        bail!("invalid value '{value}' for {key}; expected seconds.milliseconds")
    }
}

fn validate_input_timecode(key: &str, value: &str) -> Result<String> {
    if value == "auto" || is_valid_timecode(value) {
        return Ok(value.to_string());
    }

    bail!(
        "invalid value '{value}' for {key}; expected 'auto', HH:MM:SS:FF, HH:MM:SS:FFdf, or HH:MM:SS.xx"
    )
}

fn validate_language_tag(value: &str) -> Result<String> {
    if value.is_empty() {
        return Ok(String::new());
    }

    let lowered = value.to_ascii_lowercase();
    if ["mis", "mul", "und", "zxx", "qaa"].contains(&lowered.as_str()) {
        bail!("invalid value '{value}' for language; reserved language tags are not allowed");
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        Ok(value.to_string())
    } else {
        bail!("invalid value '{value}' for language; expected BCP-47 style alphanumeric tag");
    }
}

fn is_valid_timecode(value: &str) -> bool {
    parse_hh_mm_ss_ff(value).is_some() || parse_hh_mm_ss_decimal(value)
}

fn parse_hh_mm_ss_ff(value: &str) -> Option<()> {
    let core = value.strip_suffix("df").unwrap_or(value);
    let mut parts = core.split(':');
    let hours = parts.next()?;
    let minutes = parts.next()?;
    let seconds = parts.next()?;
    let frames = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    if hours.len() != 2
        || !hours.chars().all(|c| c.is_ascii_digit())
        || minutes.len() != 2
        || !minutes.chars().all(|c| c.is_ascii_digit())
        || seconds.len() != 2
        || !seconds.chars().all(|c| c.is_ascii_digit())
        || frames.len() != 2
        || !frames.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }

    Some(())
}

fn parse_hh_mm_ss_decimal(value: &str) -> bool {
    let mut parts = value.split(':');
    let Some(hours) = parts.next() else {
        return false;
    };
    let Some(minutes) = parts.next() else {
        return false;
    };
    let Some(seconds_and_fraction) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    if hours.len() != 2
        || !hours.chars().all(|c| c.is_ascii_digit())
        || minutes.len() != 2
        || !minutes.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }

    let Some((seconds, fraction)) = seconds_and_fraction.split_once('.') else {
        return false;
    };
    !seconds.is_empty()
        && seconds.len() == 2
        && seconds.chars().all(|c| c.is_ascii_digit())
        && !fraction.is_empty()
        && fraction.chars().all(|c| c.is_ascii_digit())
}

fn is_valid_decimal_duration(value: &str) -> bool {
    let Some((seconds, fraction)) = value.split_once('.') else {
        return !value.is_empty() && value.chars().all(|c| c.is_ascii_digit());
    };

    !seconds.is_empty()
        && seconds.chars().all(|c| c.is_ascii_digit())
        && !fraction.is_empty()
        && fraction.chars().all(|c| c.is_ascii_digit())
}
