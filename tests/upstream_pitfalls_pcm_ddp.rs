mod common;

use std::{fs, path::Path};

use dee_config_gen::load_job_file;
use serde::Deserialize;

use common::{find_filter_param_path_with_prefix, load_xsd_contract_at, resolve_with_defaults};

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
fn pcm_ddp_pitfalls_have_valid_contract_paths() {
    let contract = load_xsd_contract_at("tests/fixtures/xsd/contract.pcm_ddp_v1.json");
    let pitfalls = load_pitfalls();

    assert_eq!(pitfalls.template_id, "pcm_ddp_v1");
    assert!(!pitfalls.pitfalls.is_empty());

    for pitfall in &pitfalls.pitfalls {
        if let Some(param_key) = pitfall.param_key.as_deref() {
            let expected = find_filter_param_path_with_prefix(
                &contract,
                param_key,
                "/job_config/filter/audio/pcm_to_ddp/",
            )
            .unwrap_or_else(|| panic!("missing xsd path for pcm_ddp param_key={param_key}"));
            assert_eq!(
                pitfall.xsd_path, expected,
                "pitfall id={} source={} reference={} expected_behavior={} has mismatched contract path",
                pitfall.id, pitfall.source, pitfall.reference, pitfall.expected_behavior,
            );
        }
    }
}

#[test]
fn pcm_ddp_pitfall_behaviors_are_covered() {
    let pitfalls = load_pitfalls();

    let mut ddp71_job =
        load_job_file(Path::new("examples/pcm_ddp_single.ddp71.yaml")).expect("load ddp71");
    ddp71_job.filter.data_rate = Some(1024);
    resolve_with_defaults(ddp71_job).expect("ddp71 1024 should resolve");

    let mut invalid_ddp71_job =
        load_job_file(Path::new("examples/pcm_ddp_single.ddp71.yaml")).expect("load invalid ddp71");
    invalid_ddp71_job.filter.data_rate = Some(1280);
    let err = resolve_with_defaults(invalid_ddp71_job)
        .expect_err("ddp71 1280 should fail")
        .to_string();
    assert!(err.contains("invalid data_rate '1280' for mode 'ddp71'"));

    let mut invalid_override_job =
        load_job_file(Path::new("examples/pcm_ddp_single.bluray.yaml")).expect("load bluray");
    invalid_override_job.filter.encoding_backend = Some("atmosprocessor".to_string());
    let override_err = resolve_with_defaults(invalid_override_job)
        .expect_err("encoding_backend should fail on pcm_ddp_v1")
        .to_string();
    assert_eq!(
        override_err,
        "parameter 'encoding_backend' is not supported by template_id 'pcm_ddp_v1'"
    );

    let six_channel_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-6ch-expands-to-7-1")
        .expect("6ch runtime pitfall should exist");
    assert!(
        six_channel_pitfall
            .expected_behavior
            .contains("Encoding 5.1 channel input in 7.1 channel mode."),
        "6ch runtime pitfall should document 5.1->7.1 behavior"
    );
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json")
        .expect("read upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse upstream pitfalls fixture")
}
