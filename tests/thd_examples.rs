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
fn renders_thd_example() {
    let resolved = resolve_from_example("examples/thd_single.mlp.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "thd_v1");
    assert!(xml.contains("<encode_to_dthd version=\"1\">"));
    assert!(xml.contains("<mlp version=\"1\">"));
    assert!(xml.contains("<atmos_presentation>"));
    assert!(xml.contains("<drc_profile>film_light</drc_profile>"));
    assert!(xml.contains("<custom_dialnorm>0</custom_dialnorm>"));
}

#[test]
fn renders_thd_explicit_drc_and_dialnorm_overrides() {
    let mut spec = load_job_file(Path::new("examples/thd_single.mlp.yaml")).unwrap();
    spec.filter = FilterOverrides {
        custom_dialnorm: Some(-9),
        atmos_presentation_drc_profile: Some("speech".to_string()),
        presentation_8ch_drc_profile: Some("film_standard".to_string()),
        presentation_6ch_drc_profile: Some("music_light".to_string()),
        presentation_2ch_drc_profile: Some("music_standard".to_string()),
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

    assert!(xml.contains("<custom_dialnorm>-9</custom_dialnorm>"));
    assert!(xml.contains("<atmos_presentation>"));
    assert!(xml.contains("<presentation_8ch>"));
    assert!(xml.contains("<presentation_6ch>"));
    assert!(xml.contains("<presentation_2ch>"));
    assert!(xml.contains("<drc_profile>speech</drc_profile>"));
    assert!(xml.contains("<drc_profile>film_standard</drc_profile>"));
    assert!(xml.contains("<drc_profile>music_light</drc_profile>"));
    assert!(xml.contains("<drc_profile>music_standard</drc_profile>"));
}

#[test]
fn rejects_pcm_only_override_on_thd_template() {
    let mut spec = load_job_file(Path::new("examples/thd_single.mlp.yaml")).unwrap();
    spec.filter = FilterOverrides {
        bitstream_mode: Some("commentary".to_string()),
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
        "parameter 'bitstream_mode' is not supported by template_id 'thd_v1'"
    );
}
