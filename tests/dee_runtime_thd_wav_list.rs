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

fn render_thd_wav_list_xml(
    temp: &TempDir,
    input_storage_path: &str,
    input_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/thd_wav_list_single.mlp.yaml"))
        .unwrap_or_else(|err| panic!("load thd_wav_list example: {err}"));
    job.input.storage_path = input_storage_path.to_string();
    job.input.file_names = input_file_names;
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
    .expect("resolve thd_wav_list job");

    render_xml(&resolved)
}

fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

fn remove_tag(xml: &str, tag: &str) -> String {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml
        .find(&open)
        .unwrap_or_else(|| panic!("missing open tag {open}"));
    let end = xml[start..]
        .find(&close)
        .map(|offset| start + offset + close.len())
        .unwrap_or_else(|| panic!("missing close tag {close}"));
    let mut result = String::with_capacity(xml.len());
    result.push_str(&xml[..start]);
    result.push_str(&xml[end..]);
    result
}

fn replace_tag_value(xml: &str, tag: &str, value: &str) -> String {
    let old = extract_tag_value(xml, tag);
    xml.replacen(
        &format!("<{tag}>{old}</{tag}>"),
        &format!("<{tag}>{value}</{tag}>"),
        1,
    )
}

fn extract_tag_value<'a>(xml: &'a str, tag: &str) -> &'a str {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml
        .find(&open)
        .map(|idx| idx + open.len())
        .unwrap_or_else(|| panic!("missing open tag {open}"));
    let end = xml[start..]
        .find(&close)
        .map(|offset| start + offset)
        .unwrap_or_else(|| panic!("missing close tag {close}"));
    &xml[start..end]
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_mlp_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);

    let xml = render_thd_wav_list_xml(&temp, &storage_path, file_names, "wav_list_51.mlp", |_| {});
    let output = run_rendered_xml(&temp, "wav_list_51.xml", &xml);
    assert_success(&output, "thd_wav_list 5.1 smoke");
    assert_output_exists(
        &temp.path().join("out/wav_list_51.mlp"),
        "thd_wav_list 5.1 smoke",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_71_topology_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5, 6, 7]);
    let xml = render_thd_wav_list_xml(&temp, &storage_path, file_names, "wav_list_71.mlp", |_| {});
    assert!(xml.contains("<channel_configuration>7.1</channel_configuration>"));
    let output = run_rendered_xml(&temp, "wav_list_71.xml", &xml);
    assert_success(&output, "thd_wav_list 7.1 topology");
    assert_output_exists(
        &temp.path().join("out/wav_list_71.mlp"),
        "thd_wav_list 7.1 topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_mono_is_rejected_by_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);
    let base_xml = render_thd_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "wav_list_mono.mlp",
        |_| {},
    );
    let mut xml = replace_tag_value(&base_xml, "channel_configuration", "mono");
    for tag in [
        "file_name_R",
        "file_name_C",
        "file_name_LFE",
        "file_name_LS",
        "file_name_RS",
    ] {
        xml = remove_tag(&xml, tag);
    }
    let output = run_rendered_xml(&temp, "wav_list_mono.xml", &xml);
    assert!(
        !output.status.success(),
        "thd_wav_list mono should fail in runtime"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("file_name_C"),
        "unexpected stdout:\n{stdout}"
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_stereo_topology_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) = create_runtime_mono_stems("16ch.wav", &[0, 1]);
    let xml = render_thd_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "wav_list_stereo.mlp",
        |_| {},
    );
    assert!(xml.contains("<channel_configuration>stereo</channel_configuration>"));
    assert!(!xml.contains("<file_name_C>"));
    let output = run_rendered_xml(&temp, "wav_list_stereo.xml", &xml);
    assert_success(&output, "thd_wav_list stereo topology");
    assert_output_exists(
        &temp.path().join("out/wav_list_stereo.mlp"),
        "thd_wav_list stereo topology",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_dash_placeholders_are_rejected_by_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);
    let base_xml = render_thd_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "wav_list_silent.mlp",
        |_| {},
    );
    let xml = replace_tag_value(
        &replace_tag_value(&base_xml, "file_name_C", "-"),
        "file_name_LFE",
        "-",
    );
    let output = run_rendered_xml(&temp, "wav_list_silent.xml", &xml);
    assert!(
        !output.status.success(),
        "thd_wav_list dash placeholders should fail in runtime"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("File"));
    assert!(stdout.contains("\\-"));
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_wav_list_representative_params_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let (_stems_temp, storage_path, file_names) =
        create_runtime_mono_stems("16ch.wav", &[0, 1, 2, 3, 4, 5]);
    let xml = render_thd_wav_list_xml(
        &temp,
        &storage_path,
        file_names,
        "wav_list_representative.mlp",
        |job| {
            job.filter = FilterOverrides {
                channel_configuration: Some("5.1".to_string()),
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
                starting_timecode: Some("auto".to_string()),
                frame_rate: Some("24".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
    let output = run_rendered_xml(&temp, "wav_list_representative.xml", &xml);
    assert_success(&output, "thd_wav_list representative params");
    assert_output_exists(
        &temp.path().join("out/wav_list_representative.mlp"),
        "thd_wav_list representative params",
    );
}
