mod common;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    JobSpec, RenderFormat, ResolveOptions, read_job, render_config, resolve_job, spec::InputsSpec,
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

fn dee_workspace_root() -> PathBuf {
    env::var_os("DEE_WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let repo = repo_root();
            repo.parent()
                .and_then(|p| p.parent())
                .map(|root| root.join("dee-win"))
                .unwrap_or_else(|| PathBuf::from("dee-win"))
        })
}

fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root().join(path)
    }
}

fn workspace_relative_unix_path(path: &Path) -> String {
    let target = absolute_path(path);
    let root = absolute_path(&dee_workspace_root());
    if let Ok(rel) = target.strip_prefix(&root) {
        format!("/{}", rel.to_string_lossy().replace('\\', "/"))
    } else {
        target.to_string_lossy().to_string()
    }
}

fn workspace_repo_root() -> PathBuf {
    let repo = repo_root();
    let repo_tail = repo
        .strip_prefix("/")
        .unwrap_or_else(|_| panic!("repo_root must be absolute: {}", repo.display()));
    dee_workspace_root().join(repo_tail)
}

#[cfg(unix)]
fn ensure_workspace_users_passthrough() {
    use std::os::unix::fs::symlink;
    let root = dee_workspace_root();
    let link = root.join("Users");
    if link.exists() {
        return;
    }
    symlink("/Users", &link).unwrap_or_else(|err| {
        panic!(
            "failed to create Users passthrough symlink {} -> /Users: {err}",
            link.display()
        )
    });
}

#[cfg(not(unix))]
fn ensure_workspace_users_passthrough() {}

#[cfg(unix)]
fn ensure_workspace_repo_passthrough() {
    use std::os::unix::fs::symlink;

    let root = dee_workspace_root();
    let repo = repo_root();
    let repo_tail = repo
        .strip_prefix("/")
        .unwrap_or_else(|_| panic!("repo_root must be absolute: {}", repo.display()));
    let link = root.join(repo_tail);
    if link.exists() {
        return;
    }
    let parent = link
        .parent()
        .unwrap_or_else(|| panic!("missing parent for passthrough link {}", link.display()));
    fs::create_dir_all(parent).unwrap_or_else(|err| {
        panic!(
            "failed to create passthrough parent directory {}: {err}",
            parent.display()
        )
    });
    symlink(&repo, &link).unwrap_or_else(|err| {
        panic!(
            "failed to create repo passthrough symlink {} -> {}: {err}",
            link.display(),
            repo.display()
        )
    });
}

#[cfg(not(unix))]
fn ensure_workspace_repo_passthrough() {}

fn runtime_preflight() {
    require_command("dee");
    let workspace_root = dee_workspace_root();
    assert!(
        workspace_root.exists(),
        "DEE workspace root does not exist: {} (set DEE_WORKSPACE_ROOT if needed)",
        workspace_root.display()
    );
    ensure_workspace_users_passthrough();
    ensure_workspace_repo_passthrough();
}

fn runtime_tempdir(prefix: &str) -> TempDir {
    let repo = repo_root();
    let repo_tail = repo
        .strip_prefix("/")
        .unwrap_or_else(|_| panic!("repo_root must be absolute: {}", repo.display()));
    let root = dee_workspace_root()
        .join(repo_tail)
        .join("target/runtime_temp");
    fs::create_dir_all(&root).expect("create runtime temp root");
    tempfile::Builder::new()
        .prefix(prefix)
        .tempdir_in(root)
        .expect("create runtime temp dir")
}

