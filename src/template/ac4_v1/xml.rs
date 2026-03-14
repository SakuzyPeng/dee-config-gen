use crate::{
    render::{XmlNode, render_file_name_list},
    resolve::ResolvedJob,
};

pub fn xml_structure(job: &ResolvedJob) -> XmlNode {
    let input_files = render_file_name_list(&job.input.file_names);
    let output_files = render_file_name_list(&job.output.file_names);

    XmlNode::element(
        "job_config",
        vec![],
        vec![
            input_node(&job.input.storage_path, &input_files),
            output_node(&job.output.storage_path, &output_files),
            misc_node(&job.misc.temp_dir, job.misc.clean_temp),
        ],
    )
}

fn input_node(input_storage_path: &str, input_files: &str) -> XmlNode {
    XmlNode::element(
        "input",
        vec![],
        vec![XmlNode::element(
            "audio",
            vec![],
            vec![XmlNode::element(
                "ac4",
                vec![("version".to_string(), "1".to_string())],
                vec![
                    XmlNode::leaf("file_name", input_files),
                    XmlNode::element(
                        "storage",
                        vec![],
                        vec![XmlNode::element(
                            "local",
                            vec![],
                            vec![XmlNode::leaf("path", input_storage_path)],
                        )],
                    ),
                ],
            )],
        )],
    )
}

fn output_node(output_storage_path: &str, output_files: &str) -> XmlNode {
    XmlNode::element(
        "output",
        vec![],
        vec![XmlNode::element(
            "ac4",
            vec![("version".to_string(), "1".to_string())],
            vec![
                XmlNode::leaf("file_name", output_files),
                XmlNode::element(
                    "storage",
                    vec![],
                    vec![XmlNode::element(
                        "local",
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
