mod common;

use std::{fs, path::Path};

use dee_config_gen::load_job_file;
use serde::Deserialize;

use common::{
    EvidenceTier, contract_path_label, evidence_tier_for_param, find_filter_param_path,
    load_xsd_contract, resolve_with_defaults,
};
use dee_config_gen::template::atmos_ec3_v1::params;

#[derive(Debug, Deserialize)]
struct UpstreamPitfalls {
    template_id: String,
    pitfalls: Vec<Pitfall>,
}

#[derive(Debug, Deserialize)]
struct Pitfall {
    id: String,
    source: String,
    reference: String,
    param_key: Option<String>,
    xsd_path: String,
    expected_behavior: String,
}

#[test]
fn upstream_pitfalls_have_valid_xsd_paths() {
    let contract = load_xsd_contract();
    let pitfalls = load_pitfalls();

    assert_eq!(pitfalls.template_id, "atmos_ec3_v1");
    assert!(!pitfalls.pitfalls.is_empty());

    for pitfall in &pitfalls.pitfalls {
        match expected_contract_label(&contract, pitfall) {
            Some(expected_path) => assert_eq!(
                pitfall.xsd_path, expected_path,
                "pitfall id={} source={} reference={} expected_behavior={} has mismatched contract label",
                pitfall.id, pitfall.source, pitfall.reference, pitfall.expected_behavior,
            ),
            None => {
                let matched = contract
                    .paths
                    .iter()
                    .any(|path| path.path == pitfall.xsd_path);
                assert!(
                    matched,
                    "pitfall id={} source={} reference={} xsd_path={} expected_behavior={} is not present in xsd contract",
                    pitfall.id,
                    pitfall.source,
                    pitfall.reference,
                    pitfall.xsd_path,
                    pitfall.expected_behavior,
                );
            }
        }
    }

    assert!(
        find_filter_param_path(&contract, "data_rate").is_some(),
        "expected data_rate path in xsd contract"
    );
}

#[test]
fn upstream_pitfall_behaviors_are_covered() {
    let pitfalls = load_pitfalls();

    let mut bluray_job =
        load_job_file(Path::new("examples/atmos_ec3_single.bluray.yaml")).expect("load bluray");
    bluray_job.filter.data_rate = None;
    let bluray = resolve_with_defaults(bluray_job).expect("resolve bluray");
    let dee_config_gen::ResolvedFilter::AtmosEc3V1(bluray_filter) = &bluray.filter else {
        panic!("expected AtmosEc3V1 filter");
    };
    assert_eq!(bluray_filter.data_rate, 1280);

    let mut streaming_job = load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml"))
        .expect("load streaming");
    streaming_job.filter.data_rate = Some(1024);
    resolve_with_defaults(streaming_job).expect("streaming 1024 should be accepted");

    let runtime_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-atmos-ddp71-rejected")
        .expect("runtime ddp71 pitfall should exist");
    assert!(
        runtime_pitfall
            .expected_behavior
            .contains("template_id 'pcm_ddp_v1'"),
        "runtime ddp71 pitfall should document the migration guidance"
    );

    let surround_ex_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-atmos-pcm-metadata-knobs-unsupported")
        .expect("atmos pcm metadata runtime pitfall should exist");
    assert!(
        surround_ex_pitfall
            .expected_behavior
            .contains("dolby_surround_mode and dolby_surround_ex_mode"),
        "atmos pcm metadata pitfall should document the unsupported property behavior"
    );

    let mut invalid_preferred_downmix_job =
        load_job_file(Path::new("examples/atmos_ec3_single.bluray.yaml")).expect("load bluray");
    invalid_preferred_downmix_job.filter.preferred_downmix_mode = Some("ltrt-pl2".to_string());
    let preferred_downmix_err = resolve_with_defaults(invalid_preferred_downmix_job)
        .expect_err("atmos bluray ltrt-pl2 should now fail locally")
        .to_string();
    assert_eq!(
        preferred_downmix_err,
        "Preferred Downmix mode Pro Logic II is not supported in Blu-ray Mode"
    );

    let preferred_downmix_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| {
            pitfall.id == "dee-runtime-atmos-preferred-downmix-pl2-is-rejected-for-bluray"
        })
        .expect("atmos preferred_downmix runtime pitfall should exist");
    assert!(
        preferred_downmix_pitfall
            .expected_behavior
            .contains("rejects it on bluray"),
        "atmos preferred_downmix pitfall should document the bluray restriction"
    );
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json")
        .expect("read upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse upstream pitfalls fixture")
}

fn expected_contract_label(contract: &common::XsdContract, pitfall: &Pitfall) -> Option<String> {
    let param_key = pitfall.param_key.as_deref().or_else(|| {
        pitfall
            .xsd_path
            .rsplit('/')
            .next()
            .filter(|segment| !segment.is_empty() && !segment.starts_with('<'))
    })?;
    let Some(schema) = params::find_schema(param_key) else {
        if pitfall.source == "local_runtime" && pitfall.xsd_path.starts_with('<') {
            return Some(pitfall.xsd_path.clone());
        }
        return None;
    };
    let tier = evidence_tier_for_param(schema.key, schema.sources);
    match tier {
        EvidenceTier::Official => Some(contract_path_label(contract, schema.key, schema.sources)),
        EvidenceTier::RuntimeVerifiedHidden if pitfall.source == "local_runtime" => {
            Some("<runtime_verified_hidden>".to_string())
        }
        EvidenceTier::FolkloreUnverified if pitfall.source == "local_runtime" => {
            Some("<folklore_unverified>".to_string())
        }
        _ => None,
    }
}
