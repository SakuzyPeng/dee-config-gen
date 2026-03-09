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
        EncodeMode::Streaming | EncodeMode::Ddp71 => "atmos_ec3_single.streaming.yaml",
        EncodeMode::Bluray => "atmos_ec3_single.bluray.yaml",
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
        ("baseline", base.replace("baseline.ec3", "baseline.ec3")),
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

fn generate_pcm_8ch_input(temp: &TempDir) -> PathBuf {
    let root = repo_root();
    let input = root.join("testfiles/16ch.wav");
    let output = temp.path().join("in/8ch.wav");

    let status = Command::new("ffmpeg")
        .args(["-y", "-i", input.to_str().expect("utf-8 input path")])
        .args([
            "-filter_complex",
            "pan=7.1|c0=c0|c1=c1|c2=c2|c3=c3|c4=c6|c5=c7|c6=c4|c7=c5",
        ])
        .args([
            "-c:a",
            "pcm_s24le",
            output.to_str().expect("utf-8 output path"),
        ])
        .status()
        .expect("run ffmpeg");

    assert!(status.success(), "ffmpeg should generate 8ch wav");
    output
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

fn pcm_to_ddp_bluray_xml(temp: &TempDir, output_name: &str, extra_xml: &str) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
<job_config>\n\
  <input><audio><wav version=\"1\"><file_name>8ch.wav</file_name><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa><storage><local><path>Z:{}</path></local></storage></wav></audio></input>\n\
  <filter><audio><pcm_to_ddp version=\"3\"><loudness><measure_only><metering_mode>1770-3</metering_mode><dialogue_intelligence>true</dialogue_intelligence><speech_threshold>15</speech_threshold></measure_only></loudness><encoder_mode>bluray</encoder_mode><bitstream_mode>complete_main</bitstream_mode><downmix_config>off</downmix_config><data_rate>768</data_rate><timecode_frame_rate>not_indicated</timecode_frame_rate><start>first_frame_of_action</start><end>end_of_file</end><time_base>file_position</time_base><prepend_silence_duration>0.0</prepend_silence_duration><append_silence_duration>0.0</append_silence_duration><lfe_on>true</lfe_on><dolby_surround_mode>not_indicated</dolby_surround_mode><dolby_surround_ex_mode>no</dolby_surround_ex_mode><user_data>-1</user_data><drc><line_mode_drc_profile>film_light</line_mode_drc_profile><rf_mode_drc_profile>film_light</rf_mode_drc_profile></drc><lfe_lowpass_filter>true</lfe_lowpass_filter><surround_90_degree_phase_shift>true</surround_90_degree_phase_shift><surround_3db_attenuation>true</surround_3db_attenuation><downmix><loro_center_mix_level>-3</loro_center_mix_level><loro_surround_mix_level>-3</loro_surround_mix_level><ltrt_center_mix_level>-3</ltrt_center_mix_level><ltrt_surround_mix_level>-3</ltrt_surround_mix_level><preferred_downmix_mode>loro</preferred_downmix_mode></downmix>{extra_xml}<allow_hybrid_downmix>false</allow_hybrid_downmix><embedded_timecodes><starting_timecode>off</starting_timecode><frame_rate>auto</frame_rate></embedded_timecodes><custom_dialnorm>0</custom_dialnorm></pcm_to_ddp></audio></filter>\n\
  <output><ec3 version=\"1\"><file_name>{output_name}</file_name><storage><local><path>Z:{}</path></local></storage></ec3></output>\n\
  <misc><temp_dir><clean_temp>true</clean_temp><path>Z:{}</path></temp_dir></misc>\n\
</job_config>\n",
        temp.path().join("in").display(),
        temp.path().join("out").display(),
        temp.path().join("tmp").display(),
    )
}

