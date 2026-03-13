use anyhow::Result;
use clap::ValueEnum;
use serde_json::Value;

use crate::{resolve::ResolvedJob, template::TemplateRegistry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum RenderFormat {
    Xml,
    Json,
}

impl RenderFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Xml => "xml",
            Self::Json => "json",
        }
    }

    pub fn cli_flag(self) -> &'static str {
        match self {
            Self::Xml => "--xml",
            Self::Json => "--json",
        }
    }

    pub fn short_cli_flag(self) -> &'static str {
        match self {
            Self::Xml => "-x",
            Self::Json => "-j",
        }
    }
}

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

pub fn render_config(job: &ResolvedJob, format: RenderFormat) -> Result<String> {
    let template = TemplateRegistry::get(&job.template_id)?;
    match format {
        RenderFormat::Xml => {
            let tree = template.xml_structure(job);
            Ok(render_xml_tree(&tree, job))
        }
        RenderFormat::Json => {
            let value = template.json_structure(job)?;
            Ok(render_json_value(&value)?)
        }
    }
}

pub fn render_xml(job: &ResolvedJob) -> String {
    render_config(job, RenderFormat::Xml).unwrap_or_else(|err| {
        panic!(
            "template_id '{}' should be validated before XML rendering, but rendering failed: {err}",
            job.template_id
        )
    })
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
                format!("\"{f}\"")
            } else {
                f.clone()
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

fn render_json_value(value: &Value) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use crate::test_support::sample_resolved_job;

    use super::{Predicate, RenderFormat, XmlNode, render_config, render_xml_tree};

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

        let xml = render_xml_tree(&node, &sample_resolved_job());
        assert!(!xml.contains("encoding_backend"));
    }

    #[test]
    fn render_config_xml_matches_render_xml_tree() {
        let job = sample_resolved_job();
        let rendered = render_config(&job, RenderFormat::Xml).expect("render xml");
        assert!(rendered.starts_with("<?xml version=\"1.0\"?>"));
    }

    #[test]
    fn json_render_is_not_supported_for_non_json_templates_by_default() {
        let mut job = sample_resolved_job();
        job.template_id = "definitely_unknown_template".to_string();
        let err = render_config(&job, RenderFormat::Json).expect_err("json unsupported");
        assert!(err.to_string().contains("unsupported template_id"));
    }
}
