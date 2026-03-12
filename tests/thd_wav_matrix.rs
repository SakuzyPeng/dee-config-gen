use std::path::Path;

use dee_config_gen::{ResolveOptions, load_job_file, resolve_job};

fn resolve_with_defaults(
    mutate: impl FnOnce(&mut dee_config_gen::JobFile),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let mut spec =
        load_job_file(Path::new("examples/thd_wav_single.mlp.yaml")).expect("load example");
    mutate(&mut spec);
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
}

#[test]
fn resolves_default_thd_wav_filter_values() {
    let resolved = resolve_with_defaults(|_| {}).expect("resolve thd wav defaults");
    let dee_config_gen::ResolvedFilter::ThdWavV1(filter) = resolved.filter else {
        panic!("expected ThdWavV1 filter");
    };

    assert_eq!(filter.input_timecode_frame_rate, "not_indicated");
    assert_eq!(filter.offset, "auto");
    assert_eq!(filter.ffoa, "auto");
    assert_eq!(filter.metering_mode, "1770-4");
    assert!(filter.dialogue_intelligence);
    assert_eq!(filter.speech_threshold, 15);
    assert_eq!(filter.timecode_frame_rate, "not_indicated");
    assert_eq!(filter.starting_timecode, "off");
    assert_eq!(filter.frame_rate, "auto");
    assert_eq!(filter.start, "first_frame_of_action");
    assert_eq!(filter.end, "end_of_file");
    assert_eq!(filter.time_base, "file_position");
    assert_eq!(filter.prepend_silence_duration, "0");
    assert_eq!(filter.append_silence_duration, "0");
    assert_eq!(filter.custom_dialnorm, 0);
    assert_eq!(filter.spatial_clusters, "12");
    assert!(filter.legacy_authoring_compatibility);
    assert!(!filter.optimize_data_rate);
}

#[test]
fn accepts_documented_thd_wav_overrides() {
    resolve_with_defaults(|spec| {
        spec.filter.input_timecode_frame_rate = Some("24".to_string());
        spec.filter.offset = Some("auto".to_string());
        spec.filter.ffoa = Some("00:00:00.000".to_string());
        spec.filter.starting_timecode = Some("auto".to_string());
        spec.filter.frame_rate = Some("23.976".to_string());
        spec.filter.start = Some("00:00:00:00".to_string());
        spec.filter.end = Some("00:00:01.000".to_string());
        spec.filter.prepend_silence_duration = Some("1.25".to_string());
        spec.filter.append_silence_duration = Some("0.005333".to_string());
        spec.filter.custom_dialnorm = Some(-9);
        spec.filter.spatial_clusters = Some("16".to_string());
        spec.filter.legacy_authoring_compatibility = Some(false);
        spec.filter.optimize_data_rate = Some(true);
    })
    .expect("documented thd_wav overrides should resolve");
}

#[test]
fn rejects_invalid_thd_wav_formats_and_values() {
    let err = resolve_with_defaults(|spec| {
        spec.filter.start = Some("0:00:00.0".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("HH:MM:SS:FF[df], or HH:MM:SS.xx"));

    let err = resolve_with_defaults(|spec| {
        spec.filter.prepend_silence_duration = Some("1f".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("seconds.milliseconds"));

    let err = resolve_with_defaults(|spec| {
        spec.filter.custom_dialnorm = Some(1);
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("expected -31..0"));

    let err = resolve_with_defaults(|spec| {
        spec.filter.spatial_clusters = Some("10".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid value '10' for spatial_clusters"));

    let err = resolve_with_defaults(|spec| {
        spec.filter.starting_timecode = Some("bogus".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid starting_timecode"));

    let err = resolve_with_defaults(|spec| {
        spec.filter.frame_rate = Some("bogus".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid value 'bogus' for frame_rate"));
}
