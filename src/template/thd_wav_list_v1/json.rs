use serde_json::{Map, Value, json};

use crate::{
    config::JobMode,
    resolve::{ResolvedFilter, ResolvedJob},
    template::thd_json::{self, ThdFilterJson},
};

use super::{effective_channel_configuration, filter::ThdWavListV1Filter};

const CHANNEL_FIELD_TAGS: &[(&str, &str)] = &[
    ("L", "file_name_L"),
    ("R", "file_name_R"),
    ("C", "file_name_C"),
    ("LFE", "file_name_LFE"),
    ("LS", "file_name_LS"),
    ("RS", "file_name_RS"),
    ("LRS", "file_name_LRS"),
    ("RRS", "file_name_RRS"),
];

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::ThdWavListV1(filter) = &job.filter else {
        panic!("thd_wav_list_v1 received wrong ResolvedFilter variant");
    };

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    let effective_channel_configuration =
        effective_channel_configuration(&filter.channel_configuration, job.input.file_names.len())
            .expect("thd_wav_list_v1 channel configuration should be validated before rendering");

    json!({
        "job_config": {
            "input": {
                "audio": {
                    "wav_list": input_node(
                        filter,
                        storage_tag,
                        &job.input.storage_path,
                        &job.input.file_names,
                        &effective_channel_configuration,
                    )
                }
            },
            "filter": {
                "audio": {
                    "encode_to_dthd": filter_node(filter)
                }
            },
            "output": {
                "mlp": thd_json::output_node(storage_tag, &job.output.storage_path, &job.output.file_names[0])
            },
            "misc": {
                "temp_dir": thd_json::misc_node(&job.misc.temp_dir, job.misc.clean_temp)
            }
        }
    })
}

fn input_node(
    filter: &ThdWavListV1Filter,
    storage_tag: &str,
    input_storage_path: &str,
    input_files: &[String],
    channel_configuration: &str,
) -> Value {
    let channel_slot_count = match channel_configuration {
        "stereo" => 2,
        "5.1" => 6,
        "7.1" => 8,
        other => panic!("unsupported channel configuration {other}"),
    };

    let mut node = Map::new();
    node.insert("-version".to_string(), Value::String("1".to_string()));
    for (index, (_, tag)) in CHANNEL_FIELD_TAGS
        .iter()
        .take(channel_slot_count)
        .enumerate()
    {
        let file_name = input_files
            .get(index)
            .map(std::string::String::as_str)
            .expect("thd_wav_list_v1 input slots should be validated before rendering");
        node.insert((*tag).to_string(), Value::String(file_name.to_string()));
    }
    node.insert(
        "channel_configuration".to_string(),
        Value::String(channel_configuration.to_string()),
    );
    node.insert(
        "timecode_frame_rate".to_string(),
        Value::String(filter.input_timecode_frame_rate.clone()),
    );
    node.insert("offset".to_string(), Value::String(filter.offset.clone()));
    node.insert("ffoa".to_string(), Value::String(filter.ffoa.clone()));
    node.insert(
        "storage".to_string(),
        thd_json::storage_node(storage_tag, input_storage_path),
    );
    Value::Object(node)
}

fn filter_node(filter: &ThdWavListV1Filter) -> Value {
    thd_json::filter_node(ThdFilterJson {
        metering_mode: &filter.metering_mode,
        dialogue_intelligence: filter.dialogue_intelligence,
        speech_threshold: filter.speech_threshold,
        timecode_frame_rate: &filter.timecode_frame_rate,
        start: &filter.start,
        end: &filter.end,
        time_base: &filter.time_base,
        prepend_silence_duration: &filter.prepend_silence_duration,
        append_silence_duration: &filter.append_silence_duration,
        atmos_presentation_drc_profile: &filter.atmos_presentation_drc_profile,
        spatial_clusters: &filter.spatial_clusters,
        legacy_authoring_compatibility: filter.legacy_authoring_compatibility,
        presentation_8ch_drc_profile: &filter.presentation_8ch_drc_profile,
        presentation_6ch_drc_profile: &filter.presentation_6ch_drc_profile,
        presentation_2ch_drc_profile: &filter.presentation_2ch_drc_profile,
        optimize_data_rate: filter.optimize_data_rate,
        starting_timecode: &filter.starting_timecode,
        frame_rate: &filter.frame_rate,
        custom_dialnorm: filter.custom_dialnorm,
    })
}
