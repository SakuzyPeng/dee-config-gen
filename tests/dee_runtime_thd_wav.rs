mod common;

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

fn write_text(path: &Path, content: &str) {
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn create_temp_layout(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
}

fn run_dee(xml_path: &Path, log_path: &Path) -> Output {
    let xml_windows = common::host_path_to_windows_workspace(xml_path);
    let log_windows = common::host_path_to_windows_workspace(log_path);
    let mut command = Command::new("gtimeout");
    command
        .arg("120")
        .arg("dee")
        .args(["--xml", &xml_windows])
        .args(["--log-file", &log_windows])
        .arg("--stdout");
    let context = format!("dee thd_wav {}", xml_path.display());
    common::run_dee_command(command, &context)
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} should succeed, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure_contains(output: &Output, context: &str, needle: &str) {
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "{context} should fail, stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        combined.contains(needle),
        "{context} should mention '{needle}', got:\n{combined}"
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

fn replace_leaf_value(xml: &str, tag: &str, new_value: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml
        .find(&open)
        .unwrap_or_else(|| panic!("missing <{tag}> in xml"));
    let value_start = start + open.len();
    let rest = &xml[value_start..];
    let end_rel = rest
        .find(&close)
        .unwrap_or_else(|| panic!("missing </{tag}> in xml"));
    let value_end = value_start + end_rel;

    let mut patched = String::with_capacity(xml.len() + new_value.len());
    patched.push_str(&xml[..value_start]);
    patched.push_str(new_value);
    patched.push_str(&xml[value_end..]);
    patched
}

fn replace_leaf_value_in_parent(xml: &str, parent_tag: &str, tag: &str, new_value: &str) -> String {
    let parent_open = format!("<{parent_tag}>");
    let parent_close = format!("</{parent_tag}>");
    let parent_start = xml
        .find(&parent_open)
        .unwrap_or_else(|| panic!("missing <{parent_tag}> in xml"));
    let parent_end_rel = xml[parent_start..]
        .find(&parent_close)
        .unwrap_or_else(|| panic!("missing </{parent_tag}> in xml"));
    let parent_end = parent_start + parent_end_rel + parent_close.len();
    let parent_xml = &xml[parent_start..parent_end];

    let updated_parent = replace_leaf_value(parent_xml, tag, new_value);
    let mut patched = String::with_capacity(xml.len() + new_value.len());
    patched.push_str(&xml[..parent_start]);
    patched.push_str(&updated_parent);
    patched.push_str(&xml[parent_end..]);
    patched
}

fn render_thd_wav_xml(
    temp: &TempDir,
    input_name: &str,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/thd_wav_single.mlp.yaml"))
        .unwrap_or_else(|err| panic!("load thd_wav example: {err}"));
    job.input.storage_path = root.join("testfiles").display().to_string();
    job.input.file_names = vec![input_name.to_string()];
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
    .expect("resolve thd_wav job");

    render_xml(&resolved)
}

fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_mlp_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_wav_xml(&temp, "input_6ch.wav", "wav_smoke.mlp", |_| {});
    let output = run_rendered_xml(&temp, "wav_smoke.xml", &xml);
    assert_success(&output, "thd_wav mlp smoke");
    assert_output_exists(&temp.path().join("out/wav_smoke.mlp"), "thd_wav mlp smoke");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_input_topology_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_wav_xml(&temp, "input_8ch.wav", "wav_8ch.mlp", |_| {});
    let output = run_rendered_xml(&temp, "wav_8ch.xml", &xml);
    assert_success(&output, "thd_wav 8ch smoke");
    assert_output_exists(&temp.path().join("out/wav_8ch.mlp"), "thd_wav 8ch smoke");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_representative_params_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_wav_xml(&temp, "input_6ch.wav", "wav_representative.mlp", |job| {
        job.filter = FilterOverrides {
            input_timecode_frame_rate: Some("24".to_string()),
            offset: Some("00:00:00.000".to_string()),
            ffoa: Some("00:00:00.000".to_string()),
            metering_mode: Some("1770-3".to_string()),
            dialogue_intelligence: Some(false),
            speech_threshold: Some(42),
            timecode_frame_rate: Some("23.976".to_string()),
            start: Some("00:00:00:00".to_string()),
            end: Some("end_of_file".to_string()),
            time_base: Some("file_position".to_string()),
            custom_dialnorm: Some(-9),
            atmos_presentation_drc_profile: Some("speech".to_string()),
            presentation_8ch_drc_profile: Some("film_standard".to_string()),
            presentation_6ch_drc_profile: Some("music_light".to_string()),
            presentation_2ch_drc_profile: Some("music_standard".to_string()),
            spatial_clusters: Some("14".to_string()),
            legacy_authoring_compatibility: Some(false),
            optimize_data_rate: Some(true),
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "wav_representative.xml", &xml);

    assert_success(&output, "thd_wav representative params");
    assert_output_exists(
        &temp.path().join("out/wav_representative.mlp"),
        "thd_wav representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_embedded_timecodes_runtime_matrix_matches_runtime() {
    require_command("dee");

    let success_cases = [
        (
            "wav_off_auto_default",
            "input_6ch.wav",
            "not_indicated",
            "off",
            "auto",
            "file_position",
            "00:00:00:00",
            "23.976",
        ),
        (
            "wav_auto_auto",
            "input_6ch.wav",
            "24",
            "auto",
            "auto",
            "file_position",
            "00:00:00:00",
            "24",
        ),
        (
            "wav_explicit_starting_timecode",
            "input_6ch.wav",
            "24",
            "00:23:01:00",
            "30",
            "file_position",
            "00:00:00:00",
            "24",
        ),
    ];

    for (
        name,
        input_name,
        input_timecode_frame_rate,
        starting_timecode,
        frame_rate,
        time_base,
        start,
        timecode_frame_rate,
    ) in success_cases
    {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_wav_xml(&temp, input_name, &format!("{name}.mlp"), |job| {
            job.filter.input_timecode_frame_rate = Some(input_timecode_frame_rate.to_string());
            job.filter.offset = Some("00:00:00.000".to_string());
            job.filter.ffoa = Some("00:00:00.000".to_string());
            job.filter.time_base = Some(time_base.to_string());
            job.filter.start = Some(start.to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.timecode_frame_rate = Some(timecode_frame_rate.to_string());
            job.filter.starting_timecode = Some(starting_timecode.to_string());
            job.filter.frame_rate = Some(frame_rate.to_string());
        });
        let output = run_rendered_xml(&temp, &format!("{name}.xml"), &xml);
        assert_success(&output, &format!("thd_wav embedded timecodes {name}"));
        assert_output_exists(
            &temp.path().join(format!("out/{name}.mlp")),
            &format!("thd_wav embedded timecodes {name}"),
        );
    }

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let xml = render_thd_wav_xml(
        &temp,
        "input_6ch.wav",
        "wav_invalid_frame_rate.mlp",
        |job| {
            job.filter.input_timecode_frame_rate = Some("24".to_string());
            job.filter.offset = Some("00:00:00.000".to_string());
            job.filter.ffoa = Some("00:00:00.000".to_string());
            job.filter.time_base = Some("file_position".to_string());
            job.filter.start = Some("00:00:00:00".to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.timecode_frame_rate = Some("24".to_string());
        },
    );
    let xml = replace_leaf_value_in_parent(&xml, "embedded_timecodes", "starting_timecode", "auto");
    let xml = replace_leaf_value_in_parent(&xml, "embedded_timecodes", "frame_rate", "bogus");
    let output = run_rendered_xml(&temp, "wav_invalid_frame_rate.xml", &xml);
    assert_failure_contains(
        &output,
        "thd_wav embedded frame_rate=bogus",
        "embedded_timecodes:frame_rate",
    );
}
