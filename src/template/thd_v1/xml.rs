use crate::{
    render::{XmlNode, render_file_name_list},
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
};

use super::filter::ThdV1Filter;

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
    let ResolvedFilter::ThdV1(filter) = &job.filter else {
        panic!("thd_v1 received wrong ResolvedFilter variant");
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
    filter: &ThdV1Filter,
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

fn filter_node(filter: &ThdV1Filter) -> XmlNode {
    XmlNode::element(
        "filter",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "encode_to_dthd",
                vec![("version".to_string(), "1".to_string())],
                vec![
                    XmlNode::element(
                        "loudness_measurement",
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
                            XmlNode::leaf("speech_threshold", filter.speech_threshold.to_string()),
                        ],
                    ),
                    XmlNode::leaf("timecode_frame_rate", &filter.timecode_frame_rate),
                    XmlNode::leaf("start", &filter.start),
                    XmlNode::leaf("end", &filter.end),
                    XmlNode::leaf("time_base", &filter.time_base),
                    XmlNode::leaf("prepend_silence_duration", &filter.prepend_silence_duration),
                    XmlNode::leaf("append_silence_duration", &filter.append_silence_duration),
                    XmlNode::element(
                        "atmos_presentation",
                        vec![],
                        vec![
                            XmlNode::leaf("drc_profile", &filter.atmos_presentation_drc_profile),
                            XmlNode::leaf("spatial_clusters", &filter.spatial_clusters),
                            XmlNode::leaf(
                                "legacy_authoring_compatibility",
                                if filter.legacy_authoring_compatibility {
                                    "true"
                                } else {
                                    "false"
                                },
                            ),
                        ],
                    ),
                    XmlNode::element(
                        "presentation_8ch",
                        vec![],
                        vec![
                            XmlNode::leaf("drc_profile", &filter.presentation_8ch_drc_profile),
                            XmlNode::leaf("surround_3db_attenuation", "true"),
                        ],
                    ),
                    XmlNode::element(
                        "presentation_6ch",
                        vec![],
                        vec![
                            XmlNode::leaf("drc_profile", &filter.presentation_6ch_drc_profile),
                            XmlNode::leaf("surround_3db_attenuation", "true"),
                        ],
                    ),
                    XmlNode::element(
                        "presentation_2ch",
                        vec![],
                        vec![
                            XmlNode::leaf("drc_profile", &filter.presentation_2ch_drc_profile),
                            XmlNode::leaf("drc_default_on", "true"),
                            XmlNode::leaf("format", "stereo"),
                        ],
                    ),
                    XmlNode::leaf(
                        "optimize_data_rate",
                        if filter.optimize_data_rate {
                            "true"
                        } else {
                            "false"
                        },
                    ),
                    XmlNode::element(
                        "embedded_timecodes",
                        vec![],
                        vec![
                            XmlNode::leaf("starting_timecode", &filter.starting_timecode),
                            XmlNode::leaf("frame_rate", &filter.frame_rate),
                        ],
                    ),
                    XmlNode::leaf("log_format", "txt"),
                    XmlNode::leaf("custom_dialnorm", filter.custom_dialnorm.to_string()),
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
            "mlp",
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
