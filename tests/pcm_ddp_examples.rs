mod common;

use std::path::Path;

use common::create_test_wav_inputs;
use dee_config_gen::{ResolveOptions, read_job, render_xml, resolve_job, spec::FilterOverrides};

fn load_pcm_example(path: &str) -> (tempfile::TempDir, dee_config_gen::JobSpec) {
    let mut spec = read_job(Path::new(path)).unwrap();
    let (temp, storage_path, file_names) = create_test_wav_inputs(6, 16, 1);
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;
    (temp, spec)
}

fn resolve_from_example(path: &str) -> dee_config_gen::ResolvedJob {
    let (_temp, spec) = load_pcm_example(path);
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap()
}

#[test]
fn renders_pcm_dd_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.dd.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>dd</encoder_mode>"));
    assert!(xml.contains("<data_rate>640</data_rate>"));
    assert!(xml.contains("<downmix_config>5.1</downmix_config>"));
    assert!(xml.contains("<ac3 version=\"1\">"));
}

#[test]
fn renders_pcm_ddp_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.ddp.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>ddp</encoder_mode>"));
    assert!(xml.contains("<data_rate>1024</data_rate>"));
    assert!(xml.contains("<downmix_config>5.1</downmix_config>"));
    assert!(xml.contains("<ec3 version=\"1\">"));
}

#[test]
fn renders_pcm_ddp71_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.ddp71.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>ddp71</encoder_mode>"));
    assert!(xml.contains("<data_rate>1024</data_rate>"));
}

#[test]
fn renders_pcm_bluray_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.bluray.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>bluray</encoder_mode>"));
    assert!(xml.contains("<data_rate>1664</data_rate>"));
    assert!(xml.contains("<downmix_config>off</downmix_config>"));
}

#[test]
fn rejects_atmos_only_override_on_pcm_template() {
    let (_temp, mut spec) = load_pcm_example("examples/pcm_ddp_single.ddp71.yaml");
    spec.filter = FilterOverrides {
        encoding_backend: Some("atmosprocessor".to_string()),
        ..FilterOverrides::default()
    };

    let err = resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap_err()
    .to_string();

    assert_eq!(
        err,
        "parameter 'encoding_backend' is not supported by template_id 'pcm_ddp_v1'"
    );
}

#[test]
fn renders_explicit_high_value_pcm_overrides() {
    let (_temp, mut spec) = load_pcm_example("examples/pcm_ddp_single.ddp.yaml");
    spec.filter = FilterOverrides {
        bitstream_mode: Some("commentary".to_string()),
        lfe_on: Some(false),
        user_data: Some(7),
        lfe_lowpass_filter: Some(false),
        surround_90_degree_phase_shift: Some(false),
        surround_3db_attenuation: Some(false),
        allow_hybrid_downmix: Some(true),
        frame_rate: Some("29.97".to_string()),
        ..FilterOverrides::default()
    };

    let resolved = resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap();
    let xml = render_xml(&resolved);

    assert!(xml.contains("<bitstream_mode>commentary</bitstream_mode>"));
    assert!(xml.contains("<lfe_on>false</lfe_on>"));
    assert!(xml.contains("<user_data>7</user_data>"));
    assert!(xml.contains("<lfe_lowpass_filter>false</lfe_lowpass_filter>"));
    assert!(xml.contains("<surround_90_degree_phase_shift>false</surround_90_degree_phase_shift>"));
    assert!(xml.contains("<surround_3db_attenuation>false</surround_3db_attenuation>"));
    assert!(xml.contains("<allow_hybrid_downmix>true</allow_hybrid_downmix>"));
    assert!(xml.contains("<frame_rate>29.97</frame_rate>"));
}

#[test]
fn renders_dd_starting_timecode_override() {
    let (_temp, mut spec) = load_pcm_example("examples/pcm_ddp_single.dd.yaml");
    spec.filter = FilterOverrides {
        starting_timecode: Some("auto".to_string()),
        ..FilterOverrides::default()
    };

    let resolved = resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap();
    let xml = render_xml(&resolved);

    assert!(xml.contains("<starting_timecode>auto</starting_timecode>"));
    assert!(xml.contains("<ac3 version=\"1\">"));
}
