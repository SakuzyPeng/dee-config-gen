use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    JobFile, RenderFormat, ResolveOptions, config::InputsSpec, load_job_file, render_config,
    resolve_job,
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

fn create_temp_layout(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
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
    assert!(
        fs::metadata(path).expect("stat output").len() > 0,
        "{context} should produce non-empty output"
    );
}

fn resolve_job_file(spec: JobFile) -> dee_config_gen::ResolvedJob {
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .expect("resolve job")
}

fn render_json(job: &dee_config_gen::ResolvedJob) -> String {
    render_config(job, RenderFormat::Json).expect("render json")
}

fn write_json_job(temp: &TempDir, content: &str) -> (PathBuf, PathBuf) {
    let json_path = temp.path().join("job.json");
    let log_path = temp.path().join("dee.log");
    fs::write(&json_path, content).expect("write json");
    (json_path, log_path)
}

fn create_runtime_mono_stems(channel_indices: &[usize]) -> (TempDir, String, Vec<String>) {
    require_command("ffmpeg");
    let temp = TempDir::new().expect("mono stem temp dir");
    let stem_dir = temp.path().join("stems");
    fs::create_dir_all(&stem_dir).expect("create stem dir");

    let input_path = repo_root().join("testfiles").join("16ch.wav");
    for (slot, channel_idx) in channel_indices.iter().enumerate() {
        let out = stem_dir.join(format!("stem_{slot:02}.wav"));
        let pan = format!("pan=mono|c0=c{channel_idx}");
        let status = Command::new("ffmpeg")
            .args(["-y", "-i"])
            .arg(&input_path)
            .args(["-filter:a", &pan, "-c:a", "pcm_s24le"])
            .arg(&out)
            .status()
            .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
        assert!(status.success(), "ffmpeg stem generation failed");
    }

    let file_names = (0..channel_indices.len())
        .map(|idx| format!("stem_{idx:02}.wav"))
        .collect::<Vec<_>>();
    (temp, stem_dir.display().to_string(), file_names)
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
    render_json(&resolve_job_file(job))
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
    render_json(&resolve_job_file(job))
}

fn render_thd_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job =
        load_job_file(&root.join("examples/thd_single.mlp.yaml")).expect("load thd example");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["testADM.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_wav_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job =
        load_job_file(&root.join("examples/thd_wav_single.mlp.yaml")).expect("load thd wav");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["input_6ch.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_wav_list_json(temp: &TempDir) -> String {
    let root = repo_root();
    let (_stems_temp, storage_path, file_names) = create_runtime_mono_stems(&[0, 1, 2, 3, 4, 5]);
    let mut job = load_job_file(&root.join("examples/thd_wav_list_single.mlp.yaml"))
        .expect("load thd wav_list");
    job.input.storage_path = storage_path;
    job.input.file_names = file_names;
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_atmos_wav_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples/thd_atmos_wav_single.mlp.yaml"))
        .expect("load thd atmos+wav");
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::config::IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: Some(dee_config_gen::config::IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["input_6ch.wav".to_string()],
        }),
        wav_list: None,
    });
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_atmos_wav_list_json(temp: &TempDir) -> String {
    let root = repo_root();
    let (_stems_temp, storage_path, file_names) = create_runtime_mono_stems(&[0, 1, 2, 3, 4, 5]);
    let mut job = load_job_file(&root.join("examples/thd_atmos_wav_list_single.mlp.yaml"))
        .expect("load thd atmos+wav_list");
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::config::IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: None,
        wav_list: Some(dee_config_gen::config::IoSpec {
            storage_path,
            file_names,
        }),
    });
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_atmos_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.ec3");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "atmos json runtime");
    assert_output_exists(&output_path, "atmos json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn pcm_ddp_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_pcm_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.ac3");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "pcm json runtime");
    assert_output_exists(&output_path, "pcm json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd json runtime");
    assert_output_exists(&output_path, "thd json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_wav_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd wav json runtime");
    assert_output_exists(&output_path, "thd wav json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    require_command("ffmpeg");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_wav_list_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd wav_list json runtime");
    assert_output_exists(&output_path, "thd wav_list json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_atmos_wav_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd atmos+wav json runtime");
    assert_output_exists(&output_path, "thd atmos+wav json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_json_runtime_smoke_matches_xml_behavior() {
    require_command("dee");
    require_command("ffmpeg");
    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_atmos_wav_list_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd atmos+wav_list json runtime");
    assert_output_exists(&output_path, "thd atmos+wav_list json runtime");
}
