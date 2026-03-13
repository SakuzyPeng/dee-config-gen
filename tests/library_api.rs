use std::fs;

use dee_config_gen::{
    GenerateOptions, RenderFormat, RunOptions, generate_config, parse_job_str, read_job, run_job,
    validate_job,
};

fn sample_yaml_job() -> &'static str {
    r#"
template_id: atmos_ec3_v1
profile: standard
job_mode: single
encode_mode: streaming
input:
  storage_path: ./input
  file_names: [testADM.wav]
output:
  storage_path: ./output
  file_names: [output.ec3]
misc:
  temp_dir: ./tmp
"#
}

fn sample_json_job() -> &'static str {
    r#"{
  "template_id": "atmos_ec3_v1",
  "profile": "standard",
  "job_mode": "single",
  "encode_mode": "streaming",
  "input": {"storage_path": "./input", "file_names": ["testADM.wav"]},
  "output": {"storage_path": "./output", "file_names": ["output.ec3"]},
  "misc": {"temp_dir": "./tmp"}
}"#
}

#[test]
fn parse_job_str_accepts_yaml_and_json() {
    let yaml = parse_job_str(sample_yaml_job()).expect("parse yaml");
    let json = parse_job_str(sample_json_job()).expect("parse json");

    assert_eq!(yaml.template_id.as_deref(), Some("atmos_ec3_v1"));
    assert_eq!(json.template_id.as_deref(), Some("atmos_ec3_v1"));
}

#[test]
fn parse_job_str_reports_yaml_or_json_failure() {
    let err = parse_job_str("template_id: [").expect_err("invalid input should fail");
    assert!(
        err.to_string()
            .contains("failed to parse input as YAML or JSON"),
        "unexpected parse error: {err}",
    );
}

#[test]
fn validate_job_returns_resolved_job() {
    let resolved = validate_job(
        parse_job_str(sample_yaml_job()).unwrap(),
        &Default::default(),
    )
    .expect("validate job");
    assert_eq!(resolved.template_id, "atmos_ec3_v1");
    assert_eq!(resolved.output.file_names, vec!["output.ec3".to_string()]);
}

#[test]
fn generate_config_renders_xml_and_json() {
    let xml = generate_config(
        parse_job_str(sample_yaml_job()).unwrap(),
        &GenerateOptions::default(),
    )
    .expect("generate xml");
    assert!(xml.rendered.starts_with("<?xml version=\"1.0\"?>"));

    let json = generate_config(
        parse_job_str(sample_json_job()).unwrap(),
        &GenerateOptions {
            format: RenderFormat::Json,
            ..GenerateOptions::default()
        },
    )
    .expect("generate json");
    assert!(json.rendered.contains("\"job_config\""));
}

#[test]
fn read_job_reads_from_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("job.yaml");
    fs::write(&path, sample_yaml_job()).expect("write job");

    let spec = read_job(&path).expect("read job");
    assert_eq!(spec.template_id.as_deref(), Some("atmos_ec3_v1"));
}

#[test]
fn run_job_invokes_fake_runner() {
    let dir = tempfile::tempdir().expect("tempdir");
    let script = dir.path().join("capture_args.sh");
    let marker = dir.path().join("args.txt");
    let generated_config = dir.path().join("job.xml");

    fs::write(
        &script,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > {}\nexit 0\n",
            marker.display()
        ),
    )
    .expect("write script");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = fs::metadata(&script).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script, perms).unwrap();
    }

    let code = run_job(
        parse_job_str(sample_yaml_job()).unwrap(),
        &GenerateOptions::default(),
        &RunOptions {
            keep_config: true,
            generated_config: Some(generated_config.clone()),
            runner_cmd: Some(script.display().to_string()),
            ..RunOptions::default()
        },
    )
    .expect("run job");

    assert_eq!(code, 0);
    assert!(generated_config.exists());
    let args = fs::read_to_string(marker).expect("read args");
    assert!(args.contains("--xml"));
    assert!(args.contains(generated_config.to_string_lossy().as_ref()));
}
