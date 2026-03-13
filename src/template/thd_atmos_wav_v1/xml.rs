use crate::{
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    spec::JobMode,
};

use super::filter::ThdAtmosWavV1Filter;

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
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

    XmlNode::element(
        "job_config",
        vec![],
        vec![
            input_node(filter, storage_tag, atmos_mezz, wav),
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
    filter: &ThdAtmosWavV1Filter,
    storage_tag: &str,
    atmos_mezz: &crate::resolve::ResolvedIo,
    wav: &crate::resolve::ResolvedIo,
) -> XmlNode {
    XmlNode::element(
        "input",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![
                XmlNode::element(
                    "atmos_mezz",
                    vec![("version".to_string(), "1".to_string())],
                    vec![
                        XmlNode::leaf("file_name", &atmos_mezz.file_names[0]),
                        XmlNode::leaf("timecode_frame_rate", "not_indicated"),
                        XmlNode::leaf("offset", "auto"),
                        XmlNode::leaf("ffoa", "auto"),
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
                ),
                XmlNode::element(
                    "wav",
                    vec![("version".to_string(), "1".to_string())],
                    vec![
                        XmlNode::leaf("file_name", &wav.file_names[0]),
                        XmlNode::leaf("timecode_frame_rate", &filter.input_timecode_frame_rate),
                        XmlNode::leaf("offset", &filter.offset),
                        XmlNode::leaf("ffoa", &filter.ffoa),
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
                ),
            ],
        )],
    )
}

fn filter_node(filter: &ThdAtmosWavV1Filter) -> XmlNode {
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
