use serde_json::{Value, json};

use crate::{
    config::JobMode,
    resolve::{ResolvedFilter, ResolvedIo, ResolvedJob},
    template::thd_json::{self, ThdFilterJson},
};

use super::filter::ThdAtmosWavV1Filter;

pub fn json_structure(job: &ResolvedJob) -> Value {
    let ResolvedFilter::ThdAtmosWavV1(filter) = &job.filter else {
        panic!("thd_atmos_wav_v1 received wrong ResolvedFilter variant");
    };

    let input_groups = job
        .input_groups
        .as_ref()
        .expect("thd_atmos_wav_v1 requires resolved input_groups");
    let atmos_mezz = input_groups
        .atmos_mezz
        .as_ref()
        .expect("thd_atmos_wav_v1 requires atmos_mezz input group");
    let wav = input_groups
        .wav
        .as_ref()
        .expect("thd_atmos_wav_v1 requires wav input group");

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    json!({
        "job_config": {
            "input": {
                "audio": {
                    "atmos_mezz": atmos_mezz_input_node(storage_tag, atmos_mezz),
                    "wav": wav_input_node(filter, storage_tag, wav)
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

fn atmos_mezz_input_node(storage_tag: &str, atmos_mezz: &ResolvedIo) -> Value {
    json!({
        "-version": "1",
        "file_name": atmos_mezz.file_names[0],
        "timecode_frame_rate": "not_indicated",
        "offset": "auto",
        "ffoa": "auto",
        "storage": thd_json::storage_node(storage_tag, &atmos_mezz.storage_path),
    })
}

fn wav_input_node(filter: &ThdAtmosWavV1Filter, storage_tag: &str, wav: &ResolvedIo) -> Value {
    json!({
        "-version": "1",
        "file_name": wav.file_names[0],
        "timecode_frame_rate": filter.input_timecode_frame_rate,
        "offset": filter.offset,
        "ffoa": filter.ffoa,
        "storage": thd_json::storage_node(storage_tag, &wav.storage_path),
    })
}

fn filter_node(filter: &ThdAtmosWavV1Filter) -> Value {
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
