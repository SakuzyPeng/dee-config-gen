use crate::{
    config::JobMode,
    render::{Predicate, XmlNode, render_file_name_list},
    resolve::{ResolvedFilter, ResolvedJob},
};

use super::filter::AtmosEc3V1Filter;

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
    let ResolvedFilter::AtmosEc3V1(filter) = &job.filter else {
        panic!("atmos_ec3_v1 received wrong ResolvedFilter variant");
    };

    let storage_tag = match job.job_mode {
        JobMode::Single => "local",
        JobMode::Album => "local_multi_path",
    };

    let input_files = render_file_name_list(&job.input.file_names);
    let output_files = render_file_name_list(&job.output.file_names);

    XmlNode::element(
        "job_config",
        vec![],
        vec![
            input_node(filter, storage_tag, &job.input.storage_path, &input_files),
            filter_node(filter),
            output_node(storage_tag, &job.output.storage_path, &output_files),
            misc_node(&job.misc.temp_dir, job.misc.clean_temp),
        ],
    )
}

fn input_node(
    filter: &AtmosEc3V1Filter,
    storage_tag: &str,
    input_storage_path: &str,
    input_files: &str,
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
                    XmlNode::leaf("file_name", input_files),
                    XmlNode::leaf("timecode_frame_rate", &filter.timecode_frame_rate),
                    XmlNode::leaf("offset", "auto"),
                    XmlNode::leaf("ffoa", "auto"),
                    XmlNode::element(
                        "storage",
                        vec![],
                        vec![XmlNode::element(
                            storage_tag,
                            vec![],
                            vec![XmlNode::leaf("path", input_storage_path)],
                        )],
                    ),
                ],
            )],
        )],
    )
}

fn filter_node(filter: &AtmosEc3V1Filter) -> XmlNode {
    XmlNode::element(
        "filter",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "encode_to_atmos_ddp",
                vec![("version".to_string(), "1".to_string())],
                vec![
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
                    XmlNode::leaf("timecode_frame_rate", &filter.timecode_frame_rate),
                    XmlNode::leaf("start", &filter.start),
                    XmlNode::leaf("end", &filter.end),
                    XmlNode::leaf("time_base", &filter.time_base),
                    XmlNode::leaf("prepend_silence_duration", &filter.prepend_silence_duration),
                    XmlNode::leaf("append_silence_duration", &filter.append_silence_duration),
                    XmlNode::element(
                        "drc",
                        vec![],
                        vec![
                            XmlNode::leaf("line_mode_drc_profile", &filter.line_mode_drc_profile),
                            XmlNode::leaf("rf_mode_drc_profile", &filter.rf_mode_drc_profile),
                        ],
                    ),
                    XmlNode::element(
                        "downmix",
                        vec![],
                        vec![
                            XmlNode::leaf("loro_center_mix_level", &filter.loro_center_mix_level),
                            XmlNode::leaf(
                                "loro_surround_mix_level",
                                &filter.loro_surround_mix_level,
                            ),
                            XmlNode::leaf("ltrt_center_mix_level", &filter.ltrt_center_mix_level),
                            XmlNode::leaf(
                                "ltrt_surround_mix_level",
                                &filter.ltrt_surround_mix_level,
                            ),
                            XmlNode::leaf("preferred_downmix_mode", &filter.preferred_downmix_mode),
                        ],
                    ),
                    XmlNode::element(
                        "custom_trims",
                        vec![],
                        vec![
                            XmlNode::leaf("surround_trim_5_1", &filter.surround_trim_5_1),
                            XmlNode::leaf("height_trim_5_1", &filter.height_trim_5_1),
                        ],
                    ),
                    XmlNode::leaf("custom_dialnorm", filter.custom_dialnorm.to_string()),
                    XmlNode::when_node(
                        Predicate::ParamSome("encoding_backend".to_string()),
                        vec![XmlNode::leaf(
                            "encoding_backend",
                            filter.encoding_backend.clone().unwrap_or_default(),
                        )],
                    ),
                    XmlNode::when_node(
                        Predicate::ParamSome("encoder_mode".to_string()),
                        vec![XmlNode::leaf(
                            "encoder_mode",
                            filter.encoder_mode.clone().unwrap_or_default(),
                        )],
                    ),
                ],
            )],
        )],
    )
}

fn output_node(storage_tag: &str, output_storage_path: &str, output_files: &str) -> XmlNode {
    XmlNode::element(
        "output",
        vec![],
        vec![XmlNode::element(
            "ec3",
            vec![("version".to_string(), "1".to_string())],
            vec![
                XmlNode::leaf("file_name", output_files),
                XmlNode::element(
                    "storage",
                    vec![],
                    vec![XmlNode::element(
                        storage_tag,
                        vec![],
                        vec![XmlNode::leaf("path", output_storage_path)],
                    )],
                ),
            ],
        )],
    )
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
