use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, JobSpec},
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

fn write_text(path: &Path, content: &str) {
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn create_temp_layout(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
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

fn render_ac4_ims_atmos_xml(
    temp: &TempDir,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/ac4_ims_atmos_single.ac4.yaml"))
        .unwrap_or_else(|err| panic!("load ac4 ims atmos example: {err}"));
    job.inputs
        .as_mut()
        .expect("inputs")
        .atmos_mezz
        .as_mut()
        .expect("atmos_mezz")
        .storage_path = root.join("testfiles").display().to_string();
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
    .expect("resolve ac4 ims atmos");

    render_xml(&resolved)
}

fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_runtime_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, "smoke.ac4", |_| {});
    let output = run_rendered_xml(&temp, "smoke.xml", &xml);
    assert_success(&output, "ac4 ims atmos smoke");
    assert_output_exists(&temp.path().join("out/smoke.ac4"), "ac4 ims atmos smoke");
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_representative_params_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, "representative.ac4", |job| {
        job.filter = FilterOverrides {
            metering_mode: Some("1770-3".to_string()),
            dialogue_intelligence: Some(false),
            speech_threshold: Some(42),
            data_rate: Some(72),
            time_base: Some("file_position".to_string()),
            ac4_frame_rate: Some("24".to_string()),
            language: Some("eng".to_string()),
            encoding_profile: Some("ims_music".to_string()),
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "representative.xml", &xml);

    assert_success(&output, "ac4 ims atmos representative params");
    assert_output_exists(
        &temp.path().join("out/representative.ac4"),
        "ac4 ims atmos representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_fixed_flags_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, "fixed_flags.ac4", |job| {
        job.filter = FilterOverrides {
            ims_legacy_presentation: Some(true),
            iframe_interval: Some(24),
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "fixed_flags.xml", &xml);

    assert_success(&output, "ac4 ims atmos fixed flags");
    assert_output_exists(
        &temp.path().join("out/fixed_flags.ac4"),
        "ac4 ims atmos fixed flags",
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_drc_profiles_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, "drc_profiles.ac4", |job| {
        job.filter = FilterOverrides {
            ddp_drc_profile: Some("speech".to_string()),
            flat_panel_drc_profile: Some("music_standard".to_string()),
            home_theatre_drc_profile: Some("film_standard".to_string()),
            portable_hp_drc_profile: Some("music_light".to_string()),
            portable_spkr_drc_profile: Some("none".to_string()),
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "drc_profiles.xml", &xml);

    assert_success(&output, "ac4 ims atmos drc profiles");
    assert_output_exists(
        &temp.path().join("out/drc_profiles.ac4"),
        "ac4 ims atmos drc profiles",
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_time_base_runtime_matrix_matches_runtime() {
    require_command("dee");

    for (name, time_base, start, end) in [
        ("file_position", "file_position", "00:00:00:00", "00:00:00:20"),
        (
            "embedded_timecode",
            "embedded_timecode",
            "01:00:00:00",
            "01:00:00:20",
        ),
    ] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);

        let output_name = format!("{name}.ac4");
        let xml_name = format!("{name}.xml");
        let xml = render_ac4_ims_atmos_xml(&temp, &output_name, |job| {
            job.filter = FilterOverrides {
                time_base: Some(time_base.to_string()),
                start: Some(start.to_string()),
                end: Some(end.to_string()),
                timecode_frame_rate: Some("24".to_string()),
                ..FilterOverrides::default()
            };
        });
        let output = run_rendered_xml(&temp, &xml_name, &xml);

        assert_success(&output, &format!("ac4 ims atmos time_base={time_base}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("ac4 ims atmos time_base={time_base}"),
        );
    }
}
