use serde_json::{Map, Value, json};

use crate::{
    config::JobMode,
    render::render_file_name_list,
    resolve::{ResolvedFilter, ResolvedJob},
};

use super::filter::AtmosEc3V1Filter;

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::AtmosEc3V1(filter) = &job.filter else {
        panic!("atmos_ec3_v1 received wrong ResolvedFilter variant");
    };

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    let input_files = render_file_name_list(&job.input.file_names);
    let output_files = render_file_name_list(&job.output.file_names);

    json!({
        "job_config": {
            "input": {
                "audio": {
                    "atmos_mezz": input_node(filter, storage_tag, &job.input.storage_path, &input_files)
                }
            },
            "filter": {
                "audio": {
                    "encode_to_atmos_ddp": filter_node(filter)
                }
            },
            "output": {
                "ec3": output_node(storage_tag, &job.output.storage_path, &output_files)
            },
            "misc": {
                "temp_dir": misc_node(&job.misc.temp_dir, job.misc.clean_temp)
            }
        }
    })
}

fn input_node(
    filter: &AtmosEc3V1Filter,
    storage_tag: &str,
    input_storage_path: &str,
    input_files: &str,
) -> Value {
    let mut storage = Map::new();
    storage.insert(
        storage_tag.to_string(),
        json!({
            "path": quoted_path(input_storage_path),
        }),
    );

    json!({
        "-version": "1",
        "file_name": input_files,
        "timecode_frame_rate": filter.timecode_frame_rate,
        "offset": "auto",
        "ffoa": "auto",
        "storage": Value::Object(storage),
    })
}

fn filter_node(filter: &AtmosEc3V1Filter) -> Value {
    let mut node = json!({
        "-version": "1",
        "loudness": {
            "measure_only": {
                "metering_mode": filter.metering_mode,
                "dialogue_intelligence": filter.dialogue_intelligence,
                "speech_threshold": filter.speech_threshold,
            }
        },
        "data_rate": filter.data_rate,
        "timecode_frame_rate": filter.timecode_frame_rate,
        "start": filter.start,
        "end": filter.end,
        "time_base": filter.time_base,
        "prepend_silence_duration": filter.prepend_silence_duration,
        "append_silence_duration": filter.append_silence_duration,
        "drc": {
            "line_mode_drc_profile": filter.line_mode_drc_profile,
            "rf_mode_drc_profile": filter.rf_mode_drc_profile,
        },
        "downmix": {
            "loro_center_mix_level": filter.loro_center_mix_level,
            "loro_surround_mix_level": filter.loro_surround_mix_level,
            "ltrt_center_mix_level": filter.ltrt_center_mix_level,
            "ltrt_surround_mix_level": filter.ltrt_surround_mix_level,
            "preferred_downmix_mode": filter.preferred_downmix_mode,
        },
        "custom_trims": {
            "surround_trim_5_1": filter.surround_trim_5_1,
            "height_trim_5_1": filter.height_trim_5_1,
        },
        "custom_dialnorm": filter.custom_dialnorm,
    });

    let object = node
        .as_object_mut()
        .expect("encode_to_atmos_ddp json node must be object");

    if let Some(value) = &filter.encoding_backend {
        object.insert("encoding_backend".to_string(), Value::String(value.clone()));
    }
    if let Some(value) = &filter.encoder_mode {
        object.insert("encoder_mode".to_string(), Value::String(value.clone()));
    }

    node
}

fn output_node(storage_tag: &str, output_storage_path: &str, output_files: &str) -> Value {
    let mut storage = Map::new();
    storage.insert(
        storage_tag.to_string(),
        json!({
            "path": quoted_path(output_storage_path),
        }),
    );

    json!({
        "-version": "1",
        "file_name": output_files,
        "storage": Value::Object(storage),
    })
}

fn misc_node(temp_dir: &str, clean_temp: bool) -> Value {
    json!({
        "clean_temp": if clean_temp { "true" } else { "false" },
        "path": quoted_path(temp_dir),
    })
}

fn quoted_path(path: &str) -> String {
    format!("\"{path}\"")
}
