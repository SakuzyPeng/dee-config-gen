mod common;

use std::fs;

use common::{
    AC4_IMS_ATMOS_XSD_CONTRACT_PATH, AC4_IMS_PCM_XSD_CONTRACT_PATH, load_xsd_contract_at,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Ac4ParameterMatrix {
    template_id: String,
    profiles: Vec<String>,
    encode_modes: Vec<String>,
    input: Ac4IoType,
    output: Ac4IoType,
    parameters: Vec<Ac4Parameter>,
}

#[derive(Debug, Deserialize)]
struct Ac4IoType {
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct Ac4Parameter {
    key: String,
}

#[test]
fn ac4_ims_atmos_xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract_at(AC4_IMS_ATMOS_XSD_CONTRACT_PATH);

    assert_eq!(contract.meta.template_id, "ac4_ims_atmos_v1");
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/atmos_mezz")
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/filter/audio/encode_to_ims_ac4")
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/output/ac4")
    );
}

#[test]
fn ac4_ims_pcm_xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract_at(AC4_IMS_PCM_XSD_CONTRACT_PATH);

    assert_eq!(contract.meta.template_id, "ac4_ims_pcm_v1");
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/wav")
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/wav_list")
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/filter/audio/encode_to_ims_ac4")
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/output/ac4")
    );
}

#[test]
fn ac4_ims_parameter_matrices_match_official_scope() {
    let atmos_content = fs::read_to_string("docs/parameter_matrix.ac4_ims_atmos_v1.yaml")
        .expect("read atmos ac4 parameter matrix");
    let pcm_content = fs::read_to_string("docs/parameter_matrix.ac4_ims_pcm_v1.yaml")
        .expect("read pcm ac4 parameter matrix");

    let atmos: Ac4ParameterMatrix =
        serde_yaml::from_str(&atmos_content).expect("parse atmos ac4 parameter matrix");
    let pcm: Ac4ParameterMatrix =
        serde_yaml::from_str(&pcm_content).expect("parse pcm ac4 parameter matrix");

    assert_eq!(atmos.template_id, "ac4_ims_atmos_v1");
    assert_eq!(pcm.template_id, "ac4_ims_pcm_v1");
    assert_eq!(
        atmos.profiles,
        vec!["standard".to_string(), "music".to_string()]
    );
    assert_eq!(
        pcm.profiles,
        vec!["standard".to_string(), "music".to_string()]
    );
    assert_eq!(atmos.encode_modes, vec!["ac4".to_string()]);
    assert_eq!(pcm.encode_modes, vec!["ac4".to_string()]);
    assert_eq!(atmos.input.r#type, "atmos_mezz");
    assert_eq!(pcm.input.r#type, "wav or wav_list");
    assert_eq!(atmos.output.r#type, "ac4");
    assert_eq!(pcm.output.r#type, "ac4");
    assert!(
        atmos
            .parameters
            .iter()
            .any(|param| param.key == "data_rate")
    );
    assert!(
        pcm.parameters
            .iter()
            .any(|param| param.key == "encoding_profile")
    );
}
