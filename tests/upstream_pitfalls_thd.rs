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
fn thd_pitfalls_have_expected_labels() {
    let pitfalls = load_pitfalls();
    assert_eq!(pitfalls.template_id, "thd_v1");
    assert_eq!(pitfalls.pitfalls.len(), 2);

    let workflow = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-mp4-output-unsupported")
        .expect("workflow mp4 pitfall should exist");
    assert_eq!(workflow.source, "local_runtime");
    assert_eq!(workflow.xsd_path, "<unsupported_or_hidden>");
    assert!(
        workflow
            .reference
            .contains("thd_mp4_feasibility_lane_records_runtime_result")
    );
    assert!(workflow.param_key.is_none());

    let native = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "native-mp4muxer-thd-mlp-unsupported")
        .expect("native mp4muxer pitfall should exist");
    assert_eq!(native.source, "local_runtime");
    assert_eq!(native.xsd_path, "<unsupported_or_hidden>");
    assert!(native.reference.contains("thd_native_mp4muxer_rejects_mlp"));
    assert!(native.param_key.is_none());
}

#[test]
fn thd_pitfall_behavior_is_documented() {
    let pitfalls = load_pitfalls();
    let workflow = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-mp4-output-unsupported")
        .expect("thd mp4 runtime pitfall should exist");

    assert!(workflow.expected_behavior.contains("rejects output/mp4"));
    assert!(workflow.expected_behavior.contains("audio.mlp"));
    assert!(
        workflow
            .expected_behavior
            .contains("Keep mp4 out of the public thd_v1 interface")
    );

    let native = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "native-mp4muxer-thd-mlp-unsupported")
        .expect("native mp4muxer pitfall should exist");
    assert!(native.expected_behavior.contains("extension not supported"));
    assert!(
        native
            .expected_behavior
            .contains("Keep mp4 out of the public thd_v1 interface")
    );
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.thd_v1.json")
        .expect("read thd upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse thd upstream pitfalls fixture")
}
