mod common;

use dee_config_gen::spec::param_schemas;

use common::{
    EvidenceTier, THD_WAV_XSD_CONTRACT_PATH, evidence_tier_for_param,
    find_filter_param_path_with_prefix, load_xsd_contract_at,
};

fn thd_wav_filter_param_path<'a>(contract: &'a common::XsdContract, key: &str) -> Option<&'a str> {
    match key {
        "input_timecode_frame_rate" => Some("/job_config/input/audio/wav/timecode_frame_rate"),
        "offset" => Some("/job_config/input/audio/wav/offset"),
        "ffoa" => Some("/job_config/input/audio/wav/ffoa"),
        "atmos_presentation_drc_profile" => {
            Some("/job_config/filter/audio/encode_to_dthd/atmos_presentation/drc_profile")
        }
        "spatial_clusters" => {
            Some("/job_config/filter/audio/encode_to_dthd/atmos_presentation/spatial_clusters")
        }
        "starting_timecode" => {
            Some("/job_config/filter/audio/encode_to_dthd/embedded_timecodes/starting_timecode")
        }
        "frame_rate" => {
            Some("/job_config/filter/audio/encode_to_dthd/embedded_timecodes/frame_rate")
        }
        "legacy_authoring_compatibility" => Some(
            "/job_config/filter/audio/encode_to_dthd/atmos_presentation/legacy_authoring_compatibility",
        ),
        "presentation_8ch_drc_profile" => {
            Some("/job_config/filter/audio/encode_to_dthd/presentation_8ch/drc_profile")
        }
        "presentation_6ch_drc_profile" => {
            Some("/job_config/filter/audio/encode_to_dthd/presentation_6ch/drc_profile")
        }
        "presentation_2ch_drc_profile" => {
            Some("/job_config/filter/audio/encode_to_dthd/presentation_2ch/drc_profile")
        }
        "optimize_data_rate" => Some("/job_config/filter/audio/encode_to_dthd/optimize_data_rate"),
        _ => find_filter_param_path_with_prefix(
            contract,
            key,
            "/job_config/filter/audio/encode_to_dthd/",
        ),
    }
}

#[test]
fn thd_wav_xsd_contract_json_has_expected_sections() {
    let contract = load_xsd_contract_at(THD_WAV_XSD_CONTRACT_PATH);

    assert_eq!(contract.meta.template_id, "thd_wav_v1");
    assert!(!contract.meta.dee_version.trim().is_empty());
    assert!(!contract.meta.exported_at.trim().is_empty());
    assert!(!contract.meta.source_files.is_empty());
    assert!(!contract.elements.is_empty());
    assert!(!contract.attributes.is_empty());
    assert!(!contract.simple_types.is_empty());
    assert!(!contract.paths.is_empty());
}

#[test]
fn thd_wav_xsd_contract_covers_dolby_official_filter_params() {
    let contract = load_xsd_contract_at(THD_WAV_XSD_CONTRACT_PATH);

    for schema in param_schemas("thd_wav_v1").unwrap() {
        if evidence_tier_for_param(schema.key, schema.sources) == EvidenceTier::Official {
            let path = thd_wav_filter_param_path(&contract, schema.key);
            assert!(
                path.is_some(),
                "missing xsd path for official thd_wav param_key={} in contract.paths",
                schema.key
            );
        }
    }

    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/wav/timecode_frame_rate"),
        "missing xsd path for wav input timecode_frame_rate"
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/wav/offset"),
        "missing xsd path for wav input offset"
    );
    assert!(
        contract
            .paths
            .iter()
            .any(|entry| entry.path == "/job_config/input/audio/wav/ffoa"),
        "missing xsd path for wav input ffoa"
    );
}
