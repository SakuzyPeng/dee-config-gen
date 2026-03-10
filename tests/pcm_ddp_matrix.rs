use std::path::Path;

use dee_config_gen::{
    ResolveOptions,
    config::{FilterOverrides, Profile},
    load_job_file, resolve_job,
};

fn resolve_with_defaults(
    path: &str,
    mutate: impl FnOnce(&mut dee_config_gen::JobFile),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let mut spec = load_job_file(Path::new(path)).expect("load example");
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

fn resolved_filter(path: &str) -> dee_config_gen::template::pcm_ddp_v1::PcmDdpV1Filter {
    let resolved = resolve_with_defaults(path, |_| {}).expect("resolve example");
    let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = resolved.filter else {
        panic!("expected PcmDdpV1 filter");
    };
    filter
}

#[test]
fn accepts_all_runtime_verified_dd_bitrates() {
    for bitrate in [224_u16, 256, 320, 384, 448, 512, 576, 640] {
        let resolved = resolve_with_defaults("examples/pcm_ddp_single.dd.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_or_else(|err| panic!("dd bitrate={bitrate} should resolve: {err}"));

        let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
            panic!("expected PcmDdpV1 filter");
        };
        assert_eq!(filter.data_rate, bitrate);
        assert_eq!(filter.encoder_mode, "dd");
        assert_eq!(filter.downmix_config, "5.1");
    }
}

#[test]
fn accepts_all_runtime_verified_ddp_bitrates() {
    for bitrate in [
        192_u16, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384,
        400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
    ] {
        let resolved = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_or_else(|err| panic!("ddp bitrate={bitrate} should resolve: {err}"));

        let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
            panic!("expected PcmDdpV1 filter");
        };
        assert_eq!(filter.data_rate, bitrate);
        assert_eq!(filter.encoder_mode, "ddp");
        assert_eq!(filter.downmix_config, "5.1");
    }
}

#[test]
fn accepts_all_runtime_verified_ddp71_bitrates() {
    for bitrate in [384_u16, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024] {
        let resolved = resolve_with_defaults("examples/pcm_ddp_single.ddp71.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_or_else(|err| panic!("ddp71 bitrate={bitrate} should resolve: {err}"));

        let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
            panic!("expected PcmDdpV1 filter");
        };
        assert_eq!(filter.data_rate, bitrate);
        assert_eq!(filter.encoder_mode, "ddp71");
        assert_eq!(filter.downmix_config, "off");
    }
}

#[test]
fn accepts_all_runtime_verified_bluray_bitrates() {
    for bitrate in [768_u16, 1024, 1280, 1536, 1664] {
        let resolved = resolve_with_defaults("examples/pcm_ddp_single.bluray.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_or_else(|err| panic!("bluray bitrate={bitrate} should resolve: {err}"));

        let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
            panic!("expected PcmDdpV1 filter");
        };
        assert_eq!(filter.data_rate, bitrate);
        assert_eq!(filter.encoder_mode, "bluray");
        assert_eq!(filter.downmix_config, "off");
    }
}

