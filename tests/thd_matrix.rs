use std::path::Path;

use dee_config_gen::{ResolveOptions, load_job_file, resolve_job};

fn resolve_with_defaults(
    mutate: impl FnOnce(&mut dee_config_gen::JobFile),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let mut spec = load_job_file(Path::new("examples/thd_single.mlp.yaml")).expect("load example");
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
fn resolves_default_thd_filter_values() {
    let resolved = resolve_with_defaults(|_| {}).expect("resolve thd defaults");
    let dee_config_gen::ResolvedFilter::ThdV1(filter) = resolved.filter else {
        panic!("expected ThdV1 filter");
    };

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
fn accepts_documented_truehd_start_end_formats() {
    for value in [
        "first_frame_of_action",
        "00:00:00:00",
        "00:00:00:00df",
        "00:00:00.0",
    ] {
        resolve_with_defaults(|spec| {
            spec.filter.start = Some(value.to_string());
        })
        .unwrap_or_else(|err| panic!("start={value} should resolve: {err}"));
    }

    for value in ["end_of_file", "00:00:00:00", "00:00:00:00df", "00:00:00.0"] {
        resolve_with_defaults(|spec| {
            spec.filter.end = Some(value.to_string());
        })
        .unwrap_or_else(|err| panic!("end={value} should resolve: {err}"));
    }
}

#[test]
fn rejects_invalid_truehd_start_end_formats() {
    for (field, value) in [("start", "0:00:00.0"), ("end", "00:00")] {
        let err = resolve_with_defaults(|spec| match field {
            "start" => spec.filter.start = Some(value.to_string()),
            "end" => spec.filter.end = Some(value.to_string()),
            _ => unreachable!(),
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains("HH:MM:SS:FF[df], or HH:MM:SS.xx"));
    }
}

#[test]
fn accepts_truehd_decimal_silence_durations() {
    for value in ["0", "0.005333", "1.25"] {
        resolve_with_defaults(|spec| {
            spec.filter.prepend_silence_duration = Some(value.to_string());
        })
        .unwrap_or_else(|err| panic!("prepend_silence_duration={value} should resolve: {err}"));
    }
}

#[test]
fn rejects_truehd_frame_silence_durations() {
    for key in ["prepend_silence_duration", "append_silence_duration"] {
        let err = resolve_with_defaults(|spec| match key {
            "prepend_silence_duration" => {
                spec.filter.prepend_silence_duration = Some("1f".to_string())
            }
            "append_silence_duration" => {
                spec.filter.append_silence_duration = Some("1f".to_string())
            }
            _ => unreachable!(),
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("seconds.milliseconds"));
    }
}

#[test]
fn validates_truehd_custom_dialnorm_boundaries() {
    for value in [-31_i8, 0] {
        resolve_with_defaults(|spec| {
            spec.filter.custom_dialnorm = Some(value);
        })
        .unwrap_or_else(|err| panic!("custom_dialnorm={value} should resolve: {err}"));
    }

    for value in [-32_i8, 1] {
        let err = resolve_with_defaults(|spec| {
            spec.filter.custom_dialnorm = Some(value);
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("expected -31..0"));
    }
}

#[test]
fn validates_truehd_drc_profile_overrides() {
    let cases = [
        ("atmos_presentation_drc_profile", "speech"),
        ("presentation_8ch_drc_profile", "film_standard"),
        ("presentation_6ch_drc_profile", "music_light"),
        ("presentation_2ch_drc_profile", "music_standard"),
    ];

    for (field, value) in cases {
        resolve_with_defaults(|spec| match field {
            "atmos_presentation_drc_profile" => {
                spec.filter.atmos_presentation_drc_profile = Some(value.to_string())
            }
            "presentation_8ch_drc_profile" => {
                spec.filter.presentation_8ch_drc_profile = Some(value.to_string())
            }
            "presentation_6ch_drc_profile" => {
                spec.filter.presentation_6ch_drc_profile = Some(value.to_string())
            }
            "presentation_2ch_drc_profile" => {
                spec.filter.presentation_2ch_drc_profile = Some(value.to_string())
            }
            _ => unreachable!(),
        })
        .unwrap_or_else(|err| panic!("{field}={value} should resolve: {err}"));
    }

    let err = resolve_with_defaults(|spec| {
        spec.filter.presentation_2ch_drc_profile = Some("none".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid value 'none' for presentation_2ch_drc_profile"));
}

#[test]
fn validates_truehd_fixed_field_overrides() {
    resolve_with_defaults(|spec| {
        spec.filter.spatial_clusters = Some("14".to_string());
        spec.filter.legacy_authoring_compatibility = Some(false);
        spec.filter.optimize_data_rate = Some(true);
    })
    .expect("fixed field overrides should resolve");

    let err = resolve_with_defaults(|spec| {
        spec.filter.spatial_clusters = Some("10".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid value '10' for spatial_clusters"));
}

#[test]
fn validates_truehd_embedded_timecode_overrides() {
    resolve_with_defaults(|spec| {
        spec.filter.starting_timecode = Some("auto".to_string());
        spec.filter.frame_rate = Some("23.976".to_string());
    })
    .expect("embedded timecode overrides should resolve");

    resolve_with_defaults(|spec| {
        spec.filter.frame_rate = Some("24".to_string());
    })
    .expect("frame_rate=24 should resolve");

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
