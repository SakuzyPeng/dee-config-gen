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
fn renders_pcm_ddp71_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.ddp71.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>ddp71</encoder_mode>"));
    assert!(xml.contains("<data_rate>448</data_rate>"));
}

#[test]
fn renders_pcm_bluray_example() {
    let resolved = resolve_from_example("examples/pcm_ddp_single.bluray.yaml");
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "pcm_ddp_v1");
    assert!(xml.contains("<pcm_to_ddp version=\"3\">"));
    assert!(xml.contains("<encoder_mode>bluray</encoder_mode>"));
    assert!(xml.contains("<data_rate>1536</data_rate>"));
}

#[test]
fn rejects_atmos_only_override_on_pcm_template() {
    let mut spec = load_job_file(Path::new("examples/pcm_ddp_single.ddp71.yaml")).unwrap();
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
