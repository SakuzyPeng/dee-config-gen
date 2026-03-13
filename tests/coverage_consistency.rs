use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use dee_config_gen::load_job_file;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CoverageMatrix {
    templates: Vec<CoverageTemplate>,
}

#[derive(Debug, Deserialize)]
struct CoverageTemplate {
    template: String,
    parameters: Vec<CoverageParameter>,
}

#[derive(Debug, Deserialize)]
struct CoverageParameter {
    param: String,
    modes: BTreeMap<String, CoverageModeState>,
}

#[derive(Debug, Deserialize)]
struct CoverageModeState {
    schema: String,
    xsd_contract: String,
    runtime: String,
    knowledge_pitfall: String,
}

#[derive(Debug, Deserialize)]
struct UpstreamKnowledge {
    templates: BTreeMap<String, serde_json::Value>,
}

#[test]
fn coverage_truth_sources_are_complete_for_registered_templates() {
    let registered = registered_templates();
    let coverage = load_coverage_matrix();
    let knowledge = load_upstream_knowledge();

    let coverage_templates = coverage
        .templates
        .iter()
        .map(|template| template.template.clone())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        coverage_templates, registered,
        "coverage_matrix.full.yaml must describe exactly the registered templates"
    );

    let knowledge_templates = knowledge.templates.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        knowledge_templates, registered,
        "upstream_knowledge.json must contain exactly the registered templates"
    );

    let parameter_matrix_templates = fs::read_dir("docs")
        .expect("read docs dir")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| {
            name.strip_prefix("parameter_matrix.")
                .and_then(|rest| rest.strip_suffix(".yaml"))
                .map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        parameter_matrix_templates, registered,
        "docs/parameter_matrix.<template>.yaml must exist for every registered template"
    );

    let xsd_contract_templates = fs::read_dir("tests/fixtures/xsd")
        .expect("read xsd fixture dir")
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| {
            name.strip_prefix("contract.")
                .and_then(|rest| rest.strip_suffix(".json"))
                .map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        xsd_contract_templates, registered,
        "tests/fixtures/xsd/contract.<template>.json must exist for every registered template"
    );

    let example_templates = fs::read_dir("examples")
        .expect("read examples dir")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some())
        .map(|path| {
            load_job_file(&path)
                .unwrap_or_else(|err| panic!("failed to load {}: {err}", path.display()))
        })
        .filter_map(|job| job.template_id)
        .collect::<BTreeSet<_>>();
    assert_eq!(
        example_templates, registered,
        "examples/ must contain at least one job spec for every registered template"
    );
}

#[test]
fn unsupported_or_hidden_entries_have_documented_evidence() {
    let coverage = load_coverage_matrix();
    let knowledge = load_upstream_knowledge();

    for template in &coverage.templates {
        let Some(knowledge_entry) = knowledge.templates.get(&template.template) else {
            panic!("missing knowledge entry for {}", template.template);
        };
        let knowledge_blob = knowledge_entry.to_string();

        for parameter in &template.parameters {
            let has_unsupported = parameter.modes.values().any(|mode| {
                mode.schema == "unsupported_or_hidden"
                    || mode.xsd_contract == "unsupported_or_hidden"
                    || mode.runtime == "unsupported_or_hidden"
                    || mode.knowledge_pitfall == "unsupported_or_hidden"
            });

            if has_unsupported {
                assert!(
                    knowledge_blob.contains(&parameter.param),
                    "template '{}' marks '{}' as unsupported_or_hidden but upstream_knowledge.json does not mention it",
                    template.template,
                    parameter.param
                );
            }
        }
    }
}

fn registered_templates() -> BTreeSet<String> {
    fs::read_to_string("src/template/mod.rs")
        .expect("read src/template/mod.rs")
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with("pub mod ") {
                return None;
            }
            let name = line
                .trim_start_matches("pub mod ")
                .trim_end_matches(';')
                .trim();
            if name == "thd_mixed" {
                None
            } else {
                Some(name.to_string())
            }
        })
        .collect()
}

fn load_coverage_matrix() -> CoverageMatrix {
    let content = fs::read_to_string(Path::new("docs/coverage_matrix.full.yaml"))
        .expect("read docs/coverage_matrix.full.yaml");
    serde_yaml::from_str(&content).expect("parse docs/coverage_matrix.full.yaml")
}

fn load_upstream_knowledge() -> UpstreamKnowledge {
    let content = fs::read_to_string(Path::new("tests/fixtures/upstream_knowledge.json"))
        .expect("read tests/fixtures/upstream_knowledge.json");
    serde_json::from_str(&content).expect("parse tests/fixtures/upstream_knowledge.json")
}
