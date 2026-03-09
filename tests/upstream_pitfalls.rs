mod common;

use std::{fs, path::Path};

use dee_config_gen::{config::EncodeMode, load_job_file};
use serde::Deserialize;

use common::{find_filter_param_path, load_xsd_contract, resolve_with_defaults};

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

    assert!(
        find_filter_param_path(&contract, "data_rate").is_some(),
        "expected data_rate path in xsd contract"
    );
}

#[test]
fn upstream_pitfall_behaviors_are_covered() {
    let mut bluray_job =
        load_job_file(Path::new("examples/atmos_ec3_single.bluray.yaml")).expect("load bluray");
    bluray_job.filter.data_rate = None;
    let bluray = resolve_with_defaults(bluray_job).expect("resolve bluray");
    let dee_config_gen::ResolvedFilter::AtmosEc3V1(bluray_filter) = &bluray.filter;
    assert_eq!(bluray_filter.data_rate, 1280);

    let mut streaming_job = load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml"))
        .expect("load streaming");
    streaming_job.filter.data_rate = Some(1024);
    resolve_with_defaults(streaming_job).expect("streaming 1024 should be accepted");

    let mut ddp71_job =
        load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml")).expect("load ddp71");
    ddp71_job.encode_mode = EncodeMode::Ddp71;
    ddp71_job.filter.encoder_mode = Some("bluray".to_string());
    let err = resolve_with_defaults(ddp71_job)
        .expect_err("ddp71 with encoder_mode=bluray should fail")
        .to_string();
    assert!(err.contains("ddp71 mode requires encoder_mode=ddp71"));
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.atmos_ec3_v1.json")
        .expect("read upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse upstream pitfalls fixture")
}
