mod common;

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, JobSpec},
};
use tempfile::TempDir;

fn render_ac4_ims_atmos_xml(
    temp: &TempDir,
    output_name: &str,
    mutate: impl FnOnce(&mut JobSpec),
) -> String {
    let mut job = read_job(&common::repo_root().join("examples/ac4_ims_atmos_single.ac4.yaml"))
        .unwrap_or_else(|err| panic!("load ac4 ims atmos example: {err}"));
    job.inputs
        .as_mut()
        .expect("inputs")
        .atmos_mezz
        .as_mut()
        .expect("atmos_mezz")
        .storage_path = common::repo_root().join("testfiles").display().to_string();
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
    .expect("resolve ac4 ims atmos");

    render_xml(&resolved)
}

fn run_atmos_case(context: &str, output_name: &str, mutate: impl FnOnce(&mut JobSpec)) {
    let temp = TempDir::new().expect("create temp dir");
    common::create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, output_name, mutate);
    let xml_name = output_name.replace(".ac4", ".xml");
    let output = common::run_rendered_xml(&temp, &xml_name, &xml);
    common::assert_success(&output, context);
    common::assert_output_exists(&temp.path().join("out").join(output_name), context);
}

fn run_atmos_case_expect_failure(
    context: &str,
    output_name: &str,
    expected_fragment: &str,
    mutate: impl FnOnce(&mut JobSpec),
) {
    let temp = TempDir::new().expect("create temp dir");
    common::create_temp_layout(&temp);

    let xml = render_ac4_ims_atmos_xml(&temp, output_name, mutate);
    let xml_name = output_name.replace(".ac4", ".xml");
    let output = common::run_rendered_xml(&temp, &xml_name, &xml);
    common::assert_failure(&output, context, expected_fragment);
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

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_runtime_smoke_matches_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    run_atmos_case("ac4 ims atmos smoke", "smoke.ac4", |_| {});
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_core_nondefault_regression_matches_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    run_atmos_case(
        "ac4 ims atmos core nondefault regression",
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
fn ac4_ims_atmos_timecode_frame_rate_matrix_matches_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    for timecode_frame_rate in ["not_indicated", "23.976", "24", "25", "29.97", "30"] {
        let slug = timecode_frame_rate.replace('.', "_");
        run_atmos_case(
            &format!("ac4 ims atmos timecode_frame_rate={timecode_frame_rate}"),
            &format!("timecode_frame_rate_{slug}.ac4"),
            |job| {
                job.filter.timecode_frame_rate = Some(timecode_frame_rate.to_string());
            },
        );
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_boundary_timecodes_match_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    for (slug, time_base, frame_rate, start, end) in [
        (
            "file_position_frames",
            "file_position",
            "24",
            "00:00:00:00",
            "00:00:00:20",
        ),
        (
            "file_position_decimal",
            "file_position",
            "24",
            "00:00:00.00",
            "00:00:00.80",
        ),
        (
            "file_position_drop_frame",
            "file_position",
            "29.97",
            "00:00:00:00df",
            "00:00:00:10df",
        ),
        (
            "embedded_timecode_frames",
            "embedded_timecode",
            "24",
            "01:00:00:00",
            "01:00:00:20",
        ),
    ] {
        run_atmos_case(
            &format!("ac4 ims atmos boundary timecodes {slug}"),
            &format!("boundary_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    time_base: Some(time_base.to_string()),
                    timecode_frame_rate: Some(frame_rate.to_string()),
                    start: Some(start.to_string()),
                    end: Some(end.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_silence_durations_match_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    for prepend in ["0", "0.5", "2.0"] {
        let slug = prepend.replace('.', "_");
        run_atmos_case(
            &format!("ac4 ims atmos prepend_silence_duration={prepend}"),
            &format!("prepend_{slug}.ac4"),
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
        run_atmos_case(
            &format!("ac4 ims atmos append_silence_duration={append}"),
            &format!("append_{slug}.ac4"),
            |job| {
                job.filter = FilterOverrides {
                    append_silence_duration: Some(append.to_string()),
                    ..FilterOverrides::default()
                };
            },
        );
    }
}

#[test]
#[ignore = "requires local dee runtime with AC-4 package"]
fn ac4_ims_atmos_control_param_matrix_matches_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    for metering_mode in ["1770-1", "1770-2", "1770-3", "1770-4", "LeqA"] {
        let slug = metering_mode.to_lowercase().replace('.', "_");
        run_atmos_case(
            &format!("ac4 ims atmos metering_mode={metering_mode}"),
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
        let slug = if dialogue_intelligence { "true" } else { "false" };
        run_atmos_case(
            &format!("ac4 ims atmos dialogue_intelligence={dialogue_intelligence}"),
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
        run_atmos_case(
            &format!("ac4 ims atmos speech_threshold={speech_threshold}"),
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
        run_atmos_case(
            &format!("ac4 ims atmos ims_legacy_presentation={ims_legacy_presentation}"),
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
        run_atmos_case(
            &format!("ac4 ims atmos iframe_interval={iframe_interval}"),
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
fn ac4_ims_atmos_rejects_invalid_iframe_interval_value_1() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

    run_atmos_case_expect_failure(
        "ac4 ims atmos iframe_interval=1",
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
fn ac4_ims_atmos_drc_profiles_matrix_matches_runtime() {
    let _lock = common::runtime_suite_lock();
    common::require_command("dee");

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
            run_atmos_case(
                &format!("ac4 ims atmos {field}={value}"),
                &format!("{field}_{slug}.ac4"),
                |job| {
                    set_drc_profile(job, field, value);
                },
            );
        }
    }
}
