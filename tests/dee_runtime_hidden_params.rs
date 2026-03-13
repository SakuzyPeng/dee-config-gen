use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{EncodeMode, FilterOverrides, JobSpec},
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

fn assert_failure_contains(output: &Output, needle: &str, context: &str) {
    assert!(
        !output.status.success(),
        "{context} should fail but exited successfully"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");
    assert!(
        combined.contains(needle),
        "{context} should contain '{needle}', got:\n{combined}"
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

fn render_atmos_xml<F>(temp: &TempDir, example_name: &str, output_name: &str, mutate: F) -> String
where
    F: FnOnce(&mut JobSpec),
{
    let root = repo_root();
    let mut job = read_job(&root.join("examples").join(example_name))
        .unwrap_or_else(|err| panic!("load example {example_name}: {err}"));
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
    .expect("resolve atmos job");

    render_xml(&resolved)
}

fn render_atmos_bluray_xml(temp: &TempDir) -> String {
    render_atmos_xml(temp, "atmos_ec3_single.bluray.yaml", "baseline.ec3", |_| {})
}

fn render_atmos_bitrate_xml(
    temp: &TempDir,
    mode: EncodeMode,
    bitrate: u16,
    output_name: &str,
) -> String {
    let example_name = match mode {
        EncodeMode::Streaming => "atmos_ec3_single.streaming.yaml",
        EncodeMode::Dd | EncodeMode::Ddp => panic!("atmos_ec3_v1 does not support dd/ddp"),
        EncodeMode::Mlp => panic!("atmos_ec3_v1 does not support mlp"),
        EncodeMode::Bluray => "atmos_ec3_single.bluray.yaml",
        EncodeMode::Ddp71 => panic!("atmos_ec3_v1 no longer models ddp71"),
    };

    render_atmos_xml(temp, example_name, output_name, |job| {
        job.encode_mode = mode;
        job.filter = FilterOverrides::default();
        job.filter.data_rate = Some(bitrate);
    })
}

fn make_atmos_variants(base_xml: &str) -> Vec<(&'static str, String)> {
    vec![
        ("baseline", base_xml.to_string()),
        (
            "no_backend",
            base_xml
                .replace(
                    "\n        <encoding_backend>atmosprocessor</encoding_backend>",
                    "",
                )
                .replace("baseline.ec3", "no_backend.ec3"),
        ),
        (
            "backend_pe",
            base_xml
                .replace(
                    "<encoding_backend>atmosprocessor</encoding_backend>",
                    "<encoding_backend>PE</encoding_backend>",
                )
                .replace("baseline.ec3", "backend_pe.ec3"),
        ),
        (
            "no_encoder_mode",
            base_xml
                .replace("\n        <encoder_mode>bluray</encoder_mode>", "")
                .replace("baseline.ec3", "no_encoder_mode.ec3"),
        ),
        (
            "no_both",
            base_xml
                .replace(
                    "\n        <encoding_backend>atmosprocessor</encoding_backend>",
                    "",
                )
                .replace("\n        <encoder_mode>bluray</encoder_mode>", "")
                .replace("baseline.ec3", "no_both.ec3"),
        ),
    ]
}

fn replace_output_name(xml: &str, from: &str, to: &str) -> String {
    xml.replace(from, to)
}

fn replace_xml_value(xml: &str, from: &str, to: &str) -> String {
    assert!(
        xml.contains(from),
        "expected XML snippet '{from}' to exist before replacement"
    );
    xml.replace(from, to)
}

fn replace_surround_trim_5_1_with_unknown(xml: &str) -> String {
    xml.replace(
        "<surround_trim_5_1>auto</surround_trim_5_1>",
        "<surround_trim_9_1>auto</surround_trim_9_1>",
    )
}

fn replace_preferred_downmix_mode(xml: &str, value: &str) -> String {
    xml.replace(
        "<preferred_downmix_mode>loro</preferred_downmix_mode>",
        &format!("<preferred_downmix_mode>{value}</preferred_downmix_mode>"),
    )
}

fn inject_after_preferred_downmix(xml: &str, extra: &str) -> String {
    xml.replace(
        "</preferred_downmix_mode>",
        &format!("</preferred_downmix_mode>{extra}"),
    )
}

fn atmos_example_for_mode(mode: &str) -> (&'static str, &'static str) {
    match mode {
        "streaming" => ("atmos_ec3_single.streaming.yaml", "streaming.ec3"),
        "bluray" => ("atmos_ec3_single.bluray.yaml", "bluray.ec3"),
        other => panic!("unsupported atmos mode: {other}"),
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn atmos_bluray_backend_variants_match_or_fail_as_expected() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_bluray_xml(&temp);
    let variants = make_atmos_variants(&base_xml);

    for (name, xml) in &variants {
        write_text(&temp.path().join(format!("{name}.xml")), xml);
    }

    let baseline = run_dee(
        &temp.path().join("baseline.xml"),
        &temp.path().join("baseline.log"),
    );
    assert_success(&baseline, "atmos baseline");

    let no_backend = run_dee(
        &temp.path().join("no_backend.xml"),
        &temp.path().join("no_backend.log"),
    );
    assert_success(&no_backend, "atmos no_backend");
    let no_backend_stdout = String::from_utf8_lossy(&no_backend.stdout);
    assert!(
        no_backend_stdout.contains("Using atmosprocessor backend"),
        "expected no_backend variant to normalize backend, got:\n{no_backend_stdout}",
    );

    let backend_pe = run_dee(
        &temp.path().join("backend_pe.xml"),
        &temp.path().join("backend_pe.log"),
    );
    assert_success(&backend_pe, "atmos backend_pe");
    let backend_pe_stdout = String::from_utf8_lossy(&backend_pe.stdout);
    assert!(
        backend_pe_stdout.contains("Using atmosprocessor backend"),
        "expected backend_pe variant to normalize backend, got:\n{backend_pe_stdout}",
    );

    let no_encoder_mode = run_dee(
        &temp.path().join("no_encoder_mode.xml"),
        &temp.path().join("no_encoder_mode.log"),
    );
    assert_failure_contains(
        &no_encoder_mode,
        "Invalid data_rate value: 1664",
        "atmos no_encoder_mode",
    );

    let no_both = run_dee(
        &temp.path().join("no_both.xml"),
        &temp.path().join("no_both.log"),
    );
    assert_failure_contains(&no_both, "Invalid data_rate value: 1664", "atmos no_both");

    let baseline_bytes =
        fs::read(temp.path().join("out/baseline.ec3")).expect("read baseline output");
    let no_backend_bytes =
        fs::read(temp.path().join("out/no_backend.ec3")).expect("read no_backend output");
    let backend_pe_bytes =
        fs::read(temp.path().join("out/backend_pe.ec3")).expect("read backend_pe output");

    assert_eq!(baseline_bytes, no_backend_bytes);
    assert_eq!(baseline_bytes, backend_pe_bytes);
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_mode_baselines_and_unknown_trim_matrix() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let cases = [
        (
            "streaming",
            render_atmos_xml(
                &temp,
                "atmos_ec3_single.streaming.yaml",
                "streaming_base.ec3",
                |_| {},
            ),
            "streaming_base.ec3",
            "streaming_unknown_trim.ec3",
            None,
        ),
        (
            "bluray",
            render_atmos_xml(
                &temp,
                "atmos_ec3_single.bluray.yaml",
                "bluray_base.ec3",
                |_| {},
            ),
            "bluray_base.ec3",
            "bluray_unknown_trim.ec3",
            None,
        ),
    ];

    for (mode, baseline_xml, baseline_output, unknown_output, expected_failure) in cases {
        let baseline_xml_path = temp.path().join(format!("{mode}_baseline.xml"));
        let baseline_log_path = temp.path().join(format!("{mode}_baseline.log"));
        write_text(&baseline_xml_path, &baseline_xml);

        let baseline = run_dee(&baseline_xml_path, &baseline_log_path);
        if let Some(needle) = expected_failure {
            assert_failure_contains(&baseline, needle, &format!("atmos {mode} baseline"));
            continue;
        }

        assert_success(&baseline, &format!("atmos {mode} baseline"));
        let baseline_output_path = temp.path().join("out").join(baseline_output);
        assert_output_exists(&baseline_output_path, &format!("atmos {mode} baseline"));

        let unknown_xml = replace_surround_trim_5_1_with_unknown(&replace_output_name(
            &baseline_xml,
            baseline_output,
            unknown_output,
        ));
        let unknown_xml_path = temp.path().join(format!("{mode}_unknown_trim.xml"));
        let unknown_log_path = temp.path().join(format!("{mode}_unknown_trim.log"));
        write_text(&unknown_xml_path, &unknown_xml);

        let unknown = run_dee(&unknown_xml_path, &unknown_log_path);
        assert_success(&unknown, &format!("atmos {mode} unknown trim"));
        let unknown_output_path = temp.path().join("out").join(unknown_output);
        assert_output_exists(&unknown_output_path, &format!("atmos {mode} unknown trim"));

        let baseline_bytes = fs::read(&baseline_output_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", baseline_output_path.display()));
        let unknown_bytes = fs::read(&unknown_output_path)
            .unwrap_or_else(|err| panic!("read {}: {err}", unknown_output_path.display()));
        assert_eq!(
            baseline_bytes, unknown_bytes,
            "atmos {mode} unknown trim should be ignored and keep byte-identical output"
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_preferred_downmix_mode_runtime_matrix_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let cases = [
        ("streaming", "loro", None),
        ("streaming", "ltrt", None),
        ("streaming", "ltrt-pl2", None),
        ("streaming", "not_indicated", None),
        ("bluray", "loro", None),
        ("bluray", "ltrt", None),
        (
            "bluray",
            "ltrt-pl2",
            Some("Preferred Downmix mode Pro Logic II is not supported in Blu-ray Mode"),
        ),
        ("bluray", "not_indicated", None),
    ];

    for (mode, preferred_downmix_mode, expected_failure) in cases {
        let (example_name, output_name) = match mode {
            "streaming" => ("atmos_ec3_single.streaming.yaml", "preferred_streaming.ec3"),
            "bluray" => ("atmos_ec3_single.bluray.yaml", "preferred_bluray.ec3"),
            other => panic!("unsupported atmos mode: {other}"),
        };

        let xml = replace_preferred_downmix_mode(
            &render_atmos_xml(&temp, example_name, output_name, |_| {}),
            preferred_downmix_mode,
        );
        let xml_path = temp
            .path()
            .join(format!("preferred_{mode}_{preferred_downmix_mode}.xml"));
        let log_path = temp
            .path()
            .join(format!("preferred_{mode}_{preferred_downmix_mode}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        let context =
            format!("atmos preferred_downmix_mode mode={mode} value={preferred_downmix_mode}");
        match expected_failure {
            Some(needle) => assert_failure_contains(&output, needle, &context),
            None => assert_success(&output, &context),
        }
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_loudness_runtime_matrix_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    for mode in ["streaming", "bluray"] {
        let (example_name, output_name) = atmos_example_for_mode(mode);
        let base_xml = render_atmos_xml(&temp, example_name, output_name, |_| {});

        for metering_mode in ["1770-4", "1770-3", "LeqA"] {
            let xml = replace_xml_value(
                &base_xml,
                "<metering_mode>1770-4</metering_mode>",
                &format!("<metering_mode>{metering_mode}</metering_mode>"),
            );
            let xml_path = temp
                .path()
                .join(format!("loudness_{mode}_metering_{metering_mode}.xml"));
            let log_path = temp
                .path()
                .join(format!("loudness_{mode}_metering_{metering_mode}.log"));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!("atmos loudness metering_mode={metering_mode} mode={mode}"),
            );
        }

        for dialogue_intelligence in ["true", "false"] {
            let xml = replace_xml_value(
                &base_xml,
                "<dialogue_intelligence>true</dialogue_intelligence>",
                &format!("<dialogue_intelligence>{dialogue_intelligence}</dialogue_intelligence>"),
            );
            let xml_path = temp.path().join(format!(
                "loudness_{mode}_dialogue_{dialogue_intelligence}.xml"
            ));
            let log_path = temp.path().join(format!(
                "loudness_{mode}_dialogue_{dialogue_intelligence}.log"
            ));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!(
                    "atmos loudness dialogue_intelligence={dialogue_intelligence} mode={mode}"
                ),
            );
        }

        for speech_threshold in ["0", "100"] {
            let xml = replace_xml_value(
                &base_xml,
                "<speech_threshold>15</speech_threshold>",
                &format!("<speech_threshold>{speech_threshold}</speech_threshold>"),
            );
            let xml_path = temp.path().join(format!(
                "loudness_{mode}_speech_threshold_{speech_threshold}.xml"
            ));
            let log_path = temp.path().join(format!(
                "loudness_{mode}_speech_threshold_{speech_threshold}.log"
            ));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!("atmos loudness speech_threshold={speech_threshold} mode={mode}"),
            );
        }
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_streaming_timecode_frame_rate_runtime_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.streaming.yaml",
        "streaming_timecode.ec3",
        |_| {},
    );
    for timecode_frame_rate in ["not_indicated", "23.976"] {
        let xml = replace_xml_value(
            &base_xml,
            "<timecode_frame_rate>not_indicated</timecode_frame_rate>",
            &format!("<timecode_frame_rate>{timecode_frame_rate}</timecode_frame_rate>"),
        );
        let xml_path = temp.path().join(format!(
            "timecode_frame_rate_streaming_{}.xml",
            timecode_frame_rate.replace('.', "_")
        ));
        let log_path = temp.path().join(format!(
            "timecode_frame_rate_streaming_{}.log",
            timecode_frame_rate.replace('.', "_")
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos timecode_frame_rate={timecode_frame_rate} mode=streaming"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_bluray_timecode_frame_rate_runtime_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.bluray.yaml",
        "bluray_timecode.ec3",
        |_| {},
    );
    for timecode_frame_rate in ["not_indicated", "23.976"] {
        let xml = replace_xml_value(
            &base_xml,
            "<timecode_frame_rate>not_indicated</timecode_frame_rate>",
            &format!("<timecode_frame_rate>{timecode_frame_rate}</timecode_frame_rate>"),
        );
        let xml_path = temp.path().join(format!(
            "timecode_frame_rate_bluray_{}.xml",
            timecode_frame_rate.replace('.', "_")
        ));
        let log_path = temp.path().join(format!(
            "timecode_frame_rate_bluray_{}.log",
            timecode_frame_rate.replace('.', "_")
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos timecode_frame_rate={timecode_frame_rate} mode=bluray"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_time_base_runtime_matrix_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    for mode in ["streaming", "bluray"] {
        let (example_name, output_name) = atmos_example_for_mode(mode);
        let base_xml = render_atmos_xml(&temp, example_name, output_name, |_| {});
        let from = if mode == "bluray" {
            "<time_base>embedded_timecode</time_base>"
        } else {
            "<time_base>file_position</time_base>"
        };
        let alternative = if mode == "bluray" {
            "file_position"
        } else {
            "embedded_timecode"
        };
        let xml = replace_xml_value(
            &base_xml,
            from,
            &format!("<time_base>{alternative}</time_base>"),
        );
        let xml_path = temp
            .path()
            .join(format!("time_base_{mode}_{alternative}.xml"));
        let log_path = temp
            .path()
            .join(format!("time_base_{mode}_{alternative}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos time_base={alternative} mode={mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_streaming_trim_runtime_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.streaming.yaml",
        "streaming_trim.ec3",
        |_| {},
    );

    for surround_trim in ["auto", "-3"] {
        let xml = replace_xml_value(
            &base_xml,
            "<surround_trim_5_1>auto</surround_trim_5_1>",
            &format!("<surround_trim_5_1>{surround_trim}</surround_trim_5_1>"),
        );
        let xml_path = temp.path().join(format!(
            "trim_streaming_surround_{}.xml",
            surround_trim.replace('-', "neg")
        ));
        let log_path = temp.path().join(format!(
            "trim_streaming_surround_{}.log",
            surround_trim.replace('-', "neg")
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos surround_trim_5_1={surround_trim} mode=streaming"),
        );
    }

    let invalid_height_xml = replace_xml_value(
        &base_xml,
        "<height_trim_5_1>auto</height_trim_5_1>",
        "<height_trim_5_1>bogus</height_trim_5_1>",
    );
    let invalid_height_xml_path = temp.path().join("trim_streaming_height_invalid.xml");
    let invalid_height_log_path = temp.path().join("trim_streaming_height_invalid.log");
    write_text(&invalid_height_xml_path, &invalid_height_xml);
    let invalid_height = run_dee(&invalid_height_xml_path, &invalid_height_log_path);
    assert_failure_contains(
        &invalid_height,
        "height_trim_5_1",
        "invalid atmos height_trim_5_1 mode=streaming",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_bluray_trim_runtime_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.bluray.yaml",
        "bluray_trim.ec3",
        |_| {},
    );

    for height_trim in ["auto", "-6"] {
        let xml = replace_xml_value(
            &base_xml,
            "<height_trim_5_1>auto</height_trim_5_1>",
            &format!("<height_trim_5_1>{height_trim}</height_trim_5_1>"),
        );
        let xml_path = temp.path().join(format!(
            "trim_bluray_height_{}.xml",
            height_trim.replace('-', "neg")
        ));
        let log_path = temp.path().join(format!(
            "trim_bluray_height_{}.log",
            height_trim.replace('-', "neg")
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos height_trim_5_1={height_trim} mode=bluray"),
        );
    }

    let invalid_surround_xml = replace_xml_value(
        &base_xml,
        "<surround_trim_5_1>auto</surround_trim_5_1>",
        "<surround_trim_5_1>bogus</surround_trim_5_1>",
    );
    let invalid_surround_xml_path = temp.path().join("trim_bluray_surround_invalid.xml");
    let invalid_surround_log_path = temp.path().join("trim_bluray_surround_invalid.log");
    write_text(&invalid_surround_xml_path, &invalid_surround_xml);
    let invalid_surround = run_dee(&invalid_surround_xml_path, &invalid_surround_log_path);
    assert_failure_contains(
        &invalid_surround,
        "surround_trim_5_1",
        "invalid atmos surround_trim_5_1 mode=bluray",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_representative_misc_smoke_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let streaming_base = render_atmos_xml(
        &temp,
        "atmos_ec3_single.streaming.yaml",
        "streaming_misc.ec3",
        |_| {},
    );
    let bluray_base = render_atmos_xml(
        &temp,
        "atmos_ec3_single.bluray.yaml",
        "bluray_misc.ec3",
        |_| {},
    );

    let streaming_cases = [
        (
            "start_alt",
            replace_xml_value(
                &streaming_base,
                "<start>first_frame_of_action</start>",
                "<start>00:00:00.0</start>",
            ),
        ),
        (
            "end_alt",
            replace_xml_value(
                &streaming_base,
                "<end>end_of_file</end>",
                "<end>00:00:01.0</end>",
            ),
        ),
        (
            "prepend_silence_alt",
            replace_xml_value(
                &streaming_base,
                "<prepend_silence_duration>0.0</prepend_silence_duration>",
                "<prepend_silence_duration>0.005333</prepend_silence_duration>",
            ),
        ),
        (
            "append_silence_alt",
            replace_xml_value(
                &streaming_base,
                "<append_silence_duration>0.0</append_silence_duration>",
                "<append_silence_duration>0.005333</append_silence_duration>",
            ),
        ),
        (
            "line_mode_drc_alt",
            replace_xml_value(
                &streaming_base,
                "<line_mode_drc_profile>film_light</line_mode_drc_profile>",
                "<line_mode_drc_profile>speech</line_mode_drc_profile>",
            ),
        ),
        (
            "rf_mode_drc_alt",
            replace_xml_value(
                &streaming_base,
                "<rf_mode_drc_profile>film_light</rf_mode_drc_profile>",
                "<rf_mode_drc_profile>speech</rf_mode_drc_profile>",
            ),
        ),
        (
            "custom_dialnorm_alt",
            replace_xml_value(
                &streaming_base,
                "<custom_dialnorm>0</custom_dialnorm>",
                "<custom_dialnorm>-31</custom_dialnorm>",
            ),
        ),
    ];

    for (name, xml) in streaming_cases {
        let xml_path = temp.path().join(format!("streaming_misc_{name}.xml"));
        let log_path = temp.path().join(format!("streaming_misc_{name}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos streaming representative smoke {name}"),
        );
    }

    let bluray_cases = [
        (
            "prepend_silence_alt",
            replace_xml_value(
                &bluray_base,
                "<prepend_silence_duration>0f</prepend_silence_duration>",
                "<prepend_silence_duration>1f</prepend_silence_duration>",
            ),
        ),
        (
            "append_silence_alt",
            replace_xml_value(
                &bluray_base,
                "<append_silence_duration>0f</append_silence_duration>",
                "<append_silence_duration>1f</append_silence_duration>",
            ),
        ),
        (
            "line_mode_drc_alt",
            replace_xml_value(
                &bluray_base,
                "<line_mode_drc_profile>film_light</line_mode_drc_profile>",
                "<line_mode_drc_profile>speech</line_mode_drc_profile>",
            ),
        ),
        (
            "rf_mode_drc_alt",
            replace_xml_value(
                &bluray_base,
                "<rf_mode_drc_profile>film_light</rf_mode_drc_profile>",
                "<rf_mode_drc_profile>speech</rf_mode_drc_profile>",
            ),
        ),
        (
            "custom_dialnorm_alt",
            replace_xml_value(
                &bluray_base,
                "<custom_dialnorm>0</custom_dialnorm>",
                "<custom_dialnorm>-31</custom_dialnorm>",
            ),
        ),
    ];

    for (name, xml) in bluray_cases {
        let xml_path = temp.path().join(format!("bluray_misc_{name}.xml"));
        let log_path = temp.path().join(format!("bluray_misc_{name}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("atmos bluray representative smoke {name}"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_silence_duration_format_gap_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.streaming.yaml",
        "streaming_silence_gap.ec3",
        |_| {},
    );

    let cases = [
        (
            "prepend",
            replace_xml_value(
                &base_xml,
                "<prepend_silence_duration>0.0</prepend_silence_duration>",
                "<prepend_silence_duration>0:00:00.005333</prepend_silence_duration>",
            ),
            "prepend_silence_duration",
        ),
        (
            "append",
            replace_xml_value(
                &base_xml,
                "<append_silence_duration>0.0</append_silence_duration>",
                "<append_silence_duration>0:00:00.005333</append_silence_duration>",
            ),
            "append_silence_duration",
        ),
    ];

    for (name, xml, needle) in cases {
        let xml_path = temp
            .path()
            .join(format!("streaming_silence_gap_{name}.xml"));
        let log_path = temp
            .path()
            .join(format!("streaming_silence_gap_{name}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(
            &output,
            needle,
            &format!("atmos streaming silence duration format gap {name}"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_bluray_start_before_embedded_ffoa_is_rejected() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let base_xml = render_atmos_xml(
        &temp,
        "atmos_ec3_single.bluray.yaml",
        "bluray_start_gap.ec3",
        |_| {},
    );
    let xml = replace_xml_value(
        &base_xml,
        "<start>first_frame_of_action</start>",
        "<start>00:00:00:00</start>",
    );
    let xml_path = temp.path().join("bluray_start_gap.xml");
    let log_path = temp.path().join("bluray_start_gap.log");
    write_text(&xml_path, &xml);
    let output = run_dee(&xml_path, &log_path);
    assert_failure_contains(
        &output,
        "Start time value before embedded file start",
        "atmos bluray start before embedded FFOA should fail",
    );
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_unsupported_pcm_metadata_knobs_are_rejected() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let cases = [
        (
            "streaming",
            "atmos_ec3_single.streaming.yaml",
            "streaming_meta.ec3",
            "<dolby_surround_mode>yes</dolby_surround_mode>",
            "Unknown property: downmix:dolby_surround_mode.",
        ),
        (
            "streaming",
            "atmos_ec3_single.streaming.yaml",
            "streaming_meta_ex.ec3",
            "<dolby_surround_ex_mode>yes</dolby_surround_ex_mode>",
            "Unknown property: downmix:dolby_surround_ex_mode.",
        ),
        (
            "bluray",
            "atmos_ec3_single.bluray.yaml",
            "bluray_meta.ec3",
            "<dolby_surround_mode>yes</dolby_surround_mode>",
            "Unknown property: downmix:dolby_surround_mode.",
        ),
        (
            "bluray",
            "atmos_ec3_single.bluray.yaml",
            "bluray_meta_ex.ec3",
            "<dolby_surround_ex_mode>yes</dolby_surround_ex_mode>",
            "Unknown property: downmix:dolby_surround_ex_mode.",
        ),
    ];

    for (mode, example_name, output_name, extra_property, needle) in cases {
        let xml = inject_after_preferred_downmix(
            &render_atmos_xml(&temp, example_name, output_name, |_| {}),
            extra_property,
        );
        let xml_path = temp.path().join(format!(
            "unsupported_{}_{}.xml",
            mode,
            extra_property
                .trim_matches(|c| c == '<' || c == '>')
                .split('>')
                .next()
                .unwrap_or("property")
                .replace('/', "_")
        ));
        let log_path = temp.path().join(format!(
            "unsupported_{}_{}.log",
            mode,
            extra_property
                .trim_matches(|c| c == '<' || c == '>')
                .split('>')
                .next()
                .unwrap_or("property")
                .replace('/', "_")
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(
            &output,
            needle,
            &format!("atmos unsupported metadata knob mode={mode} property={extra_property}"),
        );
    }
}

#[test]
#[ignore = "requires local dee runtime"]
fn atmos_bitrate_matrix_matches_runtime() {
    require_command("dee");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");

    let streaming_cases = [384_u16, 448, 576, 640, 768, 1024];
    for bitrate in streaming_cases {
        let output_name = format!("streaming_{bitrate}.ec3");
        let xml = render_atmos_bitrate_xml(&temp, EncodeMode::Streaming, bitrate, &output_name);
        let xml_path = temp.path().join(format!("streaming_{bitrate}.xml"));
        let log_path = temp.path().join(format!("streaming_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("atmos streaming bitrate={bitrate}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("atmos streaming bitrate={bitrate}"),
        );
    }

    let bluray_cases = [1152_u16, 1280, 1408, 1512, 1536, 1664];

    for bitrate in bluray_cases {
        let output_name = format!("bluray_{bitrate}.ec3");
        let xml = render_atmos_bitrate_xml(&temp, EncodeMode::Bluray, bitrate, &output_name);
        let xml_path = temp.path().join(format!("bluray_{bitrate}.xml"));
        let log_path = temp.path().join(format!("bluray_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("atmos bluray bitrate={bitrate}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("atmos bluray bitrate={bitrate}"),
        );
    }
}
