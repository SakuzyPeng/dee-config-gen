mod common;

use dee_config_gen::template::atmos_ec3_v1::params::PARAM_SCHEMAS;

use common::{EvidenceTier, evidence_tier_for_param, find_filter_param_path, load_xsd_contract};

#[test]
fn xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract();

    assert_eq!(contract.meta.template_id, "atmos_ec3_v1");
    assert!(!contract.meta.dee_version.trim().is_empty());
    assert!(!contract.meta.exported_at.trim().is_empty());
    assert!(!contract.meta.source_files.is_empty());

    assert!(!contract.elements.is_empty());
    assert!(!contract.attributes.is_empty());
    assert!(!contract.simple_types.is_empty());
    assert!(!contract.paths.is_empty());
}

#[test]
fn xsd_contract_covers_dolby_official_filter_params() {
    let contract = load_xsd_contract();

    for schema in PARAM_SCHEMAS {
        if evidence_tier_for_param(schema.key, schema.sources) == EvidenceTier::Official {
            let path = find_filter_param_path(&contract, schema.key);
            assert!(
                path.is_some(),
                "missing xsd path for official param_key={} in contract.paths",
                schema.key
            );
        }
    }
}
