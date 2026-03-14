mod common;

use std::fs;

use common::{AC4_XSD_CONTRACT_PATH, load_xsd_contract_at};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ac4ParameterMatrix {
    template_id: String,
    profiles: Vec<String>,
    encode_modes: Vec<String>,
    input: Ac4IoType,
    output: Ac4IoType,
    parameters: Vec<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct Ac4IoType {
    r#type: String,
}

#[test]
fn ac4_xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract_at(AC4_XSD_CONTRACT_PATH);

    assert_eq!(contract.meta.template_id, "ac4_v1");
    assert!(!contract.meta.dee_version.trim().is_empty());
    assert!(!contract.meta.exported_at.trim().is_empty());
    assert!(!contract.meta.source_files.is_empty());
    assert!(!contract.elements.is_empty());
    assert!(!contract.attributes.is_empty());
    assert!(!contract.paths.is_empty());
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/ac4"),
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/output/ac4"),
    );
}

#[test]
fn ac4_parameter_matrix_matches_conservative_scope() {
    let content =
        fs::read_to_string("docs/parameter_matrix.ac4_v1.yaml").expect("read ac4 parameter matrix");
    let matrix: Ac4ParameterMatrix =
        serde_yaml::from_str(&content).expect("parse ac4 parameter matrix");

    assert_eq!(matrix.template_id, "ac4_v1");
    assert_eq!(matrix.profiles, vec!["standard".to_string()]);
    assert_eq!(matrix.encode_modes, vec!["ac4".to_string()]);
    assert_eq!(matrix.input.r#type, "ac4");
    assert_eq!(matrix.output.r#type, "ac4");
    assert!(matrix.parameters.is_empty());
}
