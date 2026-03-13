mod common;

use std::{fs, path::Path};

use dee_config_gen::read_job;
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

    let mut dd_ac3_job = read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).expect("load dd");
    dd_ac3_job.filter.data_rate = Some(640);
    resolve_with_defaults(dd_ac3_job).expect("dd 640 should resolve");

    let mut invalid_dd_ac3_job =
        read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).expect("load invalid dd");
    invalid_dd_ac3_job.filter.data_rate = Some(192);
    let dd_ac3_err = resolve_with_defaults(invalid_dd_ac3_job)
        .expect_err("dd 192 should fail")
        .to_string();
    assert!(dd_ac3_err.contains("invalid data_rate '192' for mode 'dd'"));

    let mut ddp_ec3_job =
        read_job(Path::new("examples/pcm_ddp_single.ddp.yaml")).expect("load ddp");
    ddp_ec3_job.filter.data_rate = Some(1024);
    resolve_with_defaults(ddp_ec3_job).expect("ddp 1024 should resolve");

    let mut invalid_ddp_ec3_job =
        read_job(Path::new("examples/pcm_ddp_single.ddp.yaml")).expect("load invalid ddp");
    invalid_ddp_ec3_job.filter.data_rate = Some(191);
    let ddp_ec3_err = resolve_with_defaults(invalid_ddp_ec3_job)
        .expect_err("ddp 191 should fail")
        .to_string();
    assert!(ddp_ec3_err.contains("invalid data_rate '191' for mode 'ddp'"));

    let mut ddp71_job =
        read_job(Path::new("examples/pcm_ddp_single.ddp71.yaml")).expect("load ddp71");
    ddp71_job.filter.data_rate = Some(1024);
    resolve_with_defaults(ddp71_job).expect("ddp71 1024 should resolve");

    let mut invalid_ddp71_job =
        read_job(Path::new("examples/pcm_ddp_single.ddp71.yaml")).expect("load invalid ddp71");
    invalid_ddp71_job.filter.data_rate = Some(1280);
    let err = resolve_with_defaults(invalid_ddp71_job)
        .expect_err("ddp71 1280 should fail")
        .to_string();
    assert!(err.contains("invalid data_rate '1280' for mode 'ddp71'"));

    let mut invalid_override_job =
        read_job(Path::new("examples/pcm_ddp_single.bluray.yaml")).expect("load bluray");
    invalid_override_job.filter.encoding_backend = Some("atmosprocessor".to_string());
    let override_err = resolve_with_defaults(invalid_override_job)
        .expect_err("encoding_backend should fail on pcm_ddp_v1")
        .to_string();
    assert_eq!(
        override_err,
        "parameter 'encoding_backend' is not supported by template_id 'pcm_ddp_v1'"
    );

    let mut invalid_dd_ac3_downmix =
        read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).expect("load dd");
    invalid_dd_ac3_downmix.input.storage_path = "testfiles".to_string();
    invalid_dd_ac3_downmix.input.file_names = vec!["input_8ch.wav".to_string()];
    invalid_dd_ac3_downmix.filter.downmix_config = Some("off".to_string());
    let dd_ac3_downmix_err = resolve_with_defaults(invalid_dd_ac3_downmix)
        .expect_err("dd off should fail")
        .to_string();
    assert_eq!(
        dd_ac3_downmix_err,
        "dd mode with 8-channel input requires downmix_config=5.1"
    );

    let mut invalid_ddp_ec3_downmix =
        read_job(Path::new("examples/pcm_ddp_single.ddp.yaml")).expect("load ddp");
    invalid_ddp_ec3_downmix.input.storage_path = "testfiles".to_string();
    invalid_ddp_ec3_downmix.input.file_names = vec!["input_8ch.wav".to_string()];
    invalid_ddp_ec3_downmix.filter.downmix_config = Some("off".to_string());
    let ddp_ec3_downmix_err = resolve_with_defaults(invalid_ddp_ec3_downmix)
        .expect_err("ddp off should fail")
        .to_string();
    assert_eq!(
        ddp_ec3_downmix_err,
        "ddp mode with 8-channel input requires downmix_config=5.1"
    );

    let mut valid_dd_ac3_downmix =
        read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).expect("load dd 6ch");
    valid_dd_ac3_downmix.filter.downmix_config = Some("off".to_string());
    resolve_with_defaults(valid_dd_ac3_downmix).expect("dd + 6ch + off should resolve");

    let mut valid_ddp_ec3_downmix =
        read_job(Path::new("examples/pcm_ddp_single.ddp.yaml")).expect("load ddp 6ch");
    valid_ddp_ec3_downmix.filter.downmix_config = Some("off".to_string());
    resolve_with_defaults(valid_ddp_ec3_downmix).expect("ddp + 6ch + off should resolve");

    let mut invalid_metering_job =
        read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).expect("load dd");
    invalid_metering_job.filter.metering_mode = Some("1770-4".to_string());
    let metering_err = resolve_with_defaults(invalid_metering_job)
        .expect_err("1770-4 should fail on pcm_ddp_v1")
        .to_string();
    assert!(metering_err.contains("invalid value '1770-4' for metering_mode"));

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

    let dd_output_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-dd-output-is-ac3")
        .expect("dd output pitfall should exist");
    assert!(
        dd_output_pitfall
            .expected_behavior
            .contains("AC-3 output node"),
        "dd output pitfall should document ac3 output requirement"
    );

    let metering_mode_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-metering-mode-1770-4-is-rejected")
        .expect("metering_mode runtime pitfall should exist");
    assert!(
        metering_mode_pitfall
            .expected_behavior
            .contains("rejects 1770-4"),
        "metering_mode runtime pitfall should document the 1770-4 rejection"
    );

    let downmix_runtime_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-downmix-config-is-input-sensitive")
        .expect("downmix_config runtime pitfall should exist");
    assert!(
        downmix_runtime_pitfall
            .expected_behavior
            .contains("6ch input"),
        "downmix_config runtime pitfall should document 6ch-specific behavior"
    );

    let starting_timecode_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-starting-timecode-is-dd-or-bluray-only")
        .expect("starting_timecode runtime pitfall should exist");
    assert!(
        starting_timecode_pitfall
            .expected_behavior
            .contains("rejected on ddp and ddp71"),
        "starting_timecode runtime pitfall should document the dd/ddp71 split"
    );

    let frame_rate_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-frame-rate-is-more-permissive-than-schema")
        .expect("frame_rate runtime pitfall should exist");
    assert!(
        frame_rate_pitfall
            .expected_behavior
            .contains("both local schema and DEE 5.2.1 currently accept arbitrary strings"),
        "frame_rate runtime pitfall should document the cross-mode schema/runtime compatibility gap"
    );

    let mut permissive_frame_rate_job =
        read_job(Path::new("examples/pcm_ddp_single.ddp.yaml")).expect("load ddp");
    permissive_frame_rate_job.filter.frame_rate = Some("bogus".to_string());
    resolve_with_defaults(permissive_frame_rate_job)
        .expect("local schema now intentionally allows permissive frame_rate values");

    let pl2_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-ltrt-pl2-is-rejected-for-dd-and-bluray")
        .expect("ltrt-pl2 runtime pitfall should exist");
    assert!(
        pl2_pitfall
            .expected_behavior
            .contains("accepted on ddp and ddp71"),
        "ltrt-pl2 runtime pitfall should document the dd/bluray restriction"
    );

    let bitstream_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-bitstream-commentary-is-cross-mode")
        .expect("bitstream commentary runtime pitfall should exist");
    assert!(
        bitstream_pitfall
            .expected_behavior
            .contains("accepted across dd, ddp, ddp71, and bluray"),
        "bitstream commentary pitfall should document cross-mode acceptance"
    );

    let metadata_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-metadata-knobs-are-runtime-verified")
        .expect("metadata runtime pitfall should exist");
    assert!(
        metadata_pitfall
            .expected_behavior
            .contains("bluray normalizes dolby_surround_ex_mode"),
        "metadata pitfall should document bluray normalization"
    );

    let allow_hybrid_pitfall = pitfalls
        .pitfalls
        .iter()
        .find(|pitfall| pitfall.id == "dee-runtime-pcm-allow-hybrid-downmix-is-cross-mode")
        .expect("allow_hybrid runtime pitfall should exist");
    assert!(
        allow_hybrid_pitfall
            .expected_behavior
            .contains("accepted across dd, ddp, ddp71, and bluray"),
        "allow_hybrid pitfall should document cross-mode acceptance"
    );
}

fn load_pitfalls() -> UpstreamPitfalls {
    let content = fs::read_to_string("tests/fixtures/upstream_pitfalls.pcm_ddp_v1.json")
        .expect("read upstream pitfalls fixture");
    serde_json::from_str(&content).expect("parse upstream pitfalls fixture")
}
