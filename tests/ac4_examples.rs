use std::path::Path;

use dee_config_gen::{RenderFormat, read_job, render_config, render_xml, resolve_job};

#[test]
fn resolves_ac4_example() {
    let spec = read_job(Path::new("examples/ac4_single.ac4.yaml")).expect("load ac4 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4");
    assert_eq!(resolved.template_id, "ac4_v1");
    assert_eq!(resolved.encode_mode.as_str(), "ac4");
}

#[test]
fn renders_ac4_xml_structure() {
    let spec = read_job(Path::new("examples/ac4_single.ac4.yaml")).expect("load ac4 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4");
    let xml = render_xml(&resolved);

    assert!(xml.contains("<ac4 version=\"1\">"));
    assert!(xml.contains("<input>"));
    assert!(xml.contains("<audio>"));
    assert!(xml.contains("<output>"));
    assert!(xml.contains("<file_name>input.ac4</file_name>"));
    assert!(xml.contains("<file_name>output.ac4</file_name>"));
}

#[test]
fn rejects_album_mode_for_ac4() {
    let mut spec = read_job(Path::new("examples/ac4_single.ac4.yaml")).expect("load ac4 example");
    spec.job_mode = dee_config_gen::spec::JobMode::Album;

    let err = resolve_job(spec, &Default::default())
        .expect_err("album should fail")
        .to_string();
    assert_eq!(err, "template_id 'ac4_v1' only supports job_mode=single");
}

#[test]
fn rejects_non_ac4_mode_for_ac4_template() {
    let mut spec = read_job(Path::new("examples/ac4_single.ac4.yaml")).expect("load ac4 example");
    spec.encode_mode = dee_config_gen::spec::EncodeMode::Streaming;

    let err = resolve_job(spec, &Default::default())
        .expect_err("streaming should fail")
        .to_string();
    assert_eq!(err, "unsupported encode_mode 'streaming'; allowed: ac4");
}

#[test]
fn rejects_json_output_for_ac4() {
    let spec = read_job(Path::new("examples/ac4_single.ac4.yaml")).expect("load ac4 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4");
    let err = render_config(&resolved, RenderFormat::Json)
        .expect_err("json should be unsupported")
        .to_string();
    assert_eq!(err, "template 'ac4_v1' does not support JSON output");
}
