mod common;

use dee_config_gen::template::pcm_ddp_v1::params::PARAM_SCHEMAS;

use common::{
    EvidenceTier, PCM_DDP_XSD_CONTRACT_PATH, evidence_tier_for_param,
    find_filter_param_path_with_prefix, load_xsd_contract_at,
};

#[test]
fn pcm_ddp_xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract_at(PCM_DDP_XSD_CONTRACT_PATH);

    assert_eq!(contract.meta.template_id, "pcm_ddp_v1");
    assert!(!contract.meta.dee_version.trim().is_empty());
    assert!(!contract.meta.exported_at.trim().is_empty());
    assert!(!contract.meta.source_files.is_empty());

    assert!(!contract.elements.is_empty());
    assert!(!contract.attributes.is_empty());
    assert!(!contract.simple_types.is_empty());
    assert!(!contract.paths.is_empty());
}

#[test]
fn pcm_ddp_xsd_contract_covers_dolby_official_filter_params() {
    let contract = load_xsd_contract_at(PCM_DDP_XSD_CONTRACT_PATH);

    for schema in PARAM_SCHEMAS {
        if evidence_tier_for_param(schema.key, schema.sources) == EvidenceTier::Official {
            let path = find_filter_param_path_with_prefix(
                &contract,
                schema.key,
                "/job_config/filter/audio/pcm_to_ddp/",
            );
            assert!(
                path.is_some(),
                "missing xsd path for official pcm param_key={} in contract.paths",
                schema.key
            );
        }
    }
}
