use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions,
    config::{FilterOverrides, JobFile},
    load_job_file, render_xml, resolve_job,
};
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

fn write_text(path: &Path, content: &str) {
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn run_dee(xml_path: &Path, log_path: &Path) -> Output {
    Command::new("gtimeout")
        .arg("120")
        .arg("dee")
        .args(["--xml", xml_path.to_str().expect("utf-8 xml path")])
        .args(["--log-file", log_path.to_str().expect("utf-8 log path")])
        .arg("--stdout")
        .output()
        .unwrap_or_else(|err| panic!("failed to run dee for {}: {err}", xml_path.display()))
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_output_exists(path: &Path, context: &str) {
    assert!(path.exists(), "{context} should produce {}", path.display());
    let metadata =
        fs::metadata(path).unwrap_or_else(|err| panic!("failed to stat {}: {err}", path.display()));
    assert!(
        metadata.len() > 0,
        "{context} should produce non-empty output"
    );
}

fn render_thd_xml(temp: &TempDir, output_name: &str, mutate: impl FnOnce(&mut JobFile)) -> String {
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples/thd_single.mlp.yaml"))
        .unwrap_or_else(|err| panic!("load thd example: {err}"));
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["testADM.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec![output_name.to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    mutate(&mut job);

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .expect("resolve thd job");

    render_xml(&resolved)
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_mlp_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let xml = render_thd_xml(&temp, "smoke.mlp", |_| {});
    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("job.log");
    write_text(&xml_path, &xml);

    let output = run_dee(&xml_path, &log_path);
    assert_success(&output, "thd mlp smoke");
    assert_output_exists(&temp.path().join("out/smoke.mlp"), "thd mlp smoke");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_representative_params_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let xml = render_thd_xml(&temp, "representative.mlp", |job| {
        job.filter = FilterOverrides {
            metering_mode: Some("1770-3".to_string()),
            dialogue_intelligence: Some(false),
            speech_threshold: Some(42),
            timecode_frame_rate: Some("23.976".to_string()),
            start: Some("00:00:00:00".to_string()),
            end: Some("00:00:01:00".to_string()),
            time_base: Some("file_position".to_string()),
            custom_dialnorm: Some(-9),
            atmos_presentation_drc_profile: Some("speech".to_string()),
            presentation_8ch_drc_profile: Some("film_standard".to_string()),
            presentation_6ch_drc_profile: Some("music_light".to_string()),
            presentation_2ch_drc_profile: Some("music_standard".to_string()),
            ..FilterOverrides::default()
        };
    });
    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("job.log");
    write_text(&xml_path, &xml);

    let output = run_dee(&xml_path, &log_path);
    assert_success(&output, "thd representative params");
    assert_output_exists(
        &temp.path().join("out/representative.mlp"),
        "thd representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_mp4_feasibility_lane_records_runtime_result() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_thd_xml(&temp, "candidate.mlp", |_| {});
    let xml = base_xml
        .replace(
            "<mlp version=\"1\">\n      <file_name>candidate.mlp</file_name>",
            "<mp4 version=\"1\">\n      <file_name>candidate.mp4</file_name>",
        )
        .replace("</mlp>", "</mp4>");

    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("job.log");
    write_text(&xml_path, &xml);

    let output = run_dee(&xml_path, &log_path);
    if output.status.success() {
        assert_output_exists(
            &temp.path().join("out/candidate.mp4"),
            "thd mp4 feasibility",
        );
    } else {
        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            combined.contains("unsupported")
                || combined.contains("Unknown property")
                || combined.contains("Unknown element")
                || combined.contains("mp4")
                || combined.contains("mux"),
            "thd mp4 feasibility should fail with a recognizable unsupported-output signal, got:\n{combined}"
        );
    }
}
