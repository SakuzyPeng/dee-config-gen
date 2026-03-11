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

fn replace_xml_value(xml: &str, from: &str, to: &str) -> String {
    assert!(
        xml.contains(from),
        "expected XML snippet '{from}' to exist before replacement"
    );
    xml.replace(from, to)
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

fn output_tag_for_mode(encoder_mode: &str) -> &'static str {
    match encoder_mode {
        "dd" => "ac3",
        "ddp" | "ddp71" | "bluray" => "ec3",
        _ => panic!("unsupported encoder_mode: {encoder_mode}"),
    }
}

fn default_downmix_for_mode(encoder_mode: &str) -> &'static str {
    match encoder_mode {
        "dd" | "ddp" => "5.1",
        "ddp71" | "bluray" => "off",
        _ => panic!("unsupported encoder_mode: {encoder_mode}"),
    }
}

fn default_data_rate_for_mode(encoder_mode: &str) -> u16 {
    match encoder_mode {
        "dd" => 640,
        "ddp" => 1024,
        "ddp71" => 1024,
        "bluray" => 1664,
        _ => panic!("unsupported encoder_mode: {encoder_mode}"),
    }
}

fn with_embedded_timecodes(
    xml: &str,
    timecode_frame_rate: &str,
    start: &str,
    frame_rate: &str,
    starting_timecode: &str,
) -> String {
    xml.replace(
        "<timecode_frame_rate>not_indicated</timecode_frame_rate>",
        &format!("<timecode_frame_rate>{timecode_frame_rate}</timecode_frame_rate>"),
    )
    .replace(
        "<start>first_frame_of_action</start>",
        &format!("<start>{start}</start>"),
    )
    .replace(
        "<time_base>file_position</time_base>",
        "<time_base>embedded_timecode</time_base>",
    )
    .replace(
        "<starting_timecode>off</starting_timecode>",
        &format!("<starting_timecode>{starting_timecode}</starting_timecode>"),
    )
    .replace(
        "<frame_rate>auto</frame_rate>",
        &format!("<frame_rate>{frame_rate}</frame_rate>"),
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

    let ac3_valid_bitrates = [224_u16, 256, 320, 384, 448, 512, 576, 640];
    let eac3_valid_bitrates = [
        192_u16, 200, 208, 216, 224, 232, 240, 248, 256, 272, 288, 304, 320, 336, 352, 368, 384,
        400, 448, 512, 576, 640, 704, 768, 832, 896, 960, 1008, 1024,
    ];
    let input_cases = [("6ch.wav", "5.1"), ("8ch.wav", "5.1")];

    for (input_name, downmix_config) in input_cases {
        for bitrate in ac3_valid_bitrates {
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
        for bitrate in eac3_valid_bitrates {
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
fn pcm_ddp_downmix_config_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);
    generate_pcm_8ch_input(&temp);

    let success_cases = [
        ("6ch.wav", "dd", "5.1"),
        ("6ch.wav", "dd", "off"),
        ("8ch.wav", "dd", "5.1"),
        ("6ch.wav", "ddp", "5.1"),
        ("6ch.wav", "ddp", "off"),
        ("8ch.wav", "ddp", "5.1"),
        ("6ch.wav", "ddp71", "off"),
        ("8ch.wav", "ddp71", "off"),
        ("6ch.wav", "bluray", "off"),
        ("8ch.wav", "bluray", "off"),
    ];

    for (input_name, encoder_mode, downmix_config) in success_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let output_name = format!("{input_name}_{encoder_mode}_{downmix_config}.{output_tag}");
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &output_name,
            output_tag,
            encoder_mode,
            downmix_config,
            data_rate,
        );
        let xml_path = temp
            .path()
            .join(format!("{input_name}_{encoder_mode}_{downmix_config}.xml"));
        let log_path = temp
            .path()
            .join(format!("{input_name}_{encoder_mode}_{downmix_config}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        let context =
            format!("downmix_config input={input_name} mode={encoder_mode} value={downmix_config}");
        assert_success(&output, &context);
        assert_output_exists(&temp.path().join("out").join(&output_name), &context);
    }

    let failure_cases = [
        ("8ch.wav", "dd", "off"),
        ("8ch.wav", "ddp", "off"),
        ("6ch.wav", "ddp71", "5.1"),
        ("8ch.wav", "ddp71", "5.1"),
        ("6ch.wav", "bluray", "5.1"),
        ("8ch.wav", "bluray", "5.1"),
    ];

    for (input_name, encoder_mode, downmix_config) in failure_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let output_name = format!("{input_name}_{encoder_mode}_{downmix_config}.{output_tag}");
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &output_name,
            output_tag,
            encoder_mode,
            downmix_config,
            data_rate,
        );
        let xml_path = temp.path().join(format!(
            "invalid_{input_name}_{encoder_mode}_{downmix_config}.xml"
        ));
        let log_path = temp.path().join(format!(
            "invalid_{input_name}_{encoder_mode}_{downmix_config}.log"
        ));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        let context = format!(
            "invalid downmix_config input={input_name} mode={encoder_mode} value={downmix_config}"
        );
        let expected = match (encoder_mode, input_name, downmix_config) {
            ("dd", "8ch.wav", "off") | ("ddp", "8ch.wav", "off") => {
                "Resulting output channels: 8 is not allowed. Valid value(s): 1,2,6."
            }
            ("ddp71", _, "5.1") => "Downmix_config must be set to 'off' in encoder_mode=ddp71.",
            ("bluray", _, "5.1") => "Downmix_config must be set to 'off' in DD+ Blu-ray 7.1.",
            _ => panic!("missing expected downmix_config failure for {context}"),
        };
        assert_failure_contains(&output, expected, &context);
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_starting_timecode_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let success_cases = [("dd", "6ch.wav"), ("bluray", "6ch.wav")];
    for (encoder_mode, input_name) in success_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &format!("starting_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );
        let xml = with_embedded_timecodes(&xml, "23.976", "00:00:00:00", "23.976", "auto");
        let xml_path = temp.path().join(format!("starting_{encoder_mode}.xml"));
        let log_path = temp.path().join(format!("starting_{encoder_mode}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        let context = format!("starting_timecode mode={encoder_mode}");
        assert_success(&output, &context);
        assert_output_exists(
            &temp
                .path()
                .join("out")
                .join(format!("starting_{encoder_mode}.{output_tag}")),
            &context,
        );
    }

    let failure_cases = [("ddp", "6ch.wav"), ("ddp71", "6ch.wav")];
    for (encoder_mode, input_name) in failure_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &format!("starting_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );
        let xml = with_embedded_timecodes(&xml, "23.976", "00:00:00:00", "23.976", "auto");
        let xml_path = temp
            .path()
            .join(format!("invalid_starting_{encoder_mode}.xml"));
        let log_path = temp
            .path()
            .join(format!("invalid_starting_{encoder_mode}.log"));
        write_text(&xml_path, &xml);

        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(
            &output,
            "Embedded timecodes are only supported for encoding DD and Blu-ray streams.",
            &format!("starting_timecode mode={encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_dialogue_intelligence_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("dialogue_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );

        for dialogue_intelligence in ["true", "false"] {
            let xml = replace_xml_value(
                &base_xml,
                "<dialogue_intelligence>true</dialogue_intelligence>",
                &format!("<dialogue_intelligence>{dialogue_intelligence}</dialogue_intelligence>"),
            );
            let xml_path = temp.path().join(format!(
                "dialogue_{encoder_mode}_{dialogue_intelligence}.xml"
            ));
            let log_path = temp.path().join(format!(
                "dialogue_{encoder_mode}_{dialogue_intelligence}.log"
            ));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!(
                    "pcm_ddp dialogue_intelligence={dialogue_intelligence} mode={encoder_mode}"
                ),
            );
        }

        let invalid_dialogue_xml = replace_xml_value(
            &base_xml,
            "<dialogue_intelligence>true</dialogue_intelligence>",
            "<dialogue_intelligence>bogus</dialogue_intelligence>",
        );
        let invalid_dialogue_xml_path = temp
            .path()
            .join(format!("dialogue_{encoder_mode}_invalid.xml"));
        let invalid_dialogue_log_path = temp
            .path()
            .join(format!("dialogue_{encoder_mode}_invalid.log"));
        write_text(&invalid_dialogue_xml_path, &invalid_dialogue_xml);
        let invalid_dialogue = run_dee(&invalid_dialogue_xml_path, &invalid_dialogue_log_path);
        assert_failure_contains(
            &invalid_dialogue,
            "dialogue_intelligence",
            &format!("invalid pcm_ddp dialogue_intelligence mode={encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_speech_threshold_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("speech_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );

        for speech_threshold in ["0", "100"] {
            let xml = replace_xml_value(
                &base_xml,
                "<speech_threshold>15</speech_threshold>",
                &format!("<speech_threshold>{speech_threshold}</speech_threshold>"),
            );
            let xml_path = temp
                .path()
                .join(format!("speech_{encoder_mode}_{speech_threshold}.xml"));
            let log_path = temp
                .path()
                .join(format!("speech_{encoder_mode}_{speech_threshold}.log"));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!("pcm_ddp speech_threshold={speech_threshold} mode={encoder_mode}"),
            );
        }

        let invalid_speech_xml = replace_xml_value(
            &base_xml,
            "<speech_threshold>15</speech_threshold>",
            "<speech_threshold>bogus</speech_threshold>",
        );
        let invalid_speech_xml_path = temp
            .path()
            .join(format!("speech_{encoder_mode}_invalid.xml"));
        let invalid_speech_log_path = temp
            .path()
            .join(format!("speech_{encoder_mode}_invalid.log"));
        write_text(&invalid_speech_xml_path, &invalid_speech_xml);
        let invalid_speech = run_dee(&invalid_speech_xml_path, &invalid_speech_log_path);
        assert_failure_contains(
            &invalid_speech,
            "speech_threshold",
            &format!("invalid pcm_ddp speech_threshold mode={encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_timecode_frame_rate_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("timecode_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );

        for timecode_frame_rate in ["not_indicated", "23.976"] {
            let xml = replace_xml_value(
                &base_xml,
                "<timecode_frame_rate>not_indicated</timecode_frame_rate>",
                &format!("<timecode_frame_rate>{timecode_frame_rate}</timecode_frame_rate>"),
            );
            let xml_path = temp.path().join(format!(
                "timecode_{encoder_mode}_{}.xml",
                timecode_frame_rate.replace('.', "_")
            ));
            let log_path = temp.path().join(format!(
                "timecode_{encoder_mode}_{}.log",
                timecode_frame_rate.replace('.', "_")
            ));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!("pcm_ddp timecode_frame_rate={timecode_frame_rate} mode={encoder_mode}"),
            );
        }
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_time_base_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let success_cases = [("dd", "6ch.wav"), ("bluray", "6ch.wav")];
    for (encoder_mode, input_name) in success_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &format!("embedded_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );
        let xml = with_embedded_timecodes(&xml, "23.976", "00:00:00:00", "23.976", "auto");
        let xml_path = temp.path().join(format!("embedded_{encoder_mode}.xml"));
        let log_path = temp.path().join(format!("embedded_{encoder_mode}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("pcm_ddp embedded time_base mode={encoder_mode}"),
        );
    }

    let failure_cases = [("ddp", "6ch.wav"), ("ddp71", "6ch.wav")];
    for (encoder_mode, input_name) in failure_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            input_name,
            &format!("embedded_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );
        let xml = with_embedded_timecodes(&xml, "23.976", "00:00:00:00", "23.976", "auto");
        let xml_path = temp
            .path()
            .join(format!("invalid_embedded_{encoder_mode}.xml"));
        let log_path = temp
            .path()
            .join(format!("invalid_embedded_{encoder_mode}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(
            &output,
            "Embedded timecodes are only supported for encoding DD and Blu-ray streams.",
            &format!("pcm_ddp embedded time_base mode={encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_bitstream_mode_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let success_cases = [
        ("dd", "complete_main"),
        ("dd", "commentary"),
        ("ddp", "complete_main"),
        ("ddp", "commentary"),
        ("ddp71", "complete_main"),
        ("ddp71", "commentary"),
        ("bluray", "complete_main"),
        ("bluray", "commentary"),
    ];

    for (encoder_mode, bitstream_mode) in success_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("bitstream_{encoder_mode}_{bitstream_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        )
        .replace(
            "<bitstream_mode>complete_main</bitstream_mode>",
            &format!("<bitstream_mode>{bitstream_mode}</bitstream_mode>"),
        );
        let xml_path = temp
            .path()
            .join(format!("bitstream_{encoder_mode}_{bitstream_mode}.xml"));
        let log_path = temp
            .path()
            .join(format!("bitstream_{encoder_mode}_{bitstream_mode}.log"));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_success(
            &output,
            &format!("bitstream_mode mode={encoder_mode} value={bitstream_mode}"),
        );
    }

    let invalid_cases = [("dd", "bogus"), ("ddp", "bogus")];
    for (encoder_mode, bitstream_mode) in invalid_cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("bitstream_{encoder_mode}_{bitstream_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        )
        .replace(
            "<bitstream_mode>complete_main</bitstream_mode>",
            &format!("<bitstream_mode>{bitstream_mode}</bitstream_mode>"),
        );
        let xml_path = temp.path().join(format!(
            "invalid_bitstream_{encoder_mode}_{bitstream_mode}.xml"
        ));
        let log_path = temp.path().join(format!(
            "invalid_bitstream_{encoder_mode}_{bitstream_mode}.log"
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        assert_failure_contains(
            &output,
            "Invalid bitstream_mode value",
            &format!("bitstream_mode mode={encoder_mode} value={bitstream_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_preferred_downmix_mode_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let cases = [
        ("dd", "loro", None),
        ("dd", "ltrt", None),
        (
            "dd",
            "ltrt-pl2",
            Some("Downmix Mode ltrt-pl2 not supported in DD mode."),
        ),
        ("dd", "not_indicated", None),
        ("ddp", "loro", None),
        ("ddp", "ltrt", None),
        ("ddp", "ltrt-pl2", None),
        ("ddp", "not_indicated", None),
        ("ddp71", "loro", None),
        ("ddp71", "ltrt", None),
        ("ddp71", "ltrt-pl2", None),
        ("ddp71", "not_indicated", None),
        ("bluray", "loro", None),
        ("bluray", "ltrt", None),
        (
            "bluray",
            "ltrt-pl2",
            Some("Downmix Mode ltrt-pl2 not supported in Blu-ray mode."),
        ),
        ("bluray", "not_indicated", None),
    ];

    for (encoder_mode, preferred_downmix_mode, expected_failure) in cases {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let xml = replace_xml_value(
            &pcm_to_ddp_xml(
                &temp,
                "6ch.wav",
                &format!("preferred_{encoder_mode}_{preferred_downmix_mode}.{output_tag}"),
                output_tag,
                encoder_mode,
                default_downmix_for_mode(encoder_mode),
                data_rate,
            ),
            "<preferred_downmix_mode>loro</preferred_downmix_mode>",
            &format!("<preferred_downmix_mode>{preferred_downmix_mode}</preferred_downmix_mode>"),
        );
        let xml_path = temp.path().join(format!(
            "preferred_{encoder_mode}_{preferred_downmix_mode}.xml"
        ));
        let log_path = temp.path().join(format!(
            "preferred_{encoder_mode}_{preferred_downmix_mode}.log"
        ));
        write_text(&xml_path, &xml);
        let output = run_dee(&xml_path, &log_path);
        let context =
            format!("preferred_downmix_mode mode={encoder_mode} value={preferred_downmix_mode}");
        match expected_failure {
            Some(needle) => assert_failure_contains(&output, needle, &context),
            None => assert_success(&output, &context),
        }
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_ltrt_pl2_is_rejected_for_dd_and_bluray() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let cases = [
        (
            "dd",
            "ac3",
            "5.1",
            640_u16,
            "Downmix Mode ltrt-pl2 not supported in DD mode.",
        ),
        (
            "bluray",
            "ec3",
            "off",
            1664_u16,
            "Downmix Mode ltrt-pl2 not supported in Blu-ray mode.",
        ),
    ];

    for (encoder_mode, output_tag, downmix_config, data_rate, needle) in cases {
        let baseline_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("{encoder_mode}_loro.{output_tag}"),
            output_tag,
            encoder_mode,
            downmix_config,
            data_rate,
        );
        let baseline_xml_path = temp.path().join(format!("{encoder_mode}_loro.xml"));
        let baseline_log_path = temp.path().join(format!("{encoder_mode}_loro.log"));
        write_text(&baseline_xml_path, &baseline_xml);
        let baseline_output = run_dee(&baseline_xml_path, &baseline_log_path);
        assert_success(
            &baseline_output,
            &format!("baseline preferred_downmix_mode for {encoder_mode}"),
        );

        let invalid_xml = baseline_xml.replace(
            "<preferred_downmix_mode>loro</preferred_downmix_mode>",
            "<preferred_downmix_mode>ltrt-pl2</preferred_downmix_mode>",
        );
        let invalid_xml_path = temp.path().join(format!("{encoder_mode}_ltrt-pl2.xml"));
        let invalid_log_path = temp.path().join(format!("{encoder_mode}_ltrt-pl2.log"));
        write_text(&invalid_xml_path, &invalid_xml);
        let invalid_output = run_dee(&invalid_xml_path, &invalid_log_path);
        assert_failure_contains(
            &invalid_output,
            needle,
            &format!("preferred_downmix_mode=ltrt-pl2 for {encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_boolean_processing_knobs_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    let params = [
        ("lfe_on", "<lfe_on>true</lfe_on>", "<lfe_on>false</lfe_on>"),
        (
            "lfe_lowpass_filter",
            "<lfe_lowpass_filter>true</lfe_lowpass_filter>",
            "<lfe_lowpass_filter>false</lfe_lowpass_filter>",
        ),
        (
            "surround_90_degree_phase_shift",
            "<surround_90_degree_phase_shift>true</surround_90_degree_phase_shift>",
            "<surround_90_degree_phase_shift>false</surround_90_degree_phase_shift>",
        ),
        (
            "surround_3db_attenuation",
            "<surround_3db_attenuation>true</surround_3db_attenuation>",
            "<surround_3db_attenuation>false</surround_3db_attenuation>",
        ),
    ];

    for encoder_mode in ["dd", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("boolean_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );

        for (param_name, default_snippet, valid_snippet) in params {
            let valid_xml = replace_xml_value(&base_xml, default_snippet, valid_snippet);
            let valid_xml_path = temp
                .path()
                .join(format!("valid_{encoder_mode}_{param_name}.xml"));
            let valid_log_path = temp
                .path()
                .join(format!("valid_{encoder_mode}_{param_name}.log"));
            write_text(&valid_xml_path, &valid_xml);
            let valid_output = run_dee(&valid_xml_path, &valid_log_path);
            assert_success(
                &valid_output,
                &format!("{param_name} valid override should succeed for {encoder_mode}"),
            );

            let invalid_xml = replace_xml_value(
                &base_xml,
                default_snippet,
                &default_snippet.replace("true", "bogus"),
            );
            let invalid_xml_path = temp
                .path()
                .join(format!("invalid_{encoder_mode}_{param_name}.xml"));
            let invalid_log_path = temp
                .path()
                .join(format!("invalid_{encoder_mode}_{param_name}.log"));
            write_text(&invalid_xml_path, &invalid_xml);
            let invalid_output = run_dee(&invalid_xml_path, &invalid_log_path);
            assert_failure_contains(
                &invalid_output,
                param_name,
                &format!("{param_name} invalid override should fail for {encoder_mode}"),
            );
        }
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_metadata_knobs_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);
    generate_pcm_8ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let input_names: &[&str] = &["6ch.wav", "8ch.wav"];

        for input_name in input_names {
            let base_xml = pcm_to_ddp_xml(
                &temp,
                input_name,
                &format!("metadata_{encoder_mode}_{input_name}.{output_tag}"),
                output_tag,
                encoder_mode,
                default_downmix_for_mode(encoder_mode),
                data_rate,
            );

            let dolby_surround_xml = replace_xml_value(
                &base_xml,
                "<dolby_surround_mode>not_indicated</dolby_surround_mode>",
                "<dolby_surround_mode>yes</dolby_surround_mode>",
            );
            let dolby_surround_path = temp
                .path()
                .join(format!("dolby_surround_{encoder_mode}_{input_name}.xml"));
            let dolby_surround_log = temp
                .path()
                .join(format!("dolby_surround_{encoder_mode}_{input_name}.log"));
            write_text(&dolby_surround_path, &dolby_surround_xml);
            let dolby_surround_output = run_dee(&dolby_surround_path, &dolby_surround_log);
            assert_success(
                &dolby_surround_output,
                &format!("dolby_surround_mode=yes should succeed for {encoder_mode} {input_name}"),
            );

            let surround_ex_value = if encoder_mode == "bluray" {
                "not_indicated"
            } else {
                "yes"
            };
            let surround_ex_xml = replace_xml_value(
                &base_xml,
                "<dolby_surround_ex_mode>no</dolby_surround_ex_mode>",
                &format!("<dolby_surround_ex_mode>{surround_ex_value}</dolby_surround_ex_mode>"),
            );
            let surround_ex_path = temp
                .path()
                .join(format!("surround_ex_{encoder_mode}_{input_name}.xml"));
            let surround_ex_log = temp
                .path()
                .join(format!("surround_ex_{encoder_mode}_{input_name}.log"));
            write_text(&surround_ex_path, &surround_ex_xml);
            let surround_ex_output = run_dee(&surround_ex_path, &surround_ex_log);
            assert_success(
                &surround_ex_output,
                &format!(
                    "dolby_surround_ex_mode={surround_ex_value} should succeed for {encoder_mode} {input_name}"
                ),
            );
            if encoder_mode == "bluray" {
                assert!(
                    combined_output(&surround_ex_output)
                        .contains("Auto-enabling dolby_surround_ex_mode=yes in Blu-ray mode."),
                    "bluray should document Dolby Surround EX normalization"
                );
            }

            let user_data_xml = replace_xml_value(
                &base_xml,
                "<user_data>-1</user_data>",
                "<user_data>7</user_data>",
            );
            let user_data_path = temp
                .path()
                .join(format!("user_data_{encoder_mode}_{input_name}.xml"));
            let user_data_log = temp
                .path()
                .join(format!("user_data_{encoder_mode}_{input_name}.log"));
            write_text(&user_data_path, &user_data_xml);
            let user_data_output = run_dee(&user_data_path, &user_data_log);
            assert_success(
                &user_data_output,
                &format!("user_data=7 should succeed for {encoder_mode} {input_name}"),
            );
        }

        let invalid_base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("invalid_metadata_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );
        for (param_name, default_snippet, invalid_snippet) in [
            (
                "dolby_surround_mode",
                "<dolby_surround_mode>not_indicated</dolby_surround_mode>",
                "<dolby_surround_mode>bogus</dolby_surround_mode>",
            ),
            (
                "dolby_surround_ex_mode",
                "<dolby_surround_ex_mode>no</dolby_surround_ex_mode>",
                "<dolby_surround_ex_mode>bogus</dolby_surround_ex_mode>",
            ),
            (
                "user_data",
                "<user_data>-1</user_data>",
                "<user_data>bogus</user_data>",
            ),
        ] {
            let invalid_xml =
                replace_xml_value(&invalid_base_xml, default_snippet, invalid_snippet);
            let invalid_xml_path = temp
                .path()
                .join(format!("invalid_{encoder_mode}_{param_name}.xml"));
            let invalid_log_path = temp
                .path()
                .join(format!("invalid_{encoder_mode}_{param_name}.log"));
            write_text(&invalid_xml_path, &invalid_xml);
            let invalid_output = run_dee(&invalid_xml_path, &invalid_log_path);
            if param_name == "user_data" {
                assert!(
                    !invalid_output.status.success(),
                    "{param_name} invalid override should fail for {encoder_mode}"
                );
            } else {
                assert_failure_contains(
                    &invalid_output,
                    param_name,
                    &format!("{param_name} invalid override should fail for {encoder_mode}"),
                );
            }
        }
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_allow_hybrid_downmix_runtime_matrix_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);
    generate_pcm_8ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);

        for input_name in ["6ch.wav", "8ch.wav"] {
            let valid_xml = replace_xml_value(
                &pcm_to_ddp_xml(
                    &temp,
                    input_name,
                    &format!("hybrid_{encoder_mode}_{input_name}.{output_tag}"),
                    output_tag,
                    encoder_mode,
                    default_downmix_for_mode(encoder_mode),
                    data_rate,
                ),
                "<allow_hybrid_downmix>false</allow_hybrid_downmix>",
                "<allow_hybrid_downmix>true</allow_hybrid_downmix>",
            );
            let valid_xml_path = temp
                .path()
                .join(format!("hybrid_{encoder_mode}_{input_name}.xml"));
            let valid_log_path = temp
                .path()
                .join(format!("hybrid_{encoder_mode}_{input_name}.log"));
            write_text(&valid_xml_path, &valid_xml);
            let valid_output = run_dee(&valid_xml_path, &valid_log_path);
            assert_success(
                &valid_output,
                &format!(
                    "allow_hybrid_downmix=true should succeed for {encoder_mode} {input_name}"
                ),
            );
        }

        let invalid_xml = replace_xml_value(
            &pcm_to_ddp_xml(
                &temp,
                "6ch.wav",
                &format!("invalid_hybrid_{encoder_mode}.{output_tag}"),
                output_tag,
                encoder_mode,
                default_downmix_for_mode(encoder_mode),
                data_rate,
            ),
            "<allow_hybrid_downmix>false</allow_hybrid_downmix>",
            "<allow_hybrid_downmix>bogus</allow_hybrid_downmix>",
        );
        let invalid_xml_path = temp
            .path()
            .join(format!("invalid_hybrid_{encoder_mode}.xml"));
        let invalid_log_path = temp
            .path()
            .join(format!("invalid_hybrid_{encoder_mode}.log"));
        write_text(&invalid_xml_path, &invalid_xml);
        let invalid_output = run_dee(&invalid_xml_path, &invalid_log_path);
        assert_failure_contains(
            &invalid_output,
            "allow_hybrid_downmix",
            &format!("allow_hybrid_downmix invalid override should fail for {encoder_mode}"),
        );
    }
}

#[test]
#[ignore = "requires local dee + ffmpeg runtime"]
fn pcm_ddp_representative_misc_smoke_matches_runtime() {
    require_command("dee");
    require_command("ffmpeg");

    let temp = TempDir::new().expect("create temp dir");
    fs::create_dir_all(temp.path().join("in")).expect("create in dir");
    fs::create_dir_all(temp.path().join("out")).expect("create out dir");
    fs::create_dir_all(temp.path().join("tmp")).expect("create tmp dir");
    generate_pcm_6ch_input(&temp);

    for encoder_mode in ["dd", "ddp", "ddp71", "bluray"] {
        let output_tag = output_tag_for_mode(encoder_mode);
        let data_rate = default_data_rate_for_mode(encoder_mode);
        let base_xml = pcm_to_ddp_xml(
            &temp,
            "6ch.wav",
            &format!("misc_{encoder_mode}.{output_tag}"),
            output_tag,
            encoder_mode,
            default_downmix_for_mode(encoder_mode),
            data_rate,
        );

        let cases = [
            (
                "start_alt",
                replace_xml_value(
                    &base_xml,
                    "<start>first_frame_of_action</start>",
                    "<start>00:00:00.0</start>",
                ),
            ),
            (
                "end_alt",
                replace_xml_value(&base_xml, "<end>end_of_file</end>", "<end>00:00:01.0</end>"),
            ),
            (
                "prepend_silence_alt",
                replace_xml_value(
                    &base_xml,
                    "<prepend_silence_duration>0.0</prepend_silence_duration>",
                    "<prepend_silence_duration>0f</prepend_silence_duration>",
                ),
            ),
            (
                "append_silence_alt",
                replace_xml_value(
                    &base_xml,
                    "<append_silence_duration>0.0</append_silence_duration>",
                    "<append_silence_duration>0f</append_silence_duration>",
                ),
            ),
            (
                "line_mode_drc_alt",
                replace_xml_value(
                    &base_xml,
                    "<line_mode_drc_profile>film_light</line_mode_drc_profile>",
                    "<line_mode_drc_profile>speech</line_mode_drc_profile>",
                ),
            ),
            (
                "rf_mode_drc_alt",
                replace_xml_value(
                    &base_xml,
                    "<rf_mode_drc_profile>film_light</rf_mode_drc_profile>",
                    "<rf_mode_drc_profile>speech</rf_mode_drc_profile>",
                ),
            ),
            (
                "custom_dialnorm_alt",
                replace_xml_value(
                    &base_xml,
                    "<custom_dialnorm>0</custom_dialnorm>",
                    "<custom_dialnorm>-31</custom_dialnorm>",
                ),
            ),
            (
                "loro_center_mix_alt",
                replace_xml_value(
                    &base_xml,
                    "<loro_center_mix_level>-3</loro_center_mix_level>",
                    "<loro_center_mix_level>0</loro_center_mix_level>",
                ),
            ),
            (
                "loro_surround_mix_alt",
                replace_xml_value(
                    &base_xml,
                    "<loro_surround_mix_level>-3</loro_surround_mix_level>",
                    "<loro_surround_mix_level>-6</loro_surround_mix_level>",
                ),
            ),
            (
                "ltrt_center_mix_alt",
                replace_xml_value(
                    &base_xml,
                    "<ltrt_center_mix_level>-3</ltrt_center_mix_level>",
                    "<ltrt_center_mix_level>0</ltrt_center_mix_level>",
                ),
            ),
            (
                "ltrt_surround_mix_alt",
                replace_xml_value(
                    &base_xml,
                    "<ltrt_surround_mix_level>-3</ltrt_surround_mix_level>",
                    "<ltrt_surround_mix_level>-6</ltrt_surround_mix_level>",
                ),
            ),
        ];

        for (name, xml) in cases {
            let xml_path = temp.path().join(format!("misc_{encoder_mode}_{name}.xml"));
            let log_path = temp.path().join(format!("misc_{encoder_mode}_{name}.log"));
            write_text(&xml_path, &xml);
            let output = run_dee(&xml_path, &log_path);
            assert_success(
                &output,
                &format!("pcm_ddp representative smoke mode={encoder_mode} case={name}"),
            );
        }
    }
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
