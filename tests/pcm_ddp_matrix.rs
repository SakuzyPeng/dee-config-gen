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
    let ddp71_err = resolve_with_defaults("examples/pcm_ddp_single.ddp71.yaml", |spec| {
        spec.filter.encoder_mode = Some("bluray".to_string());
    })
    .unwrap_err()
    .to_string();
    assert_eq!(ddp71_err, "ddp71 mode requires encoder_mode=ddp71");

    let bluray_err = resolve_with_defaults("examples/pcm_ddp_single.bluray.yaml", |spec| {
        spec.filter.encoder_mode = Some("ddp71".to_string());
    })
    .unwrap_err()
    .to_string();
    assert_eq!(bluray_err, "bluray mode requires encoder_mode=bluray");
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
