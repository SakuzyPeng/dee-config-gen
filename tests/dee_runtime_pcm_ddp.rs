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
    encoder_mode: &str,
    data_rate: u16,
) -> String {
    format!(
        "<?xml version=\"1.0\"?>\n\
<job_config>\n\
  <input><audio><wav version=\"1\"><file_name>{input_name}</file_name><timecode_frame_rate>not_indicated</timecode_frame_rate><offset>auto</offset><ffoa>auto</ffoa><storage><local><path>Z:{}</path></local></storage></wav></audio></input>\n\
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
        let xml = pcm_to_ddp_xml(&temp, "8ch.wav", &output_name, "ddp71", bitrate);
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
        let xml = pcm_to_ddp_xml(&temp, "8ch.wav", &output_name, "bluray", bitrate);
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
        let xml = pcm_to_ddp_xml(&temp, "6ch.wav", &output_name, "ddp71", bitrate);
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
        let xml = pcm_to_ddp_xml(&temp, "6ch.wav", &output_name, "bluray", bitrate);
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
