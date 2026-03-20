mod common;

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, JobSpec, Profile},
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

fn require_native_mp4muxer() -> PathBuf {
    let mut candidates = Vec::new();
    if let Some(explicit) = std::env::var_os("MP4MUXER_PATH") {
        candidates.push(PathBuf::from(explicit));
    }
    let repo = repo_root();
    if let Some(parent) = repo.parent().and_then(|p| p.parent()) {
        candidates.push(parent.join("upstream/dlb_mp4base/bin/mp4muxer_mac"));
    }
    candidates.push(PathBuf::from(
        "/Applications/FANTASONIC TOOLBOX.app/Contents/Resources/mp4muxer_mac",
    ));

    for path in candidates {
        if path.exists() {
            let candidate = path.to_string_lossy().to_string();
            let status = Command::new("sh")
                .arg("-lc")
                .arg(format!("test -x {}", shell_escape(&candidate)))
                .status()
                .unwrap_or_else(|err| {
                    panic!("failed to probe mp4muxer candidate '{candidate}': {err}")
                });
            if status.success() {
                return path;
            }
        }
    }

    panic!("required native mp4muxer was not found");
}

fn shell_escape(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
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
    let context = format!("dee thd {}", xml_path.display());
    common::run_dee_command(command, &context)
}

fn run_native_mp4muxer(input_path: &Path, output_path: &Path) -> Output {
    let mp4muxer = require_native_mp4muxer();
    Command::new(mp4muxer)
        .arg("-i")
        .arg(input_path)
        .arg("-o")
        .arg(output_path)
        .arg("--overwrite")
        .output()
        .unwrap_or_else(|err| {
            panic!(
                "failed to run native mp4muxer for {}: {err}",
                input_path.display()
            )
        })
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

fn render_thd_xml(temp: &TempDir, output_name: &str, mutate: impl FnOnce(&mut JobSpec)) -> String {
    let root = repo_root();
    let mut job = read_job(&root.join("examples/thd_single.mlp.yaml"))
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

fn run_rendered_xml(temp: &TempDir, file_name: &str, xml: &str) -> Output {
    let xml_path = temp.path().join(file_name);
    let log_path = temp.path().join(format!("{file_name}.log"));
    write_text(&xml_path, xml);
    run_dee(&xml_path, &log_path)
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_mlp_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_xml(&temp, "smoke.mlp", |_| {});
    let output = run_rendered_xml(&temp, "smoke.xml", &xml);
    assert_success(&output, "thd mlp smoke");
    assert_output_exists(&temp.path().join("out/smoke.mlp"), "thd mlp smoke");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_loudness_timecode_representative_params_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

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
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "representative.xml", &xml);

    assert_success(&output, "thd loudness/timecode representative params");
    assert_output_exists(
        &temp.path().join("out/representative.mlp"),
        "thd loudness/timecode representative params",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_drc_profiles_representative_values_match_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_xml(&temp, "drc_profiles.mlp", |job| {
        job.filter = FilterOverrides {
            atmos_presentation_drc_profile: Some("speech".to_string()),
            presentation_8ch_drc_profile: Some("film_standard".to_string()),
            presentation_6ch_drc_profile: Some("music_light".to_string()),
            presentation_2ch_drc_profile: Some("music_standard".to_string()),
            ..FilterOverrides::default()
        };
    });
    let output = run_rendered_xml(&temp, "drc_profiles.xml", &xml);

    assert_success(&output, "thd drc profile representative values");
    assert_output_exists(
        &temp.path().join("out/drc_profiles.mlp"),
        "thd drc profile representative values",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_silence_duration_runtime_matrix_matches_runtime() {
    require_command("dee");

    for (field, value) in [
        ("prepend_silence_duration", "0"),
        ("prepend_silence_duration", "0.005333"),
        ("prepend_silence_duration", "1.25"),
        ("append_silence_duration", "0"),
        ("append_silence_duration", "0.005333"),
        ("append_silence_duration", "1.25"),
    ] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("{field}_{value}.mlp"), |job| match field {
            "prepend_silence_duration" => {
                job.filter.prepend_silence_duration = Some(value.to_string());
            }
            "append_silence_duration" => {
                job.filter.append_silence_duration = Some(value.to_string());
            }
            _ => unreachable!(),
        });
        let output = run_rendered_xml(&temp, &format!("{field}_{value}.xml"), &xml);
        assert_success(&output, &format!("thd {field}={value}"));
    }

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let xml = render_thd_xml(&temp, "invalid_prepend.mlp", |_| {});
    let xml = replace_leaf_value(&xml, "prepend_silence_duration", "bogus");
    let output = run_rendered_xml(&temp, "invalid_prepend.xml", &xml);
    assert_failure_contains(
        &output,
        "thd prepend_silence_duration=bogus",
        "prepend_silence_duration",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_metering_mode_runtime_matrix_matches_runtime() {
    require_command("dee");

    for value in ["1770-1", "1770-3", "1770-4", "LeqA"] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("metering_{value}.mlp"), |job| {
            job.filter.metering_mode = Some(value.to_string());
        });
        let output = run_rendered_xml(&temp, &format!("metering_{value}.xml"), &xml);
        assert_success(&output, &format!("thd metering_mode={value}"));
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_timecode_boundary_runtime_matrix_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let xml = render_thd_xml(&temp, "file_position_explicit.mlp", |job| {
        job.filter.timecode_frame_rate = Some("23.976".to_string());
        job.filter.time_base = Some("file_position".to_string());
        job.filter.start = Some("00:00:00:00".to_string());
        job.filter.end = Some("00:00:01:00".to_string());
    });
    let output = run_rendered_xml(&temp, "file_position_explicit.xml", &xml);
    assert_success(&output, "thd file_position explicit timecode");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let xml = render_thd_xml(&temp, "embedded_defaults.mlp", |job| {
        job.filter.timecode_frame_rate = Some("23.976".to_string());
        job.filter.time_base = Some("embedded_timecode".to_string());
        job.filter.start = Some("first_frame_of_action".to_string());
        job.filter.end = Some("end_of_file".to_string());
    });
    let output = run_rendered_xml(&temp, "embedded_defaults.xml", &xml);
    assert_success(&output, "thd embedded_timecode with symbolic boundaries");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_fixed_fields_runtime_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_xml(&temp, "fixed_fields.mlp", |_| {});
    for needle in [
        "<spatial_clusters>12</spatial_clusters>",
        "<legacy_authoring_compatibility>true</legacy_authoring_compatibility>",
        "<surround_3db_attenuation>true</surround_3db_attenuation>",
        "<drc_default_on>true</drc_default_on>",
        "<format>stereo</format>",
        "<optimize_data_rate>false</optimize_data_rate>",
        "<starting_timecode>off</starting_timecode>",
        "<frame_rate>auto</frame_rate>",
        "<log_format>txt</log_format>",
    ] {
        assert!(
            xml.contains(needle),
            "thd fixed-fields xml should contain {needle}"
        );
    }

    let output = run_rendered_xml(&temp, "fixed_fields.xml", &xml);
    assert_success(&output, "thd fixed-field defaults");
    assert_output_exists(
        &temp.path().join("out/fixed_fields.mlp"),
        "thd fixed-field defaults",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_spatial_clusters_runtime_matrix_matches_runtime() {
    require_command("dee");

    for value in ["12", "14", "16"] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("spatial_clusters_{value}.mlp"), |job| {
            job.filter.spatial_clusters = Some(value.to_string());
        });
        let output = run_rendered_xml(&temp, &format!("spatial_clusters_{value}.xml"), &xml);
        assert_success(&output, &format!("thd spatial_clusters={value}"));
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_legacy_authoring_compatibility_runtime_matrix_matches_runtime() {
    require_command("dee");

    for value in [true, false] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("legacy_authoring_{value}.mlp"), |job| {
            job.filter.legacy_authoring_compatibility = Some(value);
        });
        let output = run_rendered_xml(&temp, &format!("legacy_authoring_{value}.xml"), &xml);
        assert_success(
            &output,
            &format!("thd legacy_authoring_compatibility={value}"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_optimize_data_rate_runtime_matrix_matches_runtime() {
    require_command("dee");

    for value in [true, false] {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("optimize_data_rate_{value}.mlp"), |job| {
            job.filter.optimize_data_rate = Some(value);
        });
        let output = run_rendered_xml(&temp, &format!("optimize_data_rate_{value}.xml"), &xml);
        assert_success(&output, &format!("thd optimize_data_rate={value}"));
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_music_profile_runtime_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_xml(&temp, "music_profile.mlp", |job| {
        job.profile = Profile::Music;
    });
    let output = run_rendered_xml(&temp, "music_profile.xml", &xml);
    assert_success(&output, "thd music profile smoke");
    assert_output_exists(
        &temp.path().join("out/music_profile.mlp"),
        "thd music profile smoke",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_mp4_feasibility_lane_records_runtime_result() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let base_xml = render_thd_xml(&temp, "candidate.mlp", |_| {});
    let xml = base_xml
        .replace(
            "<mlp version=\"1\">\n      <file_name>candidate.mlp</file_name>",
            "<mp4 version=\"1\">\n      <file_name>candidate.mp4</file_name>",
        )
        .replace("</mlp>", "</mp4>");

    let output = run_rendered_xml(&temp, "candidate_mp4.xml", &xml);
    assert_failure_contains(&output, "thd mp4 workflow feasibility", "audio.mlp");
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_native_mp4muxer_rejects_mlp() {
    require_command("dee");

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);

    let xml = render_thd_xml(&temp, "native_mp4muxer_probe.mlp", |_| {});
    let output = run_rendered_xml(&temp, "native_mp4muxer_probe.xml", &xml);
    assert_success(&output, "thd native mp4muxer probe prerequisite");

    let mlp_path = temp.path().join("out/native_mp4muxer_probe.mlp");
    let mp4_path = temp.path().join("out/native_mp4muxer_probe.mp4");
    let mux_output = run_native_mp4muxer(&mlp_path, &mp4_path);

    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&mux_output.stdout),
        String::from_utf8_lossy(&mux_output.stderr)
    );
    assert!(
        combined.contains("extension not supported"),
        "native mp4muxer should reject mlp input, got:\n{combined}"
    );
    assert!(
        !mp4_path.exists()
            || fs::metadata(&mp4_path)
                .unwrap_or_else(|err| panic!("stat {}: {err}", mp4_path.display()))
                .len()
                <= 44,
        "native mp4muxer should not produce a valid mp4 for mlp input"
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn thd_embedded_timecodes_runtime_matrix_matches_runtime() {
    require_command("dee");

    let success_cases = [
        (
            "off_auto_default",
            "off",
            "auto",
            "file_position",
            "00:00:00:00",
            "23.976",
        ),
        (
            "auto_auto",
            "auto",
            "auto",
            "file_position",
            "00:00:00:00",
            "24",
        ),
        (
            "explicit_starting_timecode",
            "00:23:01:00",
            "30",
            "file_position",
            "00:00:00:00",
            "24",
        ),
    ];

    for (name, starting_timecode, frame_rate, time_base, start, timecode_frame_rate) in
        success_cases
    {
        let temp = TempDir::new().expect("temp dir");
        create_temp_layout(&temp);
        let xml = render_thd_xml(&temp, &format!("{name}.mlp"), |job| {
            job.filter.time_base = Some(time_base.to_string());
            job.filter.start = Some(start.to_string());
            job.filter.end = Some("end_of_file".to_string());
            job.filter.timecode_frame_rate = Some(timecode_frame_rate.to_string());
            job.filter.starting_timecode = Some(starting_timecode.to_string());
            job.filter.frame_rate = Some(frame_rate.to_string());
        });
        let output = run_rendered_xml(&temp, &format!("{name}.xml"), &xml);
        assert_success(&output, &format!("thd embedded timecodes {name}"));
        assert_output_exists(
            &temp.path().join(format!("out/{name}.mlp")),
            &format!("thd embedded timecodes {name}"),
        );
    }

    let temp = TempDir::new().expect("temp dir");
    create_temp_layout(&temp);
    let xml = render_thd_xml(&temp, "embedded_invalid_frame_rate.mlp", |job| {
        job.filter.time_base = Some("file_position".to_string());
        job.filter.start = Some("00:00:00:00".to_string());
        job.filter.end = Some("end_of_file".to_string());
        job.filter.timecode_frame_rate = Some("24".to_string());
    });
    let xml = replace_leaf_value_in_parent(&xml, "embedded_timecodes", "starting_timecode", "auto");
    let xml = replace_leaf_value_in_parent(&xml, "embedded_timecodes", "frame_rate", "bogus");
    let output = run_rendered_xml(&temp, "embedded_invalid_frame_rate.xml", &xml);
    assert_failure_contains(&output, "thd embedded frame_rate=bogus", "frame_rate");
}