#[test]
fn rejects_dd_bitrates_outside_runtime_set() {
    for bitrate in [192_u16, 200, 768] {
        let err = resolve_with_defaults("examples/pcm_ddp_single.dd.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_err()
        .to_string();

        assert!(
            err.contains(&format!("invalid data_rate '{bitrate}' for mode 'dd'")),
            "expected dd bitrate={bitrate} to be rejected, got: {err}"
        );
        assert!(err.contains("allowed: 224, 256, 320, 384, 448, 512, 576, 640"));
    }
}

#[test]
fn rejects_ddp_bitrates_outside_runtime_set() {
    for bitrate in [191_u16, 193, 1664] {
        let err = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_err()
        .to_string();

        assert!(
            err.contains(&format!("invalid data_rate '{bitrate}' for mode 'ddp'")),
            "expected ddp bitrate={bitrate} to be rejected, got: {err}"
        );
        assert!(err.contains("allowed: 192, 200, 208, 216"));
    }
}

#[test]
fn rejects_ddp71_bitrates_outside_runtime_set() {
    for bitrate in [1280_u16, 1536, 1664] {
        let err = resolve_with_defaults("examples/pcm_ddp_single.ddp71.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_err()
        .to_string();

        assert!(
            err.contains(&format!("invalid data_rate '{bitrate}' for mode 'ddp71'")),
            "expected ddp71 bitrate={bitrate} to be rejected, got: {err}"
        );
        assert!(err.contains("allowed: 384, 448, 576, 640, 704, 768, 832, 896, 960, 1008, 1024"));
    }
}

#[test]
fn rejects_bluray_bitrates_outside_runtime_set() {
    for bitrate in [704_u16, 1152, 1408] {
        let err = resolve_with_defaults("examples/pcm_ddp_single.bluray.yaml", |spec| {
            spec.filter.data_rate = Some(bitrate);
        })
        .unwrap_err()
        .to_string();

        assert!(
            err.contains(&format!("invalid data_rate '{bitrate}' for mode 'bluray'")),
            "expected bluray bitrate={bitrate} to be rejected, got: {err}"
        );
        assert!(err.contains("allowed: 768, 1024, 1280, 1536, 1664"));
    }
}

#[test]
fn requires_matching_encoder_mode_for_pcm_modes() {
    let cases = [
        (
            "examples/pcm_ddp_single.dd.yaml",
            "ddp",
            "dd mode requires encoder_mode=dd",
        ),
        (
            "examples/pcm_ddp_single.ddp.yaml",
            "dd",
            "ddp mode requires encoder_mode=ddp",
        ),
        (
            "examples/pcm_ddp_single.ddp71.yaml",
            "bluray",
            "ddp71 mode requires encoder_mode=ddp71",
        ),
        (
            "examples/pcm_ddp_single.bluray.yaml",
            "ddp71",
            "bluray mode requires encoder_mode=bluray",
        ),
    ];

    for (path, override_mode, expected) in cases {
        let err = resolve_with_defaults(path, |spec| {
            spec.filter.encoder_mode = Some(override_mode.to_string());
        })
        .unwrap_err()
        .to_string();
        assert_eq!(err, expected);
    }
}

#[test]
fn enforces_mode_specific_downmix_config() {
    let cases = [
        (
            "examples/pcm_ddp_single.dd.yaml",
            "off",
            "dd mode requires downmix_config=5.1",
        ),
        (
            "examples/pcm_ddp_single.ddp.yaml",
            "off",
            "ddp mode requires downmix_config=5.1",
        ),
        (
            "examples/pcm_ddp_single.ddp71.yaml",
            "5.1",
            "ddp71 mode requires downmix_config=off",
        ),
        (
            "examples/pcm_ddp_single.bluray.yaml",
            "5.1",
            "bluray mode requires downmix_config=off",
        ),
    ];

    for (path, downmix, expected) in cases {
        let err = resolve_with_defaults(path, |spec| {
            spec.filter.downmix_config = Some(downmix.to_string());
        })
        .unwrap_err()
        .to_string();
        assert_eq!(err, expected);
    }
}

#[test]
fn validates_first_wave_pcm_advanced_fields() {
    let resolved = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| {
        spec.filter.bitstream_mode = Some("commentary".to_string());
        spec.filter.user_data = Some(65535);
        spec.filter.lfe_on = Some(false);
        spec.filter.lfe_lowpass_filter = Some(false);
        spec.filter.surround_90_degree_phase_shift = Some(false);
        spec.filter.surround_3db_attenuation = Some(false);
        spec.filter.allow_hybrid_downmix = Some(true);
        spec.filter.frame_rate = Some("29.97".to_string());
        spec.filter.dolby_surround_mode = Some("yes".to_string());
        spec.filter.dolby_surround_ex_mode = Some("not_indicated".to_string());
    })
    .expect("advanced overrides should resolve");

    let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
        panic!("expected PcmDdpV1 filter");
    };

    assert_eq!(filter.bitstream_mode, "commentary");
    assert_eq!(filter.user_data, 65535);
    assert!(!filter.lfe_on);
    assert!(!filter.lfe_lowpass_filter);
    assert!(!filter.surround_90_degree_phase_shift);
    assert!(!filter.surround_3db_attenuation);
    assert!(filter.allow_hybrid_downmix);
    assert_eq!(filter.frame_rate, "29.97");
    assert_eq!(filter.dolby_surround_mode, "yes");
    assert_eq!(filter.dolby_surround_ex_mode, "not_indicated");
}

#[test]
fn rejects_pcm_metering_mode_1770_4() {
    for path in [
        "examples/pcm_ddp_single.dd.yaml",
        "examples/pcm_ddp_single.ddp.yaml",
        "examples/pcm_ddp_single.ddp71.yaml",
        "examples/pcm_ddp_single.bluray.yaml",
    ] {
        let err = resolve_with_defaults(path, |spec| {
            spec.filter.metering_mode = Some("1770-4".to_string());
        })
        .expect_err("pcm_ddp_v1 should reject 1770-4")
        .to_string();

        assert!(
            err.contains("invalid value '1770-4' for metering_mode"),
            "expected path={path} to reject 1770-4, got: {err}"
        );
        assert!(err.contains("allowed: 1770-1, 1770-2, 1770-3, LeqA"));
    }
}

