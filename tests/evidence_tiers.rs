mod common;

use dee_config_gen::spec::find_param_schema;

use common::{
    EvidenceTier, FOLKLORE_UNVERIFIED_PARAMS, RUNTIME_VERIFIED_HIDDEN_PARAMS, contract_path_label,
    evidence_tier_for_param, load_xsd_contract,
};

const RUNTIME_REGRESSION_SOURCE: &str = include_str!("dee_runtime_hidden_params.rs");

#[test]
fn runtime_verified_hidden_params_are_covered_by_runtime_regressions() {
    let contract = load_xsd_contract();

    for key in RUNTIME_VERIFIED_HIDDEN_PARAMS {
        let schema = find_param_schema("atmos_ec3_v1", key)
            .unwrap()
            .unwrap_or_else(|| panic!("missing schema for {key}"));
        assert_eq!(
            evidence_tier_for_param(schema.key, schema.sources),
            EvidenceTier::RuntimeVerifiedHidden
        );
        assert!(
            RUNTIME_REGRESSION_SOURCE.contains(key),
            "runtime regression suite must cover hidden param {key}"
        );

        let label = contract_path_label(&contract, schema.key, schema.sources);
        assert!(
            label == "<runtime_verified_hidden>" || label.starts_with("/job_config/"),
            "runtime verified hidden param {key} should use a non-official placeholder or a real xsd path, got {label}"
        );
    }
}

#[test]
fn folklore_params_are_not_treated_as_official_contract() {
    let contract = load_xsd_contract();

    for key in FOLKLORE_UNVERIFIED_PARAMS {
        let schema = find_param_schema("atmos_ec3_v1", key)
            .unwrap()
            .unwrap_or_else(|| panic!("missing schema for {key}"));
        assert_eq!(
            evidence_tier_for_param(schema.key, schema.sources),
            EvidenceTier::FolkloreUnverified
        );
        assert_eq!(
            contract_path_label(&contract, schema.key, schema.sources),
            "<folklore_unverified>"
        );
    }
}
