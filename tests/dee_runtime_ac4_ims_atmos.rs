use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

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
fn ac4_ims_atmos_runtime_smoke_matches_runtime() {
    require_command("dee");

    let root = repo_root();
    let temp = TempDir::new().expect("create temp dir");
    let mut job = read_job(&root.join("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    job.inputs
        .as_mut()
        .expect("inputs")
        .atmos_mezz
        .as_mut()
        .expect("atmos_mezz")
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
    .expect("resolve ac4 ims atmos");

    let xml = render_xml(&resolved);
    let xml_path = temp.path().join("job.xml");
    let log_path = temp.path().join("dee.log");
    fs::write(&xml_path, xml).expect("write job xml");

    let output = run_dee(&xml_path, &log_path);
    assert!(
        output.status.success(),
        "AC-4 IMS atmos runtime smoke should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
