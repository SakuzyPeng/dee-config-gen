use crate::{resolve::ResolvedJob, template::TemplateRegistry};

#[derive(Debug, Clone)]
pub enum XmlNode {
    Element {
        tag: String,
        attrs: Vec<(String, String)>,
        children: Vec<XmlNode>,
    },
    Leaf {
        tag: String,
        value: String,
    },
    When {
        predicate: Predicate,
        children: Vec<XmlNode>,
    },
}

impl XmlNode {
    pub fn element(
        tag: impl Into<String>,
        attrs: Vec<(String, String)>,
        children: Vec<Self>,
    ) -> Self {
        Self::Element {
            tag: tag.into(),
            attrs,
            children,
        }
    }

    pub fn leaf(tag: impl Into<String>, value: impl Into<String>) -> Self {
        Self::Leaf {
            tag: tag.into(),
            value: value.into(),
        }
    }

    pub fn when_node(predicate: Predicate, children: Vec<Self>) -> Self {
        Self::When {
            predicate,
            children,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Predicate {
    Always,
    Mode(&'static str),
    ModeNot(&'static str),
    ParamSome(String),
    And(Vec<Predicate>),
}

pub fn render_xml(job: &ResolvedJob) -> String {
    let template = TemplateRegistry::get(&job.template_id).unwrap_or_else(|err| {
        panic!(
            "template_id '{}' should be validated before rendering, but lookup failed: {err}",
            job.template_id
        )
    });
    let tree = template.xml_structure(job);
    render_xml_tree(&tree, job)
}

pub fn render_xml_tree(root: &XmlNode, job: &ResolvedJob) -> String {
    let mut buf = String::with_capacity(4096);
    push_line(&mut buf, 0, "<?xml version=\"1.0\"?>");
    render_node(&mut buf, 0, root, job);
    buf
}

pub fn render_file_name_list(files: &[String]) -> String {
    files
        .iter()
        .map(|f| {
            if f.contains(' ') {
                format!("\"{}\"", f)
            } else {
                f.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_node(buf: &mut String, indent: usize, node: &XmlNode, job: &ResolvedJob) {
    match node {
        XmlNode::Element {
            tag,
            attrs,
            children,
        } => {
            let attrs = if attrs.is_empty() {
                String::new()
            } else {
                format!(
                    " {}",
                    attrs
                        .iter()
                        .map(|(k, v)| format!("{k}=\"{}\"", xml_escape(v)))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            };
            push_line(buf, indent, &format!("<{tag}{attrs}>"));
            for child in children {
                render_node(buf, indent + 1, child, job);
            }
            push_line(buf, indent, &format!("</{tag}>"));
        }
        XmlNode::Leaf { tag, value } => {
            push_line(
                buf,
                indent,
                &format!("<{tag}>{}</{tag}>", xml_escape(value)),
            );
        }
        XmlNode::When {
            predicate,
            children,
        } => {
            if eval_predicate(predicate, job) {
                for child in children {
                    render_node(buf, indent, child, job);
                }
            }
        }
    }
}

fn eval_predicate(predicate: &Predicate, job: &ResolvedJob) -> bool {
    match predicate {
        Predicate::Always => true,
        Predicate::Mode(mode) => job.encode_mode.as_str() == *mode,
        Predicate::ModeNot(mode) => job.encode_mode.as_str() != *mode,
        Predicate::ParamSome(param) => job.filter.param_some(param),
        Predicate::And(all) => all.iter().all(|item| eval_predicate(item, job)),
    }
}

fn push_line(buf: &mut String, indent: usize, value: &str) {
    let spaces = "  ".repeat(indent);
    buf.push_str(&spaces);
    buf.push_str(value);
    buf.push('\n');
}

fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use crate::{
        config::{EncodeMode, JobMode, Profile},
        resolve::{ResolvedFilter, ResolvedIo, ResolvedJob, ResolvedMisc},
        template::atmos_ec3_v1::AtmosEc3V1Filter,
    };

    use super::{Predicate, XmlNode, render_xml_tree};

    fn sample_job() -> ResolvedJob {
        ResolvedJob {
            template_id: "atmos_ec3_v1".to_string(),
            profile: Profile::Standard,
            job_mode: JobMode::Single,
            encode_mode: EncodeMode::Streaming,
            input: ResolvedIo {
                storage_path: "Y:/in".to_string(),
                file_names: vec!["a.wav".to_string()],
            },
            output: ResolvedIo {
                storage_path: "Y:/out".to_string(),
                file_names: vec!["a.ec3".to_string()],
            },
            misc: ResolvedMisc {
                temp_dir: "Y:/tmp".to_string(),
                clean_temp: true,
            },
            filter: ResolvedFilter::AtmosEc3V1(AtmosEc3V1Filter {
                metering_mode: "1770-4".to_string(),
                dialogue_intelligence: true,
                speech_threshold: 15,
                data_rate: 448,
                timecode_frame_rate: "not_indicated".to_string(),
                start: "first_frame_of_action".to_string(),
                end: "end_of_file".to_string(),
                time_base: "file_position".to_string(),
                prepend_silence_duration: "0.0".to_string(),
                append_silence_duration: "0.0".to_string(),
                line_mode_drc_profile: "film_light".to_string(),
                rf_mode_drc_profile: "film_light".to_string(),
                loro_center_mix_level: "-3".to_string(),
                loro_surround_mix_level: "-3".to_string(),
                ltrt_center_mix_level: "-3".to_string(),
                ltrt_surround_mix_level: "-3".to_string(),
                preferred_downmix_mode: "loro".to_string(),
                surround_trim_5_1: "auto".to_string(),
                surround_trim_7_1: "auto".to_string(),
                height_trim_5_1: "auto".to_string(),
                custom_dialnorm: 0,
                encoding_backend: None,
                encoder_mode: None,
            }),
            run: Default::default(),
        }
    }

    #[test]
    fn renders_when_node_only_if_predicate_true() {
        let node = XmlNode::element(
            "job_config",
            vec![],
            vec![XmlNode::when_node(
                Predicate::ParamSome("encoding_backend".to_string()),
                vec![XmlNode::leaf("encoding_backend", "atmosprocessor")],
            )],
        );

        let xml = render_xml_tree(&node, &sample_job());
        assert!(!xml.contains("encoding_backend"));
    }
}
