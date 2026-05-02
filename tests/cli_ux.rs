use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn dcg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_dee-config-gen"))
}

fn run(args: &[&str]) -> Output {
    dcg().args(args).output().expect("run dee-config-gen")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn templates_list_supports_json_output() {
    let output = run(&["templates", "list", "--format", "json"]);
    assert_success(&output);

    let parsed: Value = serde_json::from_slice(&output.stdout).expect("valid json");
    let templates = parsed.as_array().expect("template list array");
    assert!(
        templates
            .iter()
            .any(|item| item["template_id"] == "atmos_ec3_v1")
    );
    assert!(
        templates
            .iter()
            .all(|item| item["canonical_example"].is_string())
    );
}

#[test]
fn templates_show_supports_json_output() {
    let output = run(&["templates", "show", "atmos_ec3_v1", "--format", "json"]);
    assert_success(&output);

    let parsed: Value = serde_json::from_slice(&output.stdout).expect("valid json");
    assert_eq!(parsed["template_id"], "atmos_ec3_v1");
    assert!(parsed["parameters"].as_array().expect("parameters").len() > 3);
}

#[test]
fn init_writes_default_job_and_validate_accepts_it() {
    let temp = TempDir::new().expect("temp dir");

    let init = dcg()
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .expect("run init");
    assert_success(&init);

    let job = temp.path().join("job.yaml");
    assert!(job.exists(), "default init should write job.yaml");

    let validate = dcg()
        .current_dir(temp.path())
        .args(["validate", "-i", "job.yaml"])
        .output()
        .expect("run validate");
    assert_success(&validate);
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let temp = TempDir::new().expect("temp dir");
    let job = temp.path().join("job.yaml");
    std::fs::write(&job, "original").expect("seed file");

    let refused = dcg()
        .current_dir(temp.path())
        .args(["init"])
        .output()
        .expect("run init");
    assert_failure(&refused);
    assert_eq!(
        std::fs::read_to_string(&job).expect("read file"),
        "original"
    );

    let overwritten = dcg()
        .current_dir(temp.path())
        .args(["init", "--force"])
        .output()
        .expect("run init --force");
    assert_success(&overwritten);
    assert_ne!(
        std::fs::read_to_string(&job).expect("read file"),
        "original"
    );
}

#[test]
fn init_rejects_unknown_example() {
    let output = run(&["init", "--example", "missing.yaml"]);
    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown example 'missing.yaml'"));
}

#[test]
fn missing_input_error_includes_cause_chain() {
    let output = run(&["validate", "-i", "/tmp/dee-config-gen-missing-input.yaml"]);
    assert_failure(&output);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("failed to read job file"));
    assert!(stderr.contains("caused by:"));
    assert!(stderr.contains("os error 2"));
}
