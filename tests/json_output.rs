mod common;

use std::path::Path;

use common::{create_mono_wav_stems, create_test_wav_inputs};
use dee_config_gen::{
    JobFile, RenderFormat, ResolveOptions, config::InputsSpec, load_job_file, render_config,
    resolve_job,
};
use serde_json::Value;

fn resolve_from_example(path: &str) -> dee_config_gen::ResolvedJob {
    let spec = load_job_file(Path::new(path)).unwrap();
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap()
}

fn resolve_job_file(spec: JobFile) -> dee_config_gen::ResolvedJob {
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap()
}

fn render_json(job: &dee_config_gen::ResolvedJob) -> String {
    render_config(job, RenderFormat::Json).expect("render json")
}

fn assert_valid_json(rendered: &str) {
    let parsed: Value = serde_json::from_str(rendered).expect("parse rendered json");
    assert!(parsed.get("job_config").is_some());
}

fn normalize_json_snapshot(rendered: &str) -> String {
    fn normalize_value(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for child in map.values_mut() {
                    normalize_value(child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    normalize_value(child);
                }
            }
            Value::String(text) => {
                if !text.contains("Y:/var/folders/") {
                    return;
                }

                let Some(stripped) = text.strip_prefix('\"').and_then(|s| s.strip_suffix('\"'))
                else {
                    return;
                };
                let Some(last_component) = stripped.rsplit('/').next() else {
                    return;
                };
                *text = format!("\"Y:/tmp/{last_component}\"");
            }
            _ => {}
        }
    }

    let mut parsed: Value = serde_json::from_str(rendered).expect("parse json snapshot");
    normalize_value(&mut parsed);
    serde_json::to_string_pretty(&parsed).expect("serialize normalized json")
}

fn resolve_thd_wav_list_example() -> dee_config_gen::ResolvedJob {
    let (_temp, storage_path, file_names) = create_mono_wav_stems(6);
    let mut spec = load_job_file(Path::new("examples/thd_wav_list_single.mlp.yaml")).unwrap();
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;
    resolve_job_file(spec)
}

fn resolve_thd_atmos_wav_example() -> dee_config_gen::ResolvedJob {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_wav_temp, wav_storage, wav_files) = create_test_wav_inputs(6, 16, 1);
    let mut spec = load_job_file(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).unwrap();
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::config::IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: Some(dee_config_gen::config::IoSpec {
            storage_path: wav_storage,
            file_names: wav_files,
        }),
        wav_list: None,
    });
    resolve_job_file(spec)
}

fn resolve_thd_atmos_wav_list_example() -> dee_config_gen::ResolvedJob {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_stems_temp, stem_storage, stem_files) = create_mono_wav_stems(6);
    let mut spec = load_job_file(Path::new("examples/thd_atmos_wav_list_single.mlp.yaml")).unwrap();
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::config::IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: None,
        wav_list: Some(dee_config_gen::config::IoSpec {
            storage_path: stem_storage,
            file_names: stem_files,
        }),
    });
    resolve_job_file(spec)
}

#[test]
fn atmos_streaming_json_matches_snapshot() {
    let rendered = render_json(&resolve_from_example(
        "examples/atmos_ec3_single.streaming.yaml",
    ));
    let expected = include_str!("fixtures/atmos_ec3_single.streaming.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn atmos_streaming_json_is_valid_json() {
    let rendered = render_json(&resolve_from_example(
        "examples/atmos_ec3_single.streaming.yaml",
    ));
    assert_valid_json(&rendered);
}

#[test]
fn pcm_ddp_dd_json_matches_snapshot() {
    let rendered = render_json(&resolve_from_example("examples/pcm_ddp_single.dd.yaml"));
    let expected = include_str!("fixtures/pcm_ddp_single.dd.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn pcm_ddp_dd_json_is_valid_json() {
    let rendered = render_json(&resolve_from_example("examples/pcm_ddp_single.dd.yaml"));
    assert_valid_json(&rendered);
}

#[test]
fn thd_json_matches_snapshot() {
    let rendered = render_json(&resolve_from_example("examples/thd_single.mlp.yaml"));
    let expected = include_str!("fixtures/thd_single.mlp.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn thd_json_is_valid_json() {
    let rendered = render_json(&resolve_from_example("examples/thd_single.mlp.yaml"));
    assert_valid_json(&rendered);
}

#[test]
fn thd_wav_json_matches_snapshot() {
    let rendered = render_json(&resolve_from_example("examples/thd_wav_single.mlp.yaml"));
    let expected = include_str!("fixtures/thd_wav_single.mlp.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn thd_wav_json_is_valid_json() {
    let rendered = render_json(&resolve_from_example("examples/thd_wav_single.mlp.yaml"));
    assert_valid_json(&rendered);
}

#[test]
fn thd_wav_list_json_matches_snapshot() {
    let rendered = normalize_json_snapshot(&render_json(&resolve_thd_wav_list_example()));
    let expected = normalize_json_snapshot(include_str!("fixtures/thd_wav_list_single.mlp.json"));
    assert_eq!(rendered, expected);
}

#[test]
fn thd_wav_list_json_is_valid_json() {
    let rendered = render_json(&resolve_thd_wav_list_example());
    assert_valid_json(&rendered);
}

#[test]
fn thd_atmos_wav_json_matches_snapshot() {
    let rendered = normalize_json_snapshot(&render_json(&resolve_thd_atmos_wav_example()));
    let expected = normalize_json_snapshot(include_str!("fixtures/thd_atmos_wav_single.mlp.json"));
    assert_eq!(rendered, expected);
}

#[test]
fn thd_atmos_wav_json_is_valid_json() {
    let rendered = render_json(&resolve_thd_atmos_wav_example());
    assert_valid_json(&rendered);
}

#[test]
fn thd_atmos_wav_list_json_matches_snapshot() {
    let rendered = normalize_json_snapshot(&render_json(&resolve_thd_atmos_wav_list_example()));
    let expected =
        normalize_json_snapshot(include_str!("fixtures/thd_atmos_wav_list_single.mlp.json"));
    assert_eq!(rendered, expected);
}

#[test]
fn thd_atmos_wav_list_json_is_valid_json() {
    let rendered = render_json(&resolve_thd_atmos_wav_list_example());
    assert_valid_json(&rendered);
}
