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
fn xml_matches_streaming_example_snapshot() {
    let actual = render_xml(&resolve_from_example(
        "examples/atmos_ec3_single.streaming.yaml",
    ));
    let expected = include_str!("fixtures/atmos_ec3_single.streaming.xml");
    assert_eq!(actual, expected);
}

#[test]
fn xml_matches_bluray_example_snapshot() {
    let actual = render_xml(&resolve_from_example(
        "examples/atmos_ec3_single.bluray.yaml",
    ));
    let expected = include_str!("fixtures/atmos_ec3_single.bluray.xml");
    assert_eq!(actual, expected);
}

#[test]
fn xml_matches_album_music_example_snapshot() {
    let actual = render_xml(&resolve_from_example("examples/atmos_ec3_album.music.json"));
    let expected = include_str!("fixtures/atmos_ec3_album.music.xml");
    assert_eq!(actual, expected);
}

#[test]
fn preserves_streaming_mode_extension_error_message() {
    let mut spec = load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml")).unwrap();
    spec.filter = FilterOverrides {
        encoder_mode: Some("bluray".to_string()),
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
        "encoding_backend/encoder_mode are mode extensions and cannot be set for streaming mode"
    );
}

#[test]
fn preserves_bluray_encoder_mode_required_message() {
    let mut spec = load_job_file(Path::new("examples/atmos_ec3_single.bluray.yaml")).unwrap();
    spec.filter = FilterOverrides {
        encoder_mode: Some("ddp71".to_string()),
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

    assert_eq!(err, "bluray mode requires encoder_mode=bluray");
}

#[test]
fn preserves_ddp71_encoding_backend_forbidden_message() {
    let mut spec = load_job_file(Path::new("examples/atmos_ec3_single.streaming.yaml")).unwrap();
    spec.encode_mode = dee_config_gen::config::EncodeMode::Ddp71;
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

    assert_eq!(err, "encoding_backend is unsupported for ddp71 mode");
}