fn pcm_to_ddp_xml(temp: &TempDir, output_name: &str, encoder_mode: &str, data_rate: u16) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
<job_config>\n\
  <input><audio><wav version=\"1\"><file_name>8ch.wav</file_name><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa><storage><local><path>Z:{}</path></local></storage></wav></audio></input>\n\
  <filter><audio><pcm_to_ddp version=\"3\"><loudness><measure_only><metering_mode>1770-3</metering_mode><dialogue_intelligence>true</dialogue_intelligence><speech_threshold>15</speech_threshold></measure_only></loudness><encoder_mode>{encoder_mode}</encoder_mode><bitstream_mode>complete_main</bitstream_mode><downmix_config>off</downmix_config><data_rate>{data_rate}</data_rate><timecode_frame_rate>not_indicated</timecode_frame_rate><start>first_frame_of_action</start><end>end_of_file</end><time_base>file_position</time_base><prepend_silence_duration>0.0</prepend_silence_duration><append_silence_duration>0.0</append_silence_duration><lfe_on>true</lfe_on><dolby_surround_mode>not_indicated</dolby_surround_mode><dolby_surround_ex_mode>no</dolby_surround_ex_mode><user_data>-1</user_data><drc><line_mode_drc_profile>film_light</line_mode_drc_profile><rf_mode_drc_profile>film_light</rf_mode_drc_profile></drc><lfe_lowpass_filter>true</lfe_lowpass_filter><surround_90_degree_phase_shift>true</surround_90_degree_phase_shift><surround_3db_attenuation>true</surround_3db_attenuation><downmix><loro_center_mix_level>-3</loro_center_mix_level><loro_surround_mix_level>-3</loro_surround_mix_level><ltrt_center_mix_level>-3</ltrt_center_mix_level><ltrt_surround_mix_level>-3</ltrt_surround_mix_level><preferred_downmix_mode>loro</preferred_downmix_mode></downmix><allow_hybrid_downmix>false</allow_hybrid_downmix><embedded_timecodes><starting_timecode>off</starting_timecode><frame_rate>auto</frame_rate></embedded_timecodes><custom_dialnorm>0</custom_dialnorm></pcm_to_ddp></audio></filter>\n\
  <output><ec3 version=\"1\"><file_name>{output_name}</file_name><storage><local><path>Z:{}</path></local></storage></ec3></output>\n\
  <misc><temp_dir><clean_temp>true</clean_temp><path>Z:{}</path></temp_dir></misc>\n\
</job_config>\n",
        temp.path().join("in").display(),
        temp.path().join("out").display(),
        temp.path().join("tmp").display(),
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
        "expected no_backend variant to normalize backend, got:\n{}",
        no_backend_stdout
    );

    let backend_pe = run_dee(
        &temp.path().join("backend_pe.xml"),
        &temp.path().join("backend_pe.log"),
    );
    assert_success(&backend_pe, "atmos backend_pe");
    let backend_pe_stdout = String::from_utf8_lossy(&backend_pe.stdout);
    assert!(
        backend_pe_stdout.contains("Using atmosprocessor backend"),
        "expected backend_pe variant to normalize backend, got:\n{}",
        backend_pe_stdout
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
        (
            "ddp71",
            render_atmos_xml(
                &temp,
                "atmos_ec3_single.streaming.yaml",
                "ddp71_base.ec3",
                |job| {
                    job.encode_mode = EncodeMode::Ddp71;
                    job.filter = FilterOverrides::default();
                },
            ),
            "ddp71_base.ec3",
            "ddp71_unknown_trim.ec3",
            Some("Invalid encoder_mode value: ddp71"),
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

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_to_ddp_bluray_hidden_params_are_rejected() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_8ch_input(&temp);

    let baseline_xml = pcm_to_ddp_bluray_xml(&temp, "baseline.ec3", "");
    write_text(&temp.path().join("baseline.xml"), &baseline_xml);
    let baseline = run_dee(
        &temp.path().join("baseline.xml"),
        &temp.path().join("baseline.log"),
    );
    assert_success(&baseline, "pcm_to_ddp baseline");
    assert!(
        temp.path().join("out/baseline.ec3").exists(),
        "expected baseline ec3 output to exist"
    );

    let cases = [
        (
            "direct_surround_trim_5_1",
            "<surround_trim_5_1>auto</surround_trim_5_1>",
            "Unknown property: surround_trim_5_1",
        ),
        (
            "direct_height_trim_5_1",
            "<height_trim_5_1>auto</height_trim_5_1>",
            "Unknown property: height_trim_5_1",
        ),
        (
            "direct_surround_trim_7_1",
            "<surround_trim_7_1>auto</surround_trim_7_1>",
            "Unknown property: surround_trim_7_1",
        ),
        (
            "direct_encoding_backend",
            "<encoding_backend>atmosprocessor</encoding_backend>",
            "Unknown property: encoding_backend",
        ),
        (
            "custom_trims_block",
            "<custom_trims><surround_trim_5_1>auto</surround_trim_5_1><height_trim_5_1>auto</height_trim_5_1></custom_trims>",
            "Unknown property: custom_trims",
        ),
    ];

    for (name, extra_xml, needle) in cases {
        let xml = pcm_to_ddp_bluray_xml(&temp, &format!("{name}.ec3"), extra_xml);
        let xml_path = temp.path().join(format!("{name}.xml"));
        let log_path = temp.path().join(format!("{name}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(&output, needle, name);
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_to_ddp_bitrate_mode_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_8ch_input(&temp);

    let ddp71_cases = [
        (384_u16, None),
        (448_u16, None),
        (576_u16, None),
        (640_u16, None),
        (704_u16, None),
        (768_u16, None),
        (832_u16, None),
        (896_u16, None),
        (960_u16, None),
        (1008_u16, None),
        (1024_u16, None),
        (
            1280_u16,
            Some("Valid value(s): 384,448,576,640,704,768,832,896,960,1008,1024."),
        ),
        (
            1536_u16,
            Some("Valid value(s): 384,448,576,640,704,768,832,896,960,1008,1024."),
        ),
        (
            1664_u16,
            Some("Valid value(s): 384,448,576,640,704,768,832,896,960,1008,1024."),
        ),
    ];

    for (bitrate, expected_failure) in ddp71_cases {
        let output_name = format!("pcm_ddp71_{bitrate}.ec3");
        let xml = pcm_to_ddp_xml(&temp, &output_name, "ddp71", bitrate);
        let xml_path = temp.path().join(format!("pcm_ddp71_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_ddp71_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        if let Some(needle) = expected_failure {
            assert_failure_contains(
                &output,
                needle,
                &format!("pcm_to_ddp ddp71 bitrate={bitrate}"),
            );
        } else {
            assert_success(&output, &format!("pcm_to_ddp ddp71 bitrate={bitrate}"));
            assert_output_exists(
                &temp.path().join("out").join(&output_name),
                &format!("pcm_to_ddp ddp71 bitrate={bitrate}"),
            );
        }
    }

    let bluray_cases = [768_u16, 1024, 1280, 1536, 1664];
    for bitrate in bluray_cases {
        let output_name = format!("pcm_bluray_{bitrate}.ec3");
        let xml = pcm_to_ddp_xml(&temp, &output_name, "bluray", bitrate);
        let xml_path = temp.path().join(format!("pcm_bluray_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_bluray_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("pcm_to_ddp bluray bitrate={bitrate}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("pcm_to_ddp bluray bitrate={bitrate}"),
        );
    }
}
