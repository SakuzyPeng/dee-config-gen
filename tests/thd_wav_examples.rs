use std::path::Path;

use dee_config_gen::{
    ResolveOptions, config::FilterOverrides, load_job_file, render_xml, resolve_job,
};

fn resolve_from_example(path: &str) -> dee_config_gen::ResolvedJob {
    let spec = load_job_file(Path::new(path)).unwrap();
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
fn renders_thd_wav_example() {
    let resolved = resolve_from_example("examples/thd_wav_single.mlp.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "thd_wav_v1");
    assert!(xml.contains("<wav version=\"1\">"));
    assert!(xml.contains("<encode_to_dthd version=\"1\">"));
    assert!(xml.contains("<mlp version=\"1\">"));
    assert!(xml.contains("<offset>auto</offset>"));
    assert!(xml.contains("<ffoa>auto</ffoa>"));
}

#[test]
fn renders_thd_wav_explicit_field_overrides() {
    let mut spec = load_job_file(Path::new("examples/thd_wav_single.mlp.yaml")).unwrap();
    spec.filter = FilterOverrides {
        input_timecode_frame_rate: Some("24".to_string()),
        offset: Some("00:00:01.000".to_string()),
        ffoa: Some("00:00:02.000".to_string()),
        spatial_clusters: Some("14".to_string()),
        legacy_authoring_compatibility: Some(false),
        optimize_data_rate: Some(true),
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

    assert!(xml.contains("<timecode_frame_rate>24</timecode_frame_rate>"));
    assert!(xml.contains("<offset>00:00:01.000</offset>"));
    assert!(xml.contains("<ffoa>00:00:02.000</ffoa>"));
    assert!(xml.contains("<spatial_clusters>14</spatial_clusters>"));
    assert!(xml.contains("<legacy_authoring_compatibility>false</legacy_authoring_compatibility>"));
    assert!(xml.contains("<optimize_data_rate>true</optimize_data_rate>"));
}
