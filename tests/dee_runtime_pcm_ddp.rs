use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
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

fn extract_log_metric(log_path: &Path, key: &str) -> Option<String> {
    let content = fs::read_to_string(log_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", log_path.display()));
    content.lines().find_map(|line| {
        let idx = line.find(key)?;
        let value = &line[idx + key.len()..];
        let value = value.trim().trim_end_matches('.');
        Some(value.to_string())
    })
}

fn output_sha256(path: &Path) -> String {
    let output = Command::new("sh")
        .arg("-lc")
        .arg(format!(
            "shasum -a 256 \"{}\" | awk '{{print $1}}'",
            path.display()
        ))
        .output()
        .unwrap_or_else(|err| panic!("failed to hash {}: {err}", path.display()));
    assert!(
        output.status.success(),
        "hashing {} should succeed, stdout:\n{}\nstderr:\n{}",
        path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn combined_output(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
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

fn generate_pcm_6ch_input(temp: &TempDir) -> PathBuf {
    let root = repo_root();
    let input = root.join("testfiles/16ch.wav");
    let output = temp.path().join("in/6ch.wav");

    let status = Command::new("ffmpeg")
        .args(["-y", "-i", input.to_str().expect("utf-8 input path")])
        .args(["-ac", "6"])
        .args([
            "-c:a",
            "pcm_f32le",
            output.to_str().expect("utf-8 output path"),
        ])
        .status()
        .expect("run ffmpeg");

    assert!(status.success(), "ffmpeg should generate 6ch wav");
    output
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

fn pcm_to_ddp_xml(
    temp: &TempDir,
    input_name: &str,
    output_name: &str,
    output_tag: &str,
    encoder_mode: &str,
    downmix_config: &str,
    data_rate: u16,
) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
<job_config>\n\
  <input><audio><wav version=\"1\"><file_name>{input_name}</file_name><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa><storage><local><path>Z:{}</path></local></storage></wav></audio></input>\n\
  <filter><audio><pcm_to_ddp version=\"3\"><loudness><measure_only><metering_mode>1770-3</metering_mode><dialogue_intelligence>true</dialogue_intelligence><speech_threshold>15</speech_threshold></measure_only></loudness><encoder_mode>{encoder_mode}</encoder_mode><bitstream_mode>complete_main</bitstream_mode><downmix_config>{downmix_config}</downmix_config><data_rate>{data_rate}</data_rate><timecode_frame_rate>not_indicated</timecode_frame_rate><start>first_frame_of_action</start><end>end_of_file</end><time_base>file_position</time_base><prepend_silence_duration>0.0</prepend_silence_duration><append_silence_duration>0.0</append_silence_duration><lfe_on>true</lfe_on><dolby_surround_mode>not_indicated</dolby_surround_mode><dolby_surround_ex_mode>no</dolby_surround_ex_mode><user_data>-1</user_data><drc><line_mode_drc_profile>film_light</line_mode_drc_profile><rf_mode_drc_profile>film_light</rf_mode_drc_profile></drc><lfe_lowpass_filter>true</lfe_lowpass_filter><surround_90_degree_phase_shift>true</surround_90_degree_phase_shift><surround_3db_attenuation>true</surround_3db_attenuation><downmix><loro_center_mix_level>-3</loro_center_mix_level><loro_surround_mix_level>-3</loro_surround_mix_level><ltrt_center_mix_level>-3</ltrt_center_mix_level><ltrt_surround_mix_level>-3</ltrt_surround_mix_level><preferred_downmix_mode>loro</preferred_downmix_mode></downmix><allow_hybrid_downmix>false</allow_hybrid_downmix><embedded_timecodes><starting_timecode>off</starting_timecode><frame_rate>auto</frame_rate></embedded_timecodes><custom_dialnorm>0</custom_dialnorm></pcm_to_ddp></audio></filter>\n\
  <output><{output_tag} version=\"1\"><file_name>{output_name}</file_name><storage><local><path>Z:{}</path></local></storage></{output_tag}></output>\n\
  <misc><temp_dir><clean_temp>true</clean_temp><path>Z:{}</path></temp_dir></misc>\n\
</job_config>\n",
        temp.path().join("in").display(),
        temp.path().join("out").display(),
        temp.path().join("tmp").display(),
    )
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_bluray_hidden_params_are_rejected() {
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
    assert_success(&baseline, "pcm_ddp baseline");
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
fn pcm_ddp_8ch_bitrate_mode_matrix_matches_runtime() {
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
        let xml = pcm_to_ddp_xml(
            &temp,
            "8ch.wav",
            &output_name,
            "ec3",
            "ddp71",
            "off",
            bitrate,
        );
        let xml_path = temp.path().join(format!("pcm_ddp71_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_ddp71_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        if let Some(needle) = expected_failure {
            assert_failure_contains(
                &output,
                needle,
                &format!("pcm_ddp 8ch ddp71 bitrate={bitrate}"),
            );
        } else {
            assert_success(&output, &format!("pcm_ddp 8ch ddp71 bitrate={bitrate}"));
            assert_output_exists(
                &temp.path().join("out").join(&output_name),
                &format!("pcm_ddp 8ch ddp71 bitrate={bitrate}"),
            );
        }
    }

    let bluray_cases = [768_u16, 1024, 1280, 1536, 1664];
    for bitrate in bluray_cases {
        let output_name = format!("pcm_bluray_{bitrate}.ec3");
        let xml = pcm_to_ddp_xml(
            &temp,
            "8ch.wav",
            &output_name,
            "ec3",
            "bluray",
            "off",
            bitrate,
        );
        let xml_path = temp.path().join(format!("pcm_bluray_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_bluray_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("pcm_ddp 8ch bluray bitrate={bitrate}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("pcm_ddp 8ch bluray bitrate={bitrate}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_6ch_bitrate_mode_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

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
        let output_name = format!("pcm_6ch_ddp71_{bitrate}.ec3");
        let xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &output_name,
            "ec3",
            "ddp71",
            "off",
            bitrate,
        );
        let xml_path = temp.path().join(format!("pcm_6ch_ddp71_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_6ch_ddp71_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        if let Some(needle) = expected_failure {
            assert_failure_contains(
                &output,
                needle,
                &format!("pcm_ddp 6ch ddp71 bitrate={bitrate}"),
            );
        } else {
            assert_success(&output, &format!("pcm_ddp 6ch ddp71 bitrate={bitrate}"));
            assert_output_exists(
                &temp.path().join("out").join(&output_name),
                &format!("pcm_ddp 6ch ddp71 bitrate={bitrate}"),
            );
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                stdout.contains("Encoding 5.1 channel input in 7.1 channel mode."),
                "pcm_ddp 6ch ddp71 bitrate={bitrate} should announce 5.1->7.1 mode, got:\n{stdout}",
            );
        }
    }

    let bluray_cases = [768_u16, 1024, 1280, 1536, 1664];
    for bitrate in bluray_cases {
        let output_name = format!("pcm_6ch_bluray_{bitrate}.ec3");
        let xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &output_name,
            "ec3",
            "bluray",
            "off",
            bitrate,
        );
        let xml_path = temp.path().join(format!("pcm_6ch_bluray_{bitrate}.xml"));
        let log_path = temp.path().join(format!("pcm_6ch_bluray_{bitrate}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("pcm_ddp 6ch bluray bitrate={bitrate}"));
        assert_output_exists(
            &temp.path().join("out").join(&output_name),
            &format!("pcm_ddp 6ch bluray bitrate={bitrate}"),
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("Encoding 5.1 channel input in 7.1 channel mode."),
            "pcm_ddp 6ch bluray bitrate={bitrate} should announce 5.1->7.1 mode, got:\n{stdout}",
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_dd_and_ddp_full_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);
    generate_pcm_8ch_input(&temp);

    let dd_valid = [224_u16, 256, 320, 384, 448, 512, 576, 640];
    let ddp_valid = [
        192_u16, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384,
        400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
    ];
    let input_cases = [("6ch.wav", "5.1"), ("8ch.wav", "5.1")];

    for (input_name, downmix_config) in input_cases {
        for bitrate in dd_valid {
            let output_name = format!("{input_name}_dd_{bitrate}.ac3");
            let xml = pcm_to_ddp_xml(
                &temp,
                input_name,
                &output_name,
                "ac3",
                "dd",
                downmix_config,
                bitrate,
            );
            let xml_path = temp.path().join(format!("{input_name}_dd_{bitrate}.xml"));
            let log_path = temp.path().join(format!("{input_name}_dd_{bitrate}.log"));
            write_text(&xml_path, &xml);

            let output = run_dee(&xml_path, &log_path);
            let context = format!("pcm_dd input={input_name} bitrate={bitrate}");
            assert_success(&output, &context);
            assert_output_exists(&temp.path().join("out").join(&output_name), &context);
        }

        for (bitrate, needle) in [
            (192_u16, "Valid value(s): 224,256,320,384,448,512,576,640."),
            (768_u16, "Valid value(s): 224,256,320,384,448,512,576,640."),
        ] {
            let output_name = format!("{input_name}_dd_{bitrate}.ac3");
            let xml = pcm_to_ddp_xml(
                &temp,
                input_name,
                &output_name,
                "ac3",
                "dd",
                downmix_config,
                bitrate,
            );
            let xml_path = temp.path().join(format!("{input_name}_dd_{bitrate}.xml"));
            let log_path = temp.path().join(format!("{input_name}_dd_{bitrate}.log"));
            write_text(&xml_path, &xml);

            let output = run_dee(&xml_path, &log_path);
            let context = format!("pcm_dd input={input_name} bitrate={bitrate}");
            assert_failure_contains(&output, needle, &context);
        }
    }

    for (input_name, downmix_config) in input_cases {
        for bitrate in ddp_valid {
            let output_name = format!("{input_name}_ddp_{bitrate}.ec3");
            let xml = pcm_to_ddp_xml(
                &temp,
                input_name,
                &output_name,
                "ec3",
                "ddp",
                downmix_config,
                bitrate,
            );
            let xml_path = temp.path().join(format!("{input_name}_ddp_{bitrate}.xml"));
            let log_path = temp.path().join(format!("{input_name}_ddp_{bitrate}.log"));
            write_text(&xml_path, &xml);

            let output = run_dee(&xml_path, &log_path);
            let context = format!("pcm_ddp input={input_name} bitrate={bitrate}");
            assert_success(&output, &context);
            assert_output_exists(&temp.path().join("out").join(&output_name), &context);
        }

        for (bitrate, needle) in [
            (191_u16, "Invalid data_rate value: 191."),
            (1664_u16, "Data rate: 1664 is not allowed."),
        ] {
            let output_name = format!("{input_name}_ddp_{bitrate}.ec3");
            let xml = pcm_to_ddp_xml(
                &temp,
                input_name,
                &output_name,
                "ec3",
                "ddp",
                downmix_config,
                bitrate,
            );
            let xml_path = temp.path().join(format!("{input_name}_ddp_{bitrate}.xml"));
            let log_path = temp.path().join(format!("{input_name}_ddp_{bitrate}.log"));
            write_text(&xml_path, &xml);

            let output = run_dee(&xml_path, &log_path);
            let context = format!("pcm_ddp input={input_name} bitrate={bitrate}");
            assert_failure_contains(&output, needle, &context);
        }
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_advanced_parameter_smoke_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let ddp_ec3_base = pcm_to_ddp_xml(&temp, "6ch.wav", "advanced.ec3", "ec3", "ddp", "5.1", 768);
    let dd_ac3_base = pcm_to_ddp_xml(&temp, "6ch.wav", "advanced.ac3", "ac3", "dd", "5.1", 384);
    let valid_cases = [
        (
            "bitstream_mode",
            ddp_ec3_base.replace(
                "<bitstream_mode>complete_main</bitstream_mode>",
                "<bitstream_mode>commentary</bitstream_mode>",
            ),
        ),
        (
            "lfe_on",
            ddp_ec3_base.replace("<lfe_on>true</lfe_on>", "<lfe_on>false</lfe_on>"),
        ),
        (
            "dolby_surround_mode",
            ddp_ec3_base.replace(
                "<dolby_surround_mode>not_indicated</dolby_surround_mode>",
                "<dolby_surround_mode>yes</dolby_surround_mode>",
            ),
        ),
        (
            "dolby_surround_ex_mode",
            ddp_ec3_base.replace(
                "<dolby_surround_ex_mode>no</dolby_surround_ex_mode>",
                "<dolby_surround_ex_mode>not_indicated</dolby_surround_ex_mode>",
            ),
        ),
        (
            "user_data",
            ddp_ec3_base.replace("<user_data>-1</user_data>", "<user_data>7</user_data>"),
        ),
        (
            "lfe_lowpass_filter",
            ddp_ec3_base.replace(
                "<lfe_lowpass_filter>true</lfe_lowpass_filter>",
                "<lfe_lowpass_filter>false</lfe_lowpass_filter>",
            ),
        ),
        (
            "surround_90_degree_phase_shift",
            ddp_ec3_base.replace(
                "<surround_90_degree_phase_shift>true</surround_90_degree_phase_shift>",
                "<surround_90_degree_phase_shift>false</surround_90_degree_phase_shift>",
            ),
        ),
        (
            "surround_3db_attenuation",
            ddp_ec3_base.replace(
                "<surround_3db_attenuation>true</surround_3db_attenuation>",
                "<surround_3db_attenuation>false</surround_3db_attenuation>",
            ),
        ),
        (
            "allow_hybrid_downmix",
            ddp_ec3_base.replace(
                "<allow_hybrid_downmix>false</allow_hybrid_downmix>",
                "<allow_hybrid_downmix>true</allow_hybrid_downmix>",
            ),
        ),
        (
            "starting_timecode",
            dd_ac3_base
                .replace(
                    "<timecode_frame_rate>not_indicated</timecode_frame_rate>",
                    "<timecode_frame_rate>23.976</timecode_frame_rate>",
                )
                .replace(
                    "<start>first_frame_of_action</start>",
                    "<start>00:00:00:00</start>",
                )
                .replace(
                    "<time_base>file_position</time_base>",
                    "<time_base>embedded_timecode</time_base>",
                )
                .replace(
                    "<starting_timecode>off</starting_timecode>",
                    "<starting_timecode>auto</starting_timecode>",
                )
                .replace(
                    "<frame_rate>auto</frame_rate>",
                    "<frame_rate>23.976</frame_rate>",
                ),
        ),
        (
            "frame_rate",
            ddp_ec3_base.replace(
                "<frame_rate>auto</frame_rate>",
                "<frame_rate>29.97</frame_rate>",
            ),
        ),
    ];

    for (name, xml) in valid_cases {
        let xml_path = temp.path().join(format!("valid_{name}.xml"));
        let log_path = temp.path().join(format!("valid_{name}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(&output, &format!("valid advanced param {name}"));
    }

    let invalid_cases = [
        (
            "bitstream_mode",
            ddp_ec3_base.replace(
                "<bitstream_mode>complete_main</bitstream_mode>",
                "<bitstream_mode>bogus</bitstream_mode>",
            ),
        ),
        (
            "lfe_on",
            ddp_ec3_base.replace("<lfe_on>true</lfe_on>", "<lfe_on>bogus</lfe_on>"),
        ),
        (
            "dolby_surround_mode",
            ddp_ec3_base.replace(
                "<dolby_surround_mode>not_indicated</dolby_surround_mode>",
                "<dolby_surround_mode>bogus</dolby_surround_mode>",
            ),
        ),
        (
            "dolby_surround_ex_mode",
            ddp_ec3_base.replace(
                "<dolby_surround_ex_mode>no</dolby_surround_ex_mode>",
                "<dolby_surround_ex_mode>bogus</dolby_surround_ex_mode>",
            ),
        ),
        (
            "user_data",
            ddp_ec3_base.replace("<user_data>-1</user_data>", "<user_data>bogus</user_data>"),
        ),
        (
            "lfe_lowpass_filter",
            ddp_ec3_base.replace(
                "<lfe_lowpass_filter>true</lfe_lowpass_filter>",
                "<lfe_lowpass_filter>bogus</lfe_lowpass_filter>",
            ),
        ),
        (
            "surround_90_degree_phase_shift",
            ddp_ec3_base.replace(
                "<surround_90_degree_phase_shift>true</surround_90_degree_phase_shift>",
                "<surround_90_degree_phase_shift>bogus</surround_90_degree_phase_shift>",
            ),
        ),
        (
            "surround_3db_attenuation",
            ddp_ec3_base.replace(
                "<surround_3db_attenuation>true</surround_3db_attenuation>",
                "<surround_3db_attenuation>bogus</surround_3db_attenuation>",
            ),
        ),
        (
            "allow_hybrid_downmix",
            ddp_ec3_base.replace(
                "<allow_hybrid_downmix>false</allow_hybrid_downmix>",
                "<allow_hybrid_downmix>bogus</allow_hybrid_downmix>",
            ),
        ),
        (
            "starting_timecode",
            ddp_ec3_base.replace(
                "<starting_timecode>off</starting_timecode>",
                "<starting_timecode>auto</starting_timecode>",
            ),
        ),
    ];

    for (name, xml) in invalid_cases {
        let xml_path = temp.path().join(format!("invalid_{name}.xml"));
        let log_path = temp.path().join(format!("invalid_{name}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        if name == "starting_timecode" {
            assert_failure_contains(
                &output,
                "Embedded timecodes are only supported for encoding DD and Blu-ray streams.",
                "invalid advanced param starting_timecode",
            );
        } else {
            assert!(
                !output.status.success(),
                "invalid advanced param {name} should fail"
            );
        }
    }

    let permissive_frame_rate_xml = ddp_ec3_base.replace(
        "<frame_rate>auto</frame_rate>",
        "<frame_rate>bogus</frame_rate>",
    );
    let permissive_frame_rate_xml_path = temp.path().join("runtime_permissive_frame_rate.xml");
    let permissive_frame_rate_log_path = temp.path().join("runtime_permissive_frame_rate.log");
    write_text(&permissive_frame_rate_xml_path, &permissive_frame_rate_xml);
    let permissive_frame_rate_output = run_dee(
        &permissive_frame_rate_xml_path,
        &permissive_frame_rate_log_path,
    );
    assert_success(
        &permissive_frame_rate_output,
        "runtime currently accepts frame_rate=bogus on ddp",
    );
}

#[test]
#[ignore = "manual experiment: compare pcm_ddp_v1 metering_mode outputs"]
fn pcm_ddp_metering_mode_experiment() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let cases = [
        ("dd", "ac3", "5.1", 640_u16),
        ("ddp", "ec3", "5.1", 1024),
        ("ddp71", "ec3", "off", 1024),
        ("bluray", "ec3", "off", 1664),
    ];

    for (encoder_mode, output_tag, downmix_config, data_rate) in cases {
        let xml_1770_3 = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("{encoder_mode}_1770-3.{output_tag}"),
            output_tag,
            encoder_mode,
            downmix_config,
            data_rate,
        );
        let xml_1770_4 = xml_1770_3.replace(
            "<metering_mode>1770-3</metering_mode>",
            "<metering_mode>1770-4</metering_mode>",
        );

        let xml_1770_3_path = temp.path().join(format!("{encoder_mode}_1770-3.xml"));
        let xml_1770_4_path = temp.path().join(format!("{encoder_mode}_1770-4.xml"));
        let log_1770_3_path = temp.path().join(format!("{encoder_mode}_1770-3.log"));
        let log_1770_4_path = temp.path().join(format!("{encoder_mode}_1770-4.log"));
        let out_1770_3_path = temp
            .path()
            .join("out")
            .join(format!("{encoder_mode}_1770-3.{output_tag}"));
        write_text(&xml_1770_3_path, &xml_1770_3);
        let output_1770_3 = run_dee(&xml_1770_3_path, &log_1770_3_path);
        assert_success(&output_1770_3, &format!("{encoder_mode} metering=1770-3"));
        assert_output_exists(&out_1770_3_path, &format!("{encoder_mode} metering=1770-3"));
        let hash_1770_3 = output_sha256(&out_1770_3_path);
        let measured_1770_3 =
            extract_log_metric(&log_1770_3_path, "measured_loudness=").unwrap_or_default();
        let dialogue_1770_3 =
            extract_log_metric(&log_1770_3_path, "dialogue_loudness=").unwrap_or_default();

        write_text(&xml_1770_4_path, &xml_1770_4);
        let output_1770_4 = run_dee(&xml_1770_4_path, &log_1770_4_path);
        let combined_1770_4 = combined_output(&output_1770_4);
        assert!(
            !output_1770_4.status.success(),
            "{encoder_mode} metering=1770-4 is expected to fail on DEE 5.2.1"
        );
        assert!(
            combined_1770_4.contains("Invalid metering_mode value: 1770-4."),
            "{encoder_mode} metering=1770-4 should fail with invalid metering_mode, got:\n{combined_1770_4}"
        );

        println!(
            "mode={encoder_mode} bitrate={data_rate} metering_1770_3=ok hash_1770_3={hash_1770_3} measured_1770_3={measured_1770_3} dialogue_1770_3={dialogue_1770_3} metering_1770_4=invalid",
        );
    }
}
