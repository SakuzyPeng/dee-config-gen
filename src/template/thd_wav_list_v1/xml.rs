use crate::{
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
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

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
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

    XmlNode::element(
        "job_config",
        vec![],
        vec![
            input_node(
                filter,
                storage_tag,
                &job.input.storage_path,
                &job.input.file_names,
                &effective_channel_configuration,
            ),
            filter_node(filter),
            output_node(
                storage_tag,
                &job.output.storage_path,
                &job.output.file_names[0],
            ),
            misc_node(&job.misc.temp_dir, job.misc.clean_temp),
        ],
    )
}

fn input_node(
    filter: &ThdWavListV1Filter,
    storage_tag: &str,
    input_storage_path: &str,
    input_files: &[String],
    channel_configuration: &str,
) -> XmlNode {
    let channel_slot_count = match channel_configuration {
        "stereo" => 2,
        "5.1" => 6,
        "7.1" => 8,
        other => panic!("unsupported channel configuration {other}"),
    };

    let mut wav_list_children: Vec<XmlNode> = CHANNEL_FIELD_TAGS
        .iter()
        .take(channel_slot_count)
        .enumerate()
        .map(|(index, (_, tag))| {
            let file_name = input_files
                .get(index)
                .map(std::string::String::as_str)
                .expect("thd_wav_list_v1 input slots should be validated before rendering");
            XmlNode::leaf(*tag, file_name)
        })
        .collect();
    wav_list_children.push(XmlNode::leaf(
        "channel_configuration",
        channel_configuration,
    ));
    wav_list_children.push(XmlNode::leaf(
        "timecode_frame_rate",
        &filter.input_timecode_frame_rate,
    ));
    wav_list_children.push(XmlNode::leaf("offset", &filter.offset));
    wav_list_children.push(XmlNode::leaf("ffoa", &filter.ffoa));
    wav_list_children.push(XmlNode::element(
        "storage",
        vec![],
        vec![XmlNode::element(
            storage_tag,
            vec![],
            vec![XmlNode::leaf("path", input_storage_path)],
        )],
    ));

    XmlNode::element(
        "input",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "wav_list",
                vec![("version".to_string(), "1".to_string())],
                wav_list_children,
            )],
        )],
    )
}

fn filter_node(filter: &ThdWavListV1Filter) -> XmlNode {
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

fn output_node(storage_tag: &str, output_storage_path: &str, output_file: &str) -> XmlNode {
    XmlNode::element(
        "output",
        vec![],
        vec![XmlNode::element(
            "mlp",
            vec![("version".to_string(), "1".to_string())],
            vec![
                XmlNode::leaf("file_name", output_file),
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
