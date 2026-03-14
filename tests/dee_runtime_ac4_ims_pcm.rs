mod common;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use common::create_mono_wav_stems;
use dee_config_gen::{ResolveOptions, read_job, render_xml, resolve_job};
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

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_runtime_smoke_matches_runtime() {
    require_command("dee");

    let root = repo_root();
    let temp = TempDir::new().expect("create temp dir");
    let mut job = read_job(&root.join("examples/ac4_ims_pcm_single.ac4.yaml"))
        .expect("load ac4 ims pcm example");
    job.inputs
        .as_mut()
        .expect("inputs")
        .wav
        .as_mut()
        .expect("wav")
        .storage_path = root.join("testfiles").display().to_string();
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    fs::create_dir_all(&job.output.storage_path).expect("create output dir");
    fs::create_dir_all(&job.misc.temp_dir).expect("create temp dir");

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .expect("resolve ac4 ims pcm");

    let xml = render_xml(&resolved);
    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("dee.log");
    fs::write(&xml_path, xml).expect("write job xml");

    let output = run_dee(&xml_path, &log_path);
    assert!(
        output.status.success(),
        "AC-4 IMS pcm runtime smoke should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_wav_list_runtime_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    let (_stems_temp, stem_storage, stem_files) = create_mono_wav_stems(6);
    let mut job = read_job(&repo_root().join("examples/ac4_ims_pcm_single.ac4.yaml"))
        .expect("load example");
    let inputs = job.inputs.as_mut().expect("inputs");
    inputs.wav = None;
    inputs.wav_list = Some(dee_config_gen::IoSpec {
        storage_path: stem_storage,
        file_names: stem_files,
    });
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();
    fs::create_dir_all(&job.output.storage_path).expect("create output dir");
    fs::create_dir_all(&job.misc.temp_dir).expect("create temp dir");

    let resolved = resolve_job(job, &ResolveOptions::default()).expect("resolve wav_list job");
    let xml = render_xml(&resolved);
    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("dee.log");
    fs::write(&xml_path, xml).expect("write job xml");

    let output = run_dee(&xml_path, &log_path);
    assert!(
        output.status.success(),
        "AC-4 IMS pcm wav_list runtime smoke should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