#[test]
fn accepts_dd_starting_timecode_override() {
    let resolved = resolve_with_defaults("examples/pcm_ddp_single.dd.yaml", |spec| {
        spec.filter.starting_timecode = Some("auto".to_string());
    })
    .expect("dd starting_timecode should resolve");

    let dee_config_gen::ResolvedFilter::PcmDdpV1(filter) = &resolved.filter else {
        panic!("expected PcmDdpV1 filter");
    };
    assert_eq!(filter.starting_timecode, "auto");
}

#[test]
fn rejects_invalid_pcm_advanced_field_values() {
    let enum_cases = [
        ("bitstream_mode", "bad_mode"),
        ("downmix_config", "7.1"),
        ("dolby_surround_mode", "maybe"),
        ("dolby_surround_ex_mode", "auto"),
        ("frame_rate", "48"),
    ];

    for (field, value) in enum_cases {
        let err = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| match field {
            "bitstream_mode" => spec.filter.bitstream_mode = Some(value.to_string()),
            "downmix_config" => spec.filter.downmix_config = Some(value.to_string()),
            "dolby_surround_mode" => spec.filter.dolby_surround_mode = Some(value.to_string()),
            "dolby_surround_ex_mode" => {
                spec.filter.dolby_surround_ex_mode = Some(value.to_string())
            }
            "frame_rate" => spec.filter.frame_rate = Some(value.to_string()),
            other => panic!("unsupported field in test: {other}"),
        })
        .unwrap_err()
        .to_string();

        assert!(err.contains(field), "expected {field} error, got: {err}");
    }

    for invalid in [-2_i32, 65536] {
        let err = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| {
            spec.filter.user_data = Some(invalid);
        })
        .unwrap_err()
        .to_string();
        assert!(err.contains("user_data"));
    }

    let start_err = resolve_with_defaults("examples/pcm_ddp_single.ddp.yaml", |spec| {
        spec.filter.starting_timecode = Some("auto".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(start_err.contains("starting_timecode"));
    assert!(start_err.contains("mode 'ddp'"));
}

#[test]
fn keeps_pcm_default_values_stable_for_existing_modes() {
    let ddp71 = resolved_filter("examples/pcm_ddp_single.ddp71.yaml");
    assert_eq!(ddp71.bitstream_mode, "complete_main");
    assert_eq!(ddp71.user_data, -1);
    assert!(ddp71.lfe_on);
    assert!(ddp71.lfe_lowpass_filter);
    assert!(ddp71.surround_90_degree_phase_shift);
    assert!(ddp71.surround_3db_attenuation);
    assert!(!ddp71.allow_hybrid_downmix);
    assert_eq!(ddp71.starting_timecode, "off");
    assert_eq!(ddp71.frame_rate, "auto");

    let bluray = resolved_filter("examples/pcm_ddp_single.bluray.yaml");
    assert_eq!(bluray.bitstream_mode, "complete_main");
    assert_eq!(bluray.user_data, -1);
    assert_eq!(bluray.downmix_config, "off");
}

#[test]
fn locks_music_fixed_values_for_pcm_template() {
    let err = resolve_with_defaults("examples/pcm_ddp_single.ddp71.yaml", |spec| {
        spec.profile = Profile::Music;
        spec.filter = FilterOverrides {
            line_mode_drc_profile: Some("film_light".to_string()),
            ..FilterOverrides::default()
        };
    })
    .unwrap_err()
    .to_string();

    assert!(err.contains("profile=music"));
    assert!(err.contains("line_mode_drc_profile"));
}

#[test]
fn rejects_all_atmos_only_overrides_on_pcm_template() {
    let cases = [
        (
            "encoding_backend",
            FilterOverrides {
                encoding_backend: Some("atmosprocessor".to_string()),
                ..FilterOverrides::default()
            },
        ),
        (
            "surround_trim_5_1",
            FilterOverrides {
                surround_trim_5_1: Some("auto".to_string()),
                ..FilterOverrides::default()
            },
        ),
        (
            "surround_trim_7_1",
            FilterOverrides {
                surround_trim_7_1: Some("auto".to_string()),
                ..FilterOverrides::default()
            },
        ),
        (
            "height_trim_5_1",
            FilterOverrides {
                height_trim_5_1: Some("auto".to_string()),
                ..FilterOverrides::default()
            },
        ),
    ];

    for (field, filter) in cases {
        let err = resolve_with_defaults("examples/pcm_ddp_single.ddp71.yaml", |spec| {
            spec.filter = filter.clone();
        })
        .unwrap_err()
        .to_string();

        assert_eq!(
            err,
            format!("parameter '{field}' is not supported by template_id 'pcm_ddp_v1'")
        );
    }
}
