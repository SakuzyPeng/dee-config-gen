use std::process::Command;

mod common;

use dee_config_gen::{
    IoSpec, ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, JobSpec, OutputContainer},
};
use tempfile::TempDir;

fn xml_job_name(output_name: &str) -> String {
    let stem = output_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(output_name);
    format!("{stem}.xml")
}

fn render_ac4_ims_pcm_wav_xml(
    temp: &TempDir,
    input_storage_path: &str,
    input_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let mut job = read_job(&common::repo_root().join("examples/ac4_ims_pcm_single.ac4.yaml"))
        .unwrap_or_else(|err| panic!("load ac4 ims pcm example: {err}"));
    let wav = job
        .inputs
        .as_mut()
        .expect("inputs")
        .wav
        .as_mut()
        .expect("wav");
    wav.storage_path = input_storage_path.to_string();
    wav.file_names = input_file_names;
    job.output.storage_path = common::workspace_relative_unix_path(&temp.path().join("out"));
    job.output.file_names = vec![output_name.to_string()];
    job.misc.temp_dir = common::workspace_relative_unix_path(&temp.path().join("tmp"));
    mutate(&mut job);

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .expect("resolve ac4 ims pcm wav");

    render_xml(&resolved)
}

fn render_ac4_ims_pcm_wav_list_xml(
    temp: &TempDir,
    input_storage_path: &str,
    input_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let mut job = read_job(&common::repo_root().join("examples/ac4_ims_pcm_single.ac4.yaml"))
        .unwrap_or_else(|err| panic!("load ac4 ims pcm example: {err}"));
    let inputs = job.inputs.as_mut().expect("inputs");
    inputs.wav = None;
    inputs.wav_list = Some(IoSpec {
        storage_path: input_storage_path.to_string(),
        file_names: input_file_names,
    });
    job.output.storage_path = common::workspace_relative_unix_path(&temp.path().join("out"));
    job.output.file_names = vec![output_name.to_string()];
    job.misc.temp_dir = common::workspace_relative_unix_path(&temp.path().join("tmp"));
    mutate(&mut job);

    let resolved = resolve_job(
        job,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .expect("resolve ac4 ims pcm wav_list");

    render_xml(&resolved)
}

fn run_pcm_wav_case(context: &str, output_name: &str, mutate: impl FnOnce(&mut JobSpec)) {
    let temp = common::runtime_tempdir("ac4_ims_pcm_");
    common::create_temp_layout(&temp);
    let (input_temp, input_storage_path, input_file_names) = common::runtime_pcm_wav_input_51();

    let xml = render_ac4_ims_pcm_wav_xml(
        &temp,
        &input_storage_path,
        input_file_names,
        output_name,
        mutate,
    );
    let xml_name = xml_job_name(output_name);
    let output = common::run_rendered_xml(&temp, &xml_name, &xml);
    common::assert_success(&output, context);
    common::assert_output_exists(&temp.path().join("out").join(output_name), context);
    assert!(
        input_temp.path().exists(),
        "{context}: input fixture tempdir should still exist while case is running"
    );
}

fn run_pcm_wav_case_expect_failure(
    context: &str,
    output_name: &str,
    expected_fragment: &str,
    mutate: impl FnOnce(&mut JobSpec),
) {
    let temp = common::runtime_tempdir("ac4_ims_pcm_");
    common::create_temp_layout(&temp);
    let (input_temp, input_storage_path, input_file_names) = common::runtime_pcm_wav_input_51();

    let xml = render_ac4_ims_pcm_wav_xml(
        &temp,
        &input_storage_path,
        input_file_names,
        output_name,
        mutate,
    );
    let xml_name = xml_job_name(output_name);
    let output = common::run_rendered_xml(&temp, &xml_name, &xml);
    common::assert_failure(&output, context, expected_fragment);
    assert!(
        input_temp.path().exists(),
        "{context}: input fixture tempdir should still exist while case is running"
    );
}

fn run_pcm_wav_list_case(
    context: &str,
    input_storage_path: &str,
    input_file_names: Vec<String>,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) {
    let temp = common::runtime_tempdir("ac4_ims_pcm_");
    common::create_temp_layout(&temp);

    let xml = render_ac4_ims_pcm_wav_list_xml(
        &temp,
        input_storage_path,
        input_file_names,
        output_name,
        mutate,
    );
    let xml_name = xml_job_name(output_name);
    let output = common::run_rendered_xml(&temp, &xml_name, &xml);
    common::assert_success(&output, context);
    common::assert_output_exists(&temp.path().join("out").join(output_name), context);
}

fn set_drc_profile(job: &mut JobSpec, field: &str, value: &str) {
    match field {
        "ddp_drc_profile" => job.filter.ddp_drc_profile = Some(value.to_string()),
        "flat_panel_drc_profile" => job.filter.flat_panel_drc_profile = Some(value.to_string()),
        "home_theatre_drc_profile" => {
            job.filter.home_theatre_drc_profile = Some(value.to_string());
        }
        "portable_hp_drc_profile" => {
            job.filter.portable_hp_drc_profile = Some(value.to_string());
        }
        "portable_spkr_drc_profile" => {
            job.filter.portable_spkr_drc_profile = Some(value.to_string());
        }
        other => panic!("unexpected drc field: {other}"),
    }
}

fn run_native_mp4muxer(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
) -> std::process::Output {
    let mp4muxer = common::find_native_mp4muxer().expect("required native mp4muxer was not found");
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

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_runtime_smoke_matches_runtime() {
    common::runtime_preflight(false);

    run_pcm_wav_case("ac4 ims pcm smoke", "smoke.ac4", |_| {});
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_mp4_runtime_smoke_matches_runtime() {
    common::runtime_preflight(false);

    run_pcm_wav_case("ac4 ims pcm mp4 direct feasibility", "smoke.mp4", |job| {
        job.output.container = OutputContainer::Mp4;
    });
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package and native mp4muxer"]
fn ac4_ims_pcm_manual_mux_mp4_from_ac4_matches_runtime() {
    common::runtime_preflight(true);

    let temp = common::runtime_tempdir("ac4_ims_pcm_manual_mux_");
    common::create_temp_layout(&temp);
    let (input_temp, input_storage_path, input_file_names) = common::runtime_pcm_wav_input_51();
    let xml = render_ac4_ims_pcm_wav_xml(
        &temp,
        &input_storage_path,
        input_file_names,
        "manual_mux_source.ac4",
        |_| {},
    );

    let output = common::run_rendered_xml(&temp, "manual_mux_source.xml", &xml);
    common::assert_success(&output, "ac4 ims pcm manual mux source encode");
    assert!(
        input_temp.path().exists(),
        "ac4 ims pcm manual mux source encode: input fixture tempdir should still exist while case is running"
    );

    let ac4_path = temp.path().join("out").join("manual_mux_source.ac4");
    common::assert_output_exists(&ac4_path, "ac4 ims pcm manual mux source encode");

    let mp4_path = temp.path().join("out").join("manual_mux_target.mp4");
    let mux_output = run_native_mp4muxer(&ac4_path, &mp4_path);
    let combined = format!(
        "{}\n{}",
        String::from_utf8_lossy(&mux_output.stdout),
        String::from_utf8_lossy(&mux_output.stderr)
    );
    assert!(
        mux_output.status.success(),
        "native mp4muxer should succeed for AC-4 source, got:\n{combined}"
    );
    common::assert_output_exists(&mp4_path, "ac4 ims pcm manual mux target");
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_wav_list_runtime_smoke_matches_runtime() {
    common::runtime_preflight(false);

    let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
    run_pcm_wav_list_case(
        "ac4 ims pcm wav_list smoke",
        &stem_storage,
        stem_files,
        "wav_list_smoke.ac4",
        |_| {},
    );
    assert!(
        stems_temp.path().exists(),
        "ac4 ims pcm wav_list smoke: stem fixture tempdir should still exist while case is running"
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_core_nondefault_regression_matches_runtime() {
    common::runtime_preflight(false);

    run_pcm_wav_case(
        "ac4 ims pcm core nondefault regression",
        "core_nondefault.ac4",
        |job| {
            job.filter = FilterOverrides {
                data_rate: Some(72),
                ac4_frame_rate: Some("24".to_string()),
                language: Some("eng".to_string()),
                encoding_profile: Some("ims_music".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_timecode_frame_rate_matrix_matches_runtime() {
    common::runtime_preflight(false);

    for timecode_frame_rate in ["not_indicated", "23.976", "24", "25", "29.97", "30"] {
        let slug = timecode_frame_rate.replace('.', "_");
        run_pcm_wav_case(
            &format!("ac4 ims pcm timecode_frame_rate={timecode_frame_rate}"),
            &format!("timecode_frame_rate_{slug}.ac4"),
            |job| {
                job.filter.timecode_frame_rate = Some(timecode_frame_rate.to_string());
            },
        );
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_wav_time_matrix_matches_runtime() {
    common::runtime_preflight(false);

    for (slug, frame_rate, start, end) in [
        ("file_position_frames", "24", "00:00:00:00", "00:00:00:20"),
        ("file_position_decimal", "24", "00:00:00.00", "00:00:00.80"),
        (
            "file_position_drop_frame",
            "29.97",
            "00:00:00:00df",
            "00:00:00:10df",
        ),
    ] {
        run_pcm_wav_case(
            &format!("ac4 ims pcm wav boundary timecodes {slug}"),
            &format!("wav_boundary_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    time_base: Some("file_position".to_string()),
                    timecode_frame_rate: Some(frame_rate.to_string()),
                    start: Some(start.to_string()),
                    end: Some(end.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for prepend in ["0", "0.5", "2.0"] {
        let slug = prepend.replace('.', "_");
        run_pcm_wav_case(
            &format!("ac4 ims pcm wav prepend_silence_duration={prepend}"),
            &format!("wav_prepend_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    prepend_silence_duration: Some(prepend.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for append in ["0", "0.5", "2.0"] {
        let slug = append.replace('.', "_");
        run_pcm_wav_case(
            &format!("ac4 ims pcm wav append_silence_duration={append}"),
            &format!("wav_append_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    append_silence_duration: Some(append.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    run_pcm_wav_case(
        "ac4 ims pcm embedded_timecode via input offset/ffoa",
        "embedded_timecode_probe.ac4",
        |job| {
            job.filter = FilterOverrides {
                input_timecode_frame_rate: Some("24".to_string()),
                offset: Some("01:00:00:00".to_string()),
                ffoa: Some("01:00:00:00".to_string()),
                time_base: Some("embedded_timecode".to_string()),
                timecode_frame_rate: Some("24".to_string()),
                start: Some("01:00:00:00".to_string()),
                end: Some("01:00:00:20".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_control_param_matrix_matches_runtime() {
    common::runtime_preflight(false);

    for metering_mode in ["1770-1", "1770-2", "1770-3", "1770-4", "LeqA"] {
        let slug = metering_mode.to_lowercase().replace('.', "_");
        run_pcm_wav_case(
            &format!("ac4 ims pcm metering_mode={metering_mode}"),
            &format!("metering_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    metering_mode: Some(metering_mode.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for dialogue_intelligence in [true, false] {
        let slug = if dialogue_intelligence {
            "true"
        } else {
            "false"
        };
        run_pcm_wav_case(
            &format!("ac4 ims pcm dialogue_intelligence={dialogue_intelligence}"),
            &format!("dialogue_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    dialogue_intelligence: Some(dialogue_intelligence),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for speech_threshold in [0_u8, 15, 100] {
        run_pcm_wav_case(
            &format!("ac4 ims pcm speech_threshold={speech_threshold}"),
            &format!("speech_threshold_{speech_threshold}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    speech_threshold: Some(speech_threshold),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for ims_legacy_presentation in [false, true] {
        let slug = if ims_legacy_presentation {
            "true"
        } else {
            "false"
        };
        run_pcm_wav_case(
            &format!("ac4 ims pcm ims_legacy_presentation={ims_legacy_presentation}"),
            &format!("ims_legacy_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    ims_legacy_presentation: Some(ims_legacy_presentation),
                    ..FilterOverrides::default()
                };
            },
        );
    }

    for iframe_interval in [0_u16, 24, 1000] {
        run_pcm_wav_case(
            &format!("ac4 ims pcm iframe_interval={iframe_interval}"),
            &format!("iframe_interval_{iframe_interval}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    iframe_interval: Some(iframe_interval),
                    ..FilterOverrides::default()
                };
            },
        );
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_rejects_invalid_iframe_interval_value_1() {
    common::runtime_preflight(false);

    run_pcm_wav_case_expect_failure(
        "ac4 ims pcm iframe_interval=1",
        "iframe_interval_1.ac4",
        "Invalid iframe_interval value: 1",
        |job| {
            job.filter = FilterOverrides {
                iframe_interval: Some(1),
                ..FilterOverrides::default()
            };
        },
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_drc_profiles_matrix_matches_runtime() {
    common::runtime_preflight(false);

    let profile_values = [
        "film_standard",
        "film_light",
        "music_standard",
        "music_light",
        "speech",
        "none",
    ];

    for field in [
        "ddp_drc_profile",
        "flat_panel_drc_profile",
        "home_theatre_drc_profile",
        "portable_hp_drc_profile",
        "portable_spkr_drc_profile",
    ] {
        for value in profile_values {
            let slug = value.replace('.', "_");
            run_pcm_wav_case(
                &format!("ac4 ims pcm {field}={value}"),
                &format!("{field}_{slug}.ac4"),
                |job| {
                    set_drc_profile(job, field, value);
                },
            );
        }
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_wav_list_parity_regression_matches_runtime() {
    common::runtime_preflight(false);

    let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
    run_pcm_wav_list_case(
        "ac4 ims pcm wav_list parity regression",
        &stem_storage,
        stem_files,
        "wav_list_parity.ac4",
        |job| {
            job.filter = FilterOverrides {
                metering_mode: Some("1770-3".to_string()),
                dialogue_intelligence: Some(false),
                speech_threshold: Some(42),
                data_rate: Some(72),
                time_base: Some("file_position".to_string()),
                timecode_frame_rate: Some("24".to_string()),
                start: Some("00:00:00:00".to_string()),
                end: Some("00:00:00:20".to_string()),
                prepend_silence_duration: Some("0.5".to_string()),
                append_silence_duration: Some("0.5".to_string()),
                ac4_frame_rate: Some("24".to_string()),
                ims_legacy_presentation: Some(true),
                iframe_interval: Some(24),
                language: Some("eng".to_string()),
                encoding_profile: Some("ims_music".to_string()),
                ddp_drc_profile: Some("speech".to_string()),
                flat_panel_drc_profile: Some("music_standard".to_string()),
                home_theatre_drc_profile: Some("film_standard".to_string()),
                portable_hp_drc_profile: Some("music_light".to_string()),
                portable_spkr_drc_profile: Some("none".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
    assert!(
        stems_temp.path().exists(),
        "ac4 ims pcm wav_list parity regression: stem fixture tempdir should still exist while case is running"
    );
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_pcm_wav_list_time_matrix_matches_runtime() {
    common::runtime_preflight(false);

    let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
    for (slug, frame_rate, start, end) in [
        ("file_position_frames", "24", "00:00:00:00", "00:00:00:20"),
        ("file_position_decimal", "24", "00:00:00.00", "00:00:00.80"),
        (
            "file_position_drop_frame",
            "29.97",
            "00:00:00:00df",
            "00:00:00:10df",
        ),
    ] {
        run_pcm_wav_list_case(
            &format!("ac4 ims pcm wav_list boundary timecodes {slug}"),
            &stem_storage,
            stem_files.clone(),
            &format!("wav_list_boundary_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    time_base: Some("file_position".to_string()),
                    timecode_frame_rate: Some(frame_rate.to_string()),
                    start: Some(start.to_string()),
                    end: Some(end.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }
    assert!(
        stems_temp.path().exists(),
        "ac4 ims pcm wav_list time matrix: stem fixture tempdir should still exist while case is running"
    );

    for prepend in ["0", "0.5", "2.0"] {
        let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
        let slug = prepend.replace('.', "_");
        run_pcm_wav_list_case(
            &format!("ac4 ims pcm wav_list prepend_silence_duration={prepend}"),
            &stem_storage,
            stem_files,
            &format!("wav_list_prepend_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    prepend_silence_duration: Some(prepend.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
        assert!(
            stems_temp.path().exists(),
            "ac4 ims pcm wav_list prepend_silence_duration={prepend}: stem fixture tempdir should still exist while case is running"
        );
    }

    for append in ["0", "0.5", "2.0"] {
        let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
        let slug = append.replace('.', "_");
        run_pcm_wav_list_case(
            &format!("ac4 ims pcm wav_list append_silence_duration={append}"),
            &stem_storage,
            stem_files,
            &format!("wav_list_append_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    append_silence_duration: Some(append.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
        assert!(
            stems_temp.path().exists(),
            "ac4 ims pcm wav_list append_silence_duration={append}: stem fixture tempdir should still exist while case is running"
        );
    }

    let (stems_temp, stem_storage, stem_files) = common::runtime_pcm_mono_stems(6);
    run_pcm_wav_list_case(
        "ac4 ims pcm wav_list embedded_timecode via input offset/ffoa",
        &stem_storage,
        stem_files,
        "wav_list_embedded_timecode_probe.ac4",
        |job| {
            job.filter = FilterOverrides {
                input_timecode_frame_rate: Some("24".to_string()),
                offset: Some("01:00:00:00".to_string()),
                ffoa: Some("01:00:00:00".to_string()),
                time_base: Some("embedded_timecode".to_string()),
                timecode_frame_rate: Some("24".to_string()),
                start: Some("01:00:00:00".to_string()),
                end: Some("01:00:00:20".to_string()),
                ..FilterOverrides::default()
            };
        },
    );
    assert!(
        stems_temp.path().exists(),
        "ac4 ims pcm wav_list embedded_timecode via input offset/ffoa: stem fixture tempdir should still exist while case is running"
    );
}
