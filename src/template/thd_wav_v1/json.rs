use serde_json::{Value, json};

use crate::{
    config::JobMode,
    render::render_file_name_list,
    resolve::{ResolvedFilter, ResolvedJob},
    template::thd_json::{self, ThdFilterJson},
};

use super::filter::ThdWavV1Filter;

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::ThdWavV1(filter) = &job.filter else {
        panic!("thd_wav_v1 received wrong ResolvedFilter variant");
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
                    "wav": input_node(filter, storage_tag, &job.input.storage_path, &input_files)
                }
            },
            "filter": {
                "audio": {
                    "encode_to_dthd": filter_node(filter)
                }
            },
            "output": {
                "mlp": thd_json::output_node(storage_tag, &job.output.storage_path, &output_files)
            },
            "misc": {
                "temp_dir": thd_json::misc_node(&job.misc.temp_dir, job.misc.clean_temp)
            }
        }
    })
}

fn input_node(
    filter: &ThdWavV1Filter,
    storage_tag: &str,
    input_storage_path: &str,
    input_files: &str,
) -> Value {
    json!({
        "-version": "1",
        "file_name": input_files,
        "timecode_frame_rate": filter.input_timecode_frame_rate,
        "offset": filter.offset,
        "ffoa": filter.ffoa,
        "storage": thd_json::storage_node(storage_tag, input_storage_path),
    })
}

fn filter_node(filter: &ThdWavV1Filter) -> Value {
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
