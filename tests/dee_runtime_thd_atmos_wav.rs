use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions,
    config::{FilterOverrides, InputsSpec, IoSpec, JobFile},
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

fn create_temp_layout(temp: &TempDir) {
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
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

fn assert_failure_contains(output: &Output, context: &str, needle: &str) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "{context} should fail, stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains(needle) || stderr.contains(needle),
        "{context} should mention '{needle}', stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

fn create_runtime_wav(input_name: &str) -> (TempDir, String, Vec<String>) {
    create_runtime_wav_with_args(input_name, input_name, &[])
}

fn create_runtime_wav_with_channels(
    input_name: &str,
    output_name: &str,
    channel_count: u8,
) -> (TempDir, String, Vec<String>) {
    create_runtime_wav_with_args(
        input_name,
        output_name,
        &["-ac", &channel_count.to_string()],
    )
}

fn create_runtime_wav_with_args(
    input_name: &str,
    output_name: &str,
    extra_args: &[&str],
) -> (TempDir, String, Vec<String>) {
    require_command("ffmpeg");
    let temp = TempDir::new().expect("wav temp dir");
    let wav_dir = temp.path().join("wav");
    fs::create_dir_all(&wav_dir).expect("create wav dir");
    let input_path = repo_root().join("testfiles").join(input_name);
    let out = wav_dir.join(output_name);
    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-i"])
        .arg(&input_path)
        .args(extra_args)
        .args(["-c:a", "pcm_s24le"])
        .arg(&out);
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
    assert!(
        status.success(),
        "ffmpeg conversion failed for {}",
        out.display()
    );
    (
        temp,
        wav_dir.display().to_string(),
        vec![output_name.to_string()],
    )
}

fn render_thd_atmos_wav_xml(
    temp: &TempDir,
    wav_storage_path: &str,
    wav_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobFile),
) -> String {
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples/thd_atmos_wav_single.mlp.yaml"))
        .unwrap_or_else(|err| panic!("load mixed thd example: {err}"));
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: Some(IoSpec {
            storage_path: wav_storage_path.to_string(),
            file_names: wav_file_names,
        }),
        wav_list: None,
    });
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
    .expect("resolve mixed thd job");

    render_xml(&resolved)
}

fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

