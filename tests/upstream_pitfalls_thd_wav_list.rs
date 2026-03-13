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
fn thd_wav_list_pitfalls_have_expected_labels() {
    let pitfalls = load_pitfalls();
    assert_eq!(pitfalls.template_id, "thd_wav_list_v1");
    assert_eq!(pitfalls.pitfalls.len(), 2);

    let mono = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-wav-list-mono-unsupported")
        .expect("mono wav_list pitfall should exist");
    assert_eq!(mono.source, "local_runtime");
    assert_eq!(mono.param_key.as_deref(), Some("channel_configuration"));
    assert_eq!(mono.xsd_path, "<conservative_gap>");
    assert!(
        mono.reference
            .contains("thd_wav_list_mono_is_rejected_by_runtime")
    );

    let dash = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-wav-list-dash-placeholder-unsupported")
        .expect("dash placeholder pitfall should exist");
    assert_eq!(dash.source, "local_runtime");
    assert!(dash.param_key.is_none());
    assert_eq!(dash.xsd_path, "<conservative_gap>");
    assert!(
        dash.reference
            .contains("thd_wav_list_dash_placeholders_are_rejected_by_runtime")
    );
}

#[test]
fn thd_wav_list_pitfalls_document_runtime_narrowing() {
    let pitfalls = load_pitfalls();

    let mono = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-wav-list-mono-unsupported")
        .expect("mono wav_list pitfall should exist");
    assert!(mono.expected_behavior.contains("mono"));
    assert!(mono.expected_behavior.contains("file_name_C"));
    assert!(mono.expected_behavior.contains("stereo/5.1/7.1"));

    let dash = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-thd-wav-list-dash-placeholder-unsupported")
        .expect("dash placeholder pitfall should exist");
    assert!(dash.expected_behavior.contains("literal filename"));
    assert!(dash.expected_behavior.contains("CLI behavior"));
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.thd_wav_list_v1.json")
        .expect("read thd_wav_list upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse thd_wav_list upstream pitfalls fixture")
}
