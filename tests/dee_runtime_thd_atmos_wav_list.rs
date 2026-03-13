mod common;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions,
    config::{InputsSpec, IoSpec, JobFile, Profile},
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

fn create_runtime_mono_stems(
    input_name: &str,
    channel_indices: &[usize],
) -> (TempDir, String, Vec<String>) {
    require_command("ffmpeg");
    let temp = TempDir::new().expect("mono stem temp dir");
    let stem_dir = temp.path().join("stems");
    fs::create_dir_all(&stem_dir).expect("create stem dir");

    let input_path = repo_root().join("testfiles").join(input_name);
    for (slot, channel_idx) in channel_indices.iter().enumerate() {
        let out = stem_dir.join(format!("stem_{slot:02}.wav"));
        let pan = format!("pan=mono|c0=c{channel_idx}");
        let status = Command::new("ffmpeg")
            .args(["-y", "-i"])
            .arg(&input_path)
            .args(["-filter:a", &pan])
            .args(["-c:a", "pcm_s24le"])
            .arg(&out)
            .status()
            .unwrap_or_else(|err| panic!("failed to run ffmpeg for {}: {err}", out.display()));
        assert!(
            status.success(),
            "ffmpeg stem generation failed for channel {channel_idx}"
        );
    }

    let file_names = (0..channel_indices.len())
        .map(|idx| format!("stem_{idx:02}.wav"))
        .collect::<Vec<_>>();
    (temp, stem_dir.display().to_string(), file_names)
}

fn render_thd_atmos_wav_list_xml(
    temp: &TempDir,
    wav_list_storage_path: &str,
    wav_list_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobFile),
) -> String {
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples/thd_atmos_wav_list_single.mlp.yaml"))
        .unwrap_or_else(|err| panic!("load mixed thd example: {err}"));
    job.inputs = Some(InputsSpec {
        atmos_mezz: Some(IoSpec {
            storage_path: root.join("testfiles").display().to_string(),
            file_names: vec!["testADM.wav".to_string()],
        }),
        wav: None,
        wav_list: Some(IoSpec {
            storage_path: wav_list_storage_path.to_string(),
            file_names: wav_list_file_names,
        }),
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

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_mlp_smoke_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);
    let xml =
        render_thd_atmos_wav_list_xml(&temp, &storage_path, file_names, "mixed_51.mlp", |_| {});
    let output = run_rendered_xml(&temp, "mixed_51.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list 5.1 smoke");
    assert_output_exists(
        &temp.path().join("out/mixed_51.mlp"),
        "thd_atmos_wav_list 5.1 smoke",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_71_topology_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5, 6, 7]);
    let xml =
        render_thd_atmos_wav_list_xml(&temp, &storage_path, file_names, "mixed_71.mlp", |_| {});
    assert!(xml.contains("<channel_configuration>7.1</channel_configuration>"));
    let output = run_rendered_xml(&temp, "mixed_71.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list 7.1 topology");
    assert_output_exists(
        &temp.path().join("out/mixed_71.mlp"),
        "thd_atmos_wav_list 7.1 topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_stereo_topology_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) = create_runtime_mono_stems("16ch.wav", &[0, 1]);
    let xml =
        render_thd_atmos_wav_list_xml(&temp, &storage_path, file_names, "mixed_stereo.mlp", |_| {});
    assert!(xml.contains("<channel_configuration>stereo</channel_configuration>"));
    let output = run_rendered_xml(&temp, "mixed_stereo.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list stereo topology");
    assert_output_exists(
        &temp.path().join("out/mixed_stereo.mlp"),
        "thd_atmos_wav_list stereo topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_representative_params_match_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);
    let xml = render_thd_atmos_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_params.mlp",
        |job| {
            job.filter.channel_configuration = Some("5.1".to_string());
            job.filter.input_timecode_frame_rate = Some("24".to_string());
            job.filter.offset = Some("00:00:01.000".to_string());
            job.filter.ffoa = Some("00:00:02.000".to_string());
            job.filter.metering_mode = Some("1770-3".to_string());
            job.filter.dialogue_intelligence = Some(false);
            job.filter.speech_threshold = Some(42);
            job.filter.start = Some("00:00:00:00".to_string());
            job.filter.end = Some("00:00:01:00".to_string());
            job.filter.time_base = Some("file_position".to_string());
            job.filter.custom_dialnorm = Some(-9);
            job.filter.spatial_clusters = Some("14".to_string());
            job.filter.legacy_authoring_compatibility = Some(false);
            job.filter.optimize_data_rate = Some(true);
        },
    );
    let output = run_rendered_xml(&temp, "mixed_params.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list representative params");
    assert_output_exists(
        &temp.path().join("out/mixed_params.mlp"),
        "thd_atmos_wav_list representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_music_profile_runtime_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);

    let xml = render_thd_atmos_wav_list_xml(
        &temp,
        &storage_path,
        file_names.clone(),
        "mixed_music_defaults.mlp",
        |job| {
            job.profile = Profile::Music;
        },
    );
    let output = run_rendered_xml(&temp, "mixed_music_defaults.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list music profile defaults");
    assert_output_exists(
        &temp.path().join("out/mixed_music_defaults.mlp"),
        "thd_atmos_wav_list music profile defaults",
    );

    let xml = render_thd_atmos_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_music_drc.mlp",
        |job| {
            job.profile = Profile::Music;
            job.filter.presentation_6ch_drc_profile = Some("music_light".to_string());
            job.filter.presentation_2ch_drc_profile = Some("music_standard".to_string());
        },
    );
    let output = run_rendered_xml(&temp, "mixed_music_drc.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list music profile drc variant");
    assert_output_exists(
        &temp.path().join("out/mixed_music_drc.mlp"),
        "thd_atmos_wav_list music profile drc variant",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_atmos_wav_list_offset_with_default_start_matches_runtime() {
    require_command("dee");
    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);

    let xml = render_thd_atmos_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "mixed_offset_default_start.mlp",
        |job| {
            job.filter.channel_configuration = Some("5.1".to_string());
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
    let output = run_rendered_xml(&temp, "mixed_offset_default_start.xml", &xml);
    assert_success(&output, "thd_atmos_wav_list offset/ffoa with default start");
    assert_output_exists(
        &temp.path().join("out/mixed_offset_default_start.mlp"),
        "thd_atmos_wav_list offset/ffoa with default start",
    );
}
