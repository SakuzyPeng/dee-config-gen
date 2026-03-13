use serde_json::{Map, Value, json};

use crate::{
    render::render_file_name_list,
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
};

use super::filter::PcmDdpV1Filter;

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::PcmDdpV1(filter) = &job.filter else {
        panic!("pcm_ddp_v1 received wrong ResolvedFilter variant");
    };

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    let input_files = render_file_name_list(&job.input.file_names);
    let output_files = render_file_name_list(&job.output.file_names);
    let output_tag = match job.encode_mode.as_str() {
        "dd" => "ac3",
        "ddp" | "ddp71" | "bluray" => "ec3",
        other => panic!("pcm_ddp_v1 received unsupported encode_mode '{other}'"),
    };

    json!({
        "job_config": {
            "input": {
                "audio": {
                    "wav": input_node(storage_tag, &job.input.storage_path, &input_files)
                }
            },
            "filter": {
                "audio": {
                    "pcm_to_ddp": filter_node(filter)
                }
            },
            "output": {
                output_tag: output_node(storage_tag, &job.output.storage_path, &output_files)
            },
            "misc": {
                "temp_dir": misc_node(&job.misc.temp_dir, job.misc.clean_temp)
            }
        }
    })
}

fn input_node(storage_tag: &str, input_storage_path: &str, input_files: &str) -> Value {
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
        "timecode_frame_rate": "not_indicated",
        "offset": "auto",
        "ffoa": "auto",
        "storage": Value::Object(storage),
    })
}

fn filter_node(filter: &PcmDdpV1Filter) -> Value {
    json!({
        "-version": "3",
        "loudness": {
            "measure_only": {
                "metering_mode": filter.metering_mode,
                "dialogue_intelligence": filter.dialogue_intelligence,
                "speech_threshold": filter.speech_threshold,
            }
        },
        "encoder_mode": filter.encoder_mode,
        "bitstream_mode": filter.bitstream_mode,
        "downmix_config": filter.downmix_config,
        "data_rate": filter.data_rate,
        "timecode_frame_rate": filter.timecode_frame_rate,
        "start": filter.start,
        "end": filter.end,
        "time_base": filter.time_base,
        "prepend_silence_duration": filter.prepend_silence_duration,
        "append_silence_duration": filter.append_silence_duration,
        "lfe_on": bool_string(filter.lfe_on),
        "dolby_surround_mode": filter.dolby_surround_mode,
        "dolby_surround_ex_mode": filter.dolby_surround_ex_mode,
        "user_data": filter.user_data.to_string(),
        "drc": {
            "line_mode_drc_profile": filter.line_mode_drc_profile,
            "rf_mode_drc_profile": filter.rf_mode_drc_profile,
        },
        "lfe_lowpass_filter": bool_string(filter.lfe_lowpass_filter),
        "surround_90_degree_phase_shift": bool_string(filter.surround_90_degree_phase_shift),
        "surround_3db_attenuation": bool_string(filter.surround_3db_attenuation),
        "downmix": {
            "loro_center_mix_level": filter.loro_center_mix_level,
            "loro_surround_mix_level": filter.loro_surround_mix_level,
            "ltrt_center_mix_level": filter.ltrt_center_mix_level,
            "ltrt_surround_mix_level": filter.ltrt_surround_mix_level,
            "preferred_downmix_mode": filter.preferred_downmix_mode,
        },
        "allow_hybrid_downmix": bool_string(filter.allow_hybrid_downmix),
        "embedded_timecodes": {
            "starting_timecode": filter.starting_timecode,
            "frame_rate": filter.frame_rate,
        },
        "custom_dialnorm": filter.custom_dialnorm.to_string(),
    })
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
        "clean_temp": bool_string(clean_temp),
        "path": quoted_path(temp_dir),
    })
}

fn quoted_path(path: &str) -> String {
    format!("\"{path}\"")
}

fn bool_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
