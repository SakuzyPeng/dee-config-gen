use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{RenderFormat, ResolveOptions, load_job_file, render_config, resolve_job};
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn require_command(name: &str) {
    let status = Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .unwrap_or_else(|err| panic!("failed to probe command '{name}': {err}"));
    assert!(
        status.success(),
        "required command '{name}' is not available"
    );
}

fn run_dee_json(json_path: &Path, log_path: &Path) -> Output {
    Command::new("gtimeout")
        .arg("120")
        .arg("dee")
        .args(["--json", json_path.to_str().expect("utf-8 json path")])
        .args(["--log-file", log_path.to_str().expect("utf-8 log path")])
        .arg("--stdout")
        .output()
        .unwrap_or_else(|err| panic!("failed to run dee for {}: {err}", json_path.display()))
}

fn render_atmos_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples/atmos_ec3_single.streaming.yaml"))
        .expect("load atmos example");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["testADM.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.ec3".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .expect("resolve atmos example");

    render_config(&resolved, RenderFormat::Json).expect("render json")
}

fn render_pcm_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job =
        load_job_file(&root.join("examples/pcm_ddp_single.dd.yaml")).expect("load pcm example");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["input_6ch.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.ac3".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .expect("resolve pcm example");

    render_config(&resolved, RenderFormat::Json).expect("render json")
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let json_path = temp.path().join("job.json");
    let log_path = temp.path().join("dee.log");
    let output_path = temp.path().join("out").join("json_runtime.ec3");
    fs::write(&json_path, render_atmos_json(&temp)).expect("write json");

    let output = run_dee_json(&json_path, &log_path);
    assert!(
        output.status.success(),
        "DEE JSON run should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "DEE JSON run should produce output");
    assert!(
        fs::metadata(output_path).expect("stat output").len() > 0,
        "DEE JSON run should produce non-empty output"
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn pcm_ddp_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let json_path = temp.path().join("job.json");
    let log_path = temp.path().join("dee.log");
    let output_path = temp.path().join("out").join("json_runtime.ac3");
    fs::write(&json_path, render_pcm_json(&temp)).expect("write json");

    let output = run_dee_json(&json_path, &log_path);
    assert!(
        output.status.success(),
        "DEE JSON run should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output_path.exists(), "DEE JSON run should produce output");
    assert!(
        fs::metadata(output_path).expect("stat output").len() > 0,
        "DEE JSON run should produce non-empty output"
    );
}
