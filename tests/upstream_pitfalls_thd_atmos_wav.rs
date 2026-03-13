use std::fs;

use serde::Deserialize;

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
fn thd_atmos_wav_pitfalls_have_expected_labels() {
    let pitfalls = load_pitfalls();
    assert_eq!(pitfalls.template_id, "thd_atmos_wav_v1");
    assert_eq!(pitfalls.pitfalls.len(), 2);

    let mono = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-atmos-wav-mono-unsupported")
        .expect("mono topology pitfall should exist");
    assert_eq!(mono.source, "local_runtime");
    assert_eq!(mono.xsd_path, "<conservative_gap>");
    assert!(
        mono.reference
            .contains("thd_atmos_wav_mono_topology_matches_runtime")
    );
    assert!(mono.param_key.is_none());

    let offset = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-atmos-wav-offset-default-start-rejected")
        .expect("offset/start pitfall should exist");
    assert_eq!(offset.source, "local_runtime");
    assert_eq!(offset.xsd_path, "<conservative_gap>");
    assert_eq!(offset.param_key.as_deref(), Some("offset"));
    assert!(
        offset
            .reference
            .contains("thd_atmos_wav_offset_with_default_start_is_rejected")
    );
}

#[test]
fn thd_atmos_wav_pitfall_behavior_is_documented() {
    let pitfalls = load_pitfalls();

    let mono = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-atmos-wav-mono-unsupported")
        .expect("mono topology pitfall should exist");
    assert!(
        mono.expected_behavior
            .contains("Missing media info property: SamplingCount")
    );
    assert!(mono.expected_behavior.contains("stereo, 5.1, and 7.1"));

    let offset = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-atmos-wav-offset-default-start-rejected")
        .expect("offset/start pitfall should exist");
    assert!(offset.expected_behavior.contains("first_frame_of_action"));
    assert!(offset.expected_behavior.contains("explicit start value"));
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.thd_atmos_wav_v1.json")
        .expect("read thd_atmos_wav upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse thd_atmos_wav upstream pitfalls fixture")
}