fn run_dee_json(json_path: &Path, log_path: &Path) -> Output {
    let timeout_seconds = env::var("DEE_RUNTIME_TIMEOUT_SECS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(300);
    let timeout_seconds = timeout_seconds.to_string();
    let json_windows = common::host_path_to_windows_workspace(json_path);
    let log_windows = common::host_path_to_windows_workspace(log_path);
    let mut command = Command::new("gtimeout");
    command
        .arg(&timeout_seconds)
        .arg("dee")
        .args(["--json", &json_windows])
        .args(["--log-file", &log_windows])
        .arg("--stdout");
    let context = format!("dee json {}", json_path.display());
    common::run_dee_command(command, &context)
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} should succeed (exit status: {:?}), stdout:\n{}\nstderr:\n{}",
        output.status.code(),
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

fn resolve_job_file(spec: JobSpec) -> dee_config_gen::ResolvedJob {
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
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
    let temp = runtime_tempdir("json_stems_");
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
    let mut job = read_job(&root.join("examples/atmos_ec3_single.streaming.yaml"))
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
        read_job(&root.join("examples/pcm_ddp_single.dd.yaml")).expect("load pcm example");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["input_6ch.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.ac3".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/thd_single.mlp.yaml")).expect("load thd example");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["testADM.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_wav_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/thd_wav_single.mlp.yaml")).expect("load thd wav");
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec!["input_6ch.wav".to_string()];
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    render_json(&resolve_job_file(job))
}

fn render_thd_wav_list_json(temp: &TempDir) -> String {
    let root = repo_root();
    let (stems_temp, storage_path, file_names) = create_runtime_mono_stems(&[0, 1, 2, 3, 4, 5]);
    let mut job =
        read_job(&root.join("examples/thd_wav_list_single.mlp.yaml")).expect("load thd wav_list");
    job.input.storage_path = storage_path;
    job.input.file_names = file_names;
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    let rendered = render_json(&resolve_job_file(job));
    assert!(
        stems_temp.path().exists(),
        "thd wav_list runtime fixture tempdir should still exist while rendering"
    );
    rendered
}

fn render_thd_atmos_wav_json(temp: &TempDir) -> String {
    let root = repo_root();
    let mut job =
        read_job(&root.join("examples/thd_atmos_wav_single.mlp.yaml")).expect("load thd atmos+wav");
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::spec::IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: Some(dee_config_gen::spec::IoSpec {
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
    let (stems_temp, storage_path, file_names) = create_runtime_mono_stems(&[0, 1, 2, 3, 4, 5]);
    let mut job = read_job(&root.join("examples/thd_atmos_wav_list_single.mlp.yaml"))
        .expect("load thd atmos+wav_list");
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::spec::IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: None,
        wav_list: Some(dee_config_gen::spec::IoSpec {
            storage_path,
            file_names,
        }),
    });
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["json_runtime.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    let rendered = render_json(&resolve_job_file(job));
    assert!(
        stems_temp.path().exists(),
        "thd atmos+wav_list runtime fixture tempdir should still exist while rendering"
    );
    rendered
}

fn render_ac4_ims_atmos_json(temp: &TempDir, output_name: &str) -> String {
    let workspace_repo = workspace_repo_root();
    let mut job = read_job(&repo_root().join("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 atmos");
    job.inputs
        .as_mut()
        .expect("inputs")
        .atmos_mezz
        .as_mut()
        .expect("atmos_mezz")
        .storage_path = workspace_relative_unix_path(&workspace_repo.join("testfiles"));
    job.output.storage_path = workspace_relative_unix_path(&temp.path().join("out"));
    job.output.file_names = vec![output_name.to_string()];
    if output_name.ends_with(".mp4") {
        job.output.container = dee_config_gen::spec::OutputContainer::Mp4;
    }
    job.misc.temp_dir = workspace_relative_unix_path(&temp.path().join("tmp"));
    render_json(&resolve_job_file(job))
}

fn render_ac4_ims_pcm_json(temp: &TempDir, output_name: &str) -> String {
    let workspace_repo = workspace_repo_root();
    let mut job =
        read_job(&repo_root().join("examples/ac4_ims_pcm_single.ac4.yaml")).expect("load ac4 pcm");
    job.inputs
        .as_mut()
        .expect("inputs")
        .wav
        .as_mut()
        .expect("wav")
        .storage_path = workspace_relative_unix_path(&workspace_repo.join("testfiles"));
    job.output.storage_path = workspace_relative_unix_path(&temp.path().join("out"));
    job.output.file_names = vec![output_name.to_string()];
    if output_name.ends_with(".mp4") {
        job.output.container = dee_config_gen::spec::OutputContainer::Mp4;
    }
    job.misc.temp_dir = workspace_relative_unix_path(&temp.path().join("tmp"));
    render_json(&resolve_job_file(job))
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_json_runtime_smoke_matches_xml_behavior() {
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    require_command("ffmpeg");
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
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
    runtime_preflight();
    require_command("ffmpeg");
    let temp = runtime_tempdir("json_runtime_");
    create_temp_layout(&temp);
    let (json_path, log_path) = write_json_job(&temp, &render_thd_atmos_wav_list_json(&temp));
    let output_path = temp.path().join("out").join("json_runtime.mlp");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "thd atmos+wav_list json runtime");
    assert_output_exists(&output_path, "thd atmos+wav_list json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn ac4_ims_atmos_json_runtime_smoke_matches_xml_behavior() {
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
    create_temp_layout(&temp);
    let (json_path, log_path) =
        write_json_job(&temp, &render_ac4_ims_atmos_json(&temp, "json_runtime.ac4"));
    let output_path = temp.path().join("out").join("json_runtime.ac4");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "ac4 atmos json runtime");
    assert_output_exists(&output_path, "ac4 atmos json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn ac4_ims_atmos_mp4_json_runtime_smoke_matches_xml_behavior() {
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
    create_temp_layout(&temp);
    let (json_path, log_path) =
        write_json_job(&temp, &render_ac4_ims_atmos_json(&temp, "json_runtime.mp4"));
    let output_path = temp.path().join("out").join("json_runtime.mp4");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "ac4 atmos mp4 json runtime");
    assert_output_exists(&output_path, "ac4 atmos mp4 json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn ac4_ims_pcm_json_runtime_smoke_matches_xml_behavior() {
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
    create_temp_layout(&temp);
    let (json_path, log_path) =
        write_json_job(&temp, &render_ac4_ims_pcm_json(&temp, "json_runtime.ac4"));
    let output_path = temp.path().join("out").join("json_runtime.ac4");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "ac4 pcm json runtime");
    assert_output_exists(&output_path, "ac4 pcm json runtime");
}

#[test]
#[ignore = "requires local dee runtime"]
fn ac4_ims_pcm_mp4_json_runtime_smoke_matches_xml_behavior() {
    runtime_preflight();
    let temp = runtime_tempdir("json_runtime_");
    create_temp_layout(&temp);
    let (json_path, log_path) =
        write_json_job(&temp, &render_ac4_ims_pcm_json(&temp, "json_runtime.mp4"));
    let output_path = temp.path().join("out").join("json_runtime.mp4");
    let output = run_dee_json(&json_path, &log_path);
    assert_success(&output, "ac4 pcm mp4 json runtime");
    assert_output_exists(&output_path, "ac4 pcm mp4 json runtime");
}
