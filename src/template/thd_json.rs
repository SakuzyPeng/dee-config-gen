use serde_json::{Map, Value, json};

pub(crate) struct ThdFilterJson<'a> {
    pub metering_mode: &'a str,
    pub dialogue_intelligence: bool,
    pub speech_threshold: u8,
    pub timecode_frame_rate: &'a str,
    pub start: &'a str,
    pub end: &'a str,
    pub time_base: &'a str,
    pub prepend_silence_duration: &'a str,
    pub append_silence_duration: &'a str,
    pub atmos_presentation_drc_profile: &'a str,
    pub spatial_clusters: &'a str,
    pub legacy_authoring_compatibility: bool,
    pub presentation_8ch_drc_profile: &'a str,
    pub presentation_6ch_drc_profile: &'a str,
    pub presentation_2ch_drc_profile: &'a str,
    pub optimize_data_rate: bool,
    pub starting_timecode: &'a str,
    pub frame_rate: &'a str,
    pub custom_dialnorm: i8,
}

pub(crate) fn quoted_path(path: &str) -> String {
    format!("\"{path}\"")
}

pub(crate) fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

pub(crate) fn storage_node(storage_tag: &str, path: &str) -> Value {
    let mut storage = Map::new();
    storage.insert(
        storage_tag.to_string(),
        json!({
            "path": quoted_path(path),
        }),
    );
    Value::Object(storage)
}

pub(crate) fn output_node(
    storage_tag: &str,
    output_storage_path: &str,
    output_files: &str,
) -> Value {
    json!({
        "-version": "1",
        "file_name": output_files,
        "storage": storage_node(storage_tag, output_storage_path),
    })
}

pub(crate) fn misc_node(temp_dir: &str, clean_temp: bool) -> Value {
    json!({
        "clean_temp": bool_string(clean_temp),
        "path": quoted_path(temp_dir),
    })
}

pub(crate) fn filter_node(filter: ThdFilterJson<'_>) -> Value {
    json!({
        "-version": "1",
        "loudness_measurement": {
            "metering_mode": filter.metering_mode,
            "dialogue_intelligence": filter.dialogue_intelligence,
            "speech_threshold": filter.speech_threshold,
        },
        "timecode_frame_rate": filter.timecode_frame_rate,
        "start": filter.start,
        "end": filter.end,
        "time_base": filter.time_base,
        "prepend_silence_duration": filter.prepend_silence_duration,
        "append_silence_duration": filter.append_silence_duration,
        "atmos_presentation": {
            "drc_profile": filter.atmos_presentation_drc_profile,
            "spatial_clusters": filter.spatial_clusters,
            "legacy_authoring_compatibility": bool_string(filter.legacy_authoring_compatibility),
        },
        "presentation_8ch": {
            "drc_profile": filter.presentation_8ch_drc_profile,
            "surround_3db_attenuation": "true",
        },
        "presentation_6ch": {
            "drc_profile": filter.presentation_6ch_drc_profile,
            "surround_3db_attenuation": "true",
        },
        "presentation_2ch": {
            "drc_profile": filter.presentation_2ch_drc_profile,
            "drc_default_on": "true",
            "format": "stereo",
        },
        "optimize_data_rate": bool_string(filter.optimize_data_rate),
        "embedded_timecodes": {
            "starting_timecode": filter.starting_timecode,
            "frame_rate": filter.frame_rate,
        },
        "log_format": "txt",
        "custom_dialnorm": filter.custom_dialnorm.to_string(),
    })
}
