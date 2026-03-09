use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use dee_config_gen::{
    ResolveOptions,
    config::{EncodeMode, FilterOverrides, JobFile},
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

fn write_text(path: &Path, content: &str) {
    fs::write(path, content)
        .unwrap_or_else(|err| panic!("failed to write {}: {err}", path.display()));
}

fn run_dee(xml_path: &Path, log_path: &Path) -> Output {
    Command::new("dee")
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
    F: FnOnce(&mut JobFile),
{
    let root = repo_root();
    let mut job = load_job_file(&root.join("examples").join(example_name))
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
    let base = base_xml.replace(
        "\n          <surround_trim_7_1>auto</surround_trim_7_1>",
        "",
    );

    vec![
        ("baseline", base.clone()),
        (
            "no_backend",
            base.replace(
                "\n        <encoding_backend>atmosprocessor</encoding_backend>",
                "",
            )
            .replace("baseline.ec3", "no_backend.ec3"),
        ),
        (
            "backend_pe",
            base.replace(
                "<encoding_backend>atmosprocessor</encoding_backend>",
                "<encoding_backend>PE</encoding_backend>",
            )
            .replace("baseline.ec3", "backend_pe.ec3"),
        ),
        (
            "no_encoder_mode",
            base.replace("\n        <encoder_mode>bluray</encoder_mode>", "")
                .replace("baseline.ec3", "no_encoder_mode.ec3"),
        ),
        (
            "no_both",
            base.replace(
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

fn replace_surround_trim_7_1_with_unknown(xml: &str) -> String {
    xml.replace(
        "<surround_trim_7_1>auto</surround_trim_7_1>",
        "<surround_trim_9_1>auto</surround_trim_9_1>",
    )
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

        let unknown_xml = replace_surround_trim_7_1_with_unknown(&replace_output_name(
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

    let bluray_cases = [
        (
            768_u16,
            Some("DD+JOC: min data rate is 1152 for Blu-ray mode."),
        ),
        (
            1024_u16,
            Some("DD+JOC: min data rate is 1152 for Blu-ray mode."),
        ),
        (1152_u16, None),
        (1280_u16, None),
        (1408_u16, None),
        (1512_u16, None),
        (1536_u16, None),
        (1664_u16, None),
    ];

    for (bitrate, expected_failure) in bluray_cases {
        let output_name = format!("bluray_{bitrate}.ec3");
        let xml = render_atmos_bitrate_xml(&temp, EncodeMode::Bluray, bitrate, &output_name);
        let xml_path = temp.path().join(format!("bluray_{bitrate}.xml"));
        let log_path = temp.path().join(format!("bluray_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        if let Some(needle) = expected_failure {
            assert_failure_contains(&output, needle, &format!("atmos bluray bitrate={bitrate}"));
        } else {
            assert_success(&output, &format!("atmos bluray bitrate={bitrate}"));
            assert_output_exists(
                &temp.path().join("out").join(&output_name),
                &format!("atmos bluray bitrate={bitrate}"),
            );
        }
    }
}