fn create_runtime_variant_in_dir(
    storage_path: &str,
    input_name: &str,
    output_name: &str,
    extra_args: &[&str],
) {
    require_command("ffmpeg");
    let input_path = repo_root().join("testfiles").join(input_name);
    let out = Path::new(storage_path).join(output_name);
    let mut command = Command::new("ffmpeg");
    command
        .args(["-y", "-i"])
        .arg(&input_path)
        .args(extra_args)
        .args(["-c:a", "pcm_s24le"])
        .arg(&out);
    let status = command
        .status()
        .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
    assert!(
        status.success(),
        "ffmpeg conversion failed for {}",
        out.display()
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_mlp_smoke_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, file_names) = create_runtime_wav("input_6ch.wav");
    let xml =
        render_thd_atmos_wav_xml(&temp, &storage_path, file_names, "mixed_wav_51.mlp", |_| {});
    let output = run_rendered_xml(&temp, "mixed_wav_51.xml", &xml);
    assert_success(&output, "thd_atmos_wav 5.1 smoke");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_51.mlp"),
        "thd_atmos_wav 5.1 smoke",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_8ch_topology_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, file_names) = create_runtime_wav("input_8ch.wav");
    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_wav_8ch.mlp",
        |_| {},
    );
    let output = run_rendered_xml(&temp, "mixed_wav_8ch.xml", &xml);
    assert_success(&output, "thd_atmos_wav 8ch topology");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_8ch.mlp"),
        "thd_atmos_wav 8ch topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_stereo_topology_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, file_names) =
        create_runtime_wav_with_channels("input_6ch.wav", "input_2ch.wav", 2);
    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_wav_stereo.mlp",
        |_| {},
    );
    let output = run_rendered_xml(&temp, "mixed_wav_stereo.xml", &xml);
    assert_success(&output, "thd_atmos_wav stereo topology");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_stereo.mlp"),
        "thd_atmos_wav stereo topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_mono_topology_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, _file_names) = create_runtime_wav("input_6ch.wav");
    create_runtime_variant_in_dir(
        &storage_path,
        "input_6ch.wav",
        "input_1ch.wav",
        &["-ac", "1"],
    );
    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        vec!["input_6ch.wav".to_string()],
        "mixed_wav_mono.mlp",
        |_| {},
    )
    .replace("input_6ch.wav", "input_1ch.wav");
    let output = run_rendered_xml(&temp, "mixed_wav_mono.xml", &xml);
    assert_failure_contains(
        &output,
        "thd_atmos_wav mono topology",
        "Missing media info property: SamplingCount",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_input_timecode_context_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, file_names) = create_runtime_wav("input_6ch.wav");

    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        file_names.clone(),
        "mixed_wav_decimal_timecode.mlp",
        |job| {
            job.filter.input_timecode_frame_rate = Some("not_indicated".to_string());
            job.filter.offset = Some("00:00:01.000".to_string());
            job.filter.ffoa = Some("00:00:02.000".to_string());
            job.filter.start = Some("00:00:00.000".to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.time_base = Some("file_position".to_string());
        },
    );
    let output = run_rendered_xml(&temp, "mixed_wav_decimal_timecode.xml", &xml);
    assert_success(&output, "thd_atmos_wav decimal input timecode context");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_decimal_timecode.mlp"),
        "thd_atmos_wav decimal input timecode context",
    );

    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_wav_frame_timecode.mlp",
        |job| {
            job.filter.input_timecode_frame_rate = Some("24".to_string());
            job.filter.offset = Some("00:00:01:00".to_string());
            job.filter.ffoa = Some("00:00:02:00".to_string());
            job.filter.start = Some("00:00:00:00".to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.time_base = Some("file_position".to_string());
        },
    );
    let output = run_rendered_xml(&temp, "mixed_wav_frame_timecode.xml", &xml);
    assert_success(&output, "thd_atmos_wav frame-style input timecode context");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_frame_timecode.mlp"),
        "thd_atmos_wav frame-style input timecode context",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_offset_with_default_start_is_rejected() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, _file_names) = create_runtime_wav("input_6ch.wav");

    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        vec!["input_6ch.wav".to_string()],
        "mixed_wav_offset_default_start.mlp",
        |job| {
            job.filter.input_timecode_frame_rate = Some("not_indicated".to_string());
            job.filter.offset = Some("00:00:01.000".to_string());
            job.filter.ffoa = Some("00:00:02.000".to_string());
            job.filter.start = Some("00:00:00.000".to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.time_base = Some("file_position".to_string());
        },
    )
    .replace(
        "<start>00:00:00.000</start>",
        "<start>first_frame_of_action</start>",
    );
    let output = run_rendered_xml(&temp, "mixed_wav_offset_default_start.xml", &xml);
    assert_failure_contains(
        &output,
        "thd_atmos_wav offset/ffoa with default start",
        "The duration of 6-channel shifted by its 'offset' value is lesser than the specified 'start' value",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_representative_params_match_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_wav_temp, storage_path, file_names) = create_runtime_wav("input_6ch.wav");
    let xml = render_thd_atmos_wav_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_wav_params.mlp",
        |job| {
            job.filter = FilterOverrides {
                input_timecode_frame_rate: Some("24".to_string()),
                offset: Some("00:00:01.000".to_string()),
                ffoa: Some("00:00:02.000".to_string()),
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
                starting_timecode: Some("auto".to_string()),
                frame_rate: Some("24".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
    let output = run_rendered_xml(&temp, "mixed_wav_params.xml", &xml);
    assert_success(&output, "thd_atmos_wav representative params");
    assert_output_exists(
        &temp.path().join("out/mixed_wav_params.mlp"),
        "thd_atmos_wav representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_media_consistency_guard_matches_runtime() {
    let root = repo_root();
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let mut job = load_job_file(&root.join("examples/thd_atmos_wav_single.mlp.yaml"))
        .expect("load mixed thd example");
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: Some(IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["input_6ch.wav".to_string()],
        }),
        wav_list: None,
    });
    job.output.storage_path = temp.path().join("out").display().to_string();
    job.output.file_names = vec!["mixed_guard.mlp".to_string()];
    job.misc.temp_dir = temp.path().join("tmp").display().to_string();

    let err = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Z',
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires matching bits_per_sample"));
}
