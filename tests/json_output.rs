mod common;

use std::fs;
use std::path::Path;

use common::{create_mono_wav_stems, create_test_wav_inputs};
use dee_config_gen::{
    JobSpec, RenderFormat, ResolveOptions, read_job, render_config, resolve_job, spec::InputsSpec,
};
use serde_json::Value;

fn resolve_from_example(path: &str) -> dee_config_gen::ResolvedJob {
    let spec = read_job(Path::new(path)).unwrap();
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

fn resolve_job_file(spec: JobSpec) -> dee_config_gen::ResolvedJob {
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

fn create_named_test_wav_input(
    channel_count: usize,
    bits_per_sample: u16,
    file_name: &str,
) -> (tempfile::TempDir, String, Vec<String>) {
    let (temp, storage_path, file_names) =
        create_test_wav_inputs(channel_count, bits_per_sample, 1);
    let generated_name = file_names
        .first()
        .cloned()
        .expect("generated wav file name");
    let storage_dir = Path::new(&storage_path);
    fs::rename(
        storage_dir.join(&generated_name),
        storage_dir.join(file_name),
    )
    .expect("rename generated wav");
    (temp, storage_path, vec![file_name.to_string()])
}

fn assert_valid_json(rendered: &str) {
    let parsed: Value = serde_json::from_str(rendered).expect("parse rendered json");
    assert!(parsed.get("job_config").is_some());
}

fn normalize_wav_input_storage_path(rendered: &str) -> String {
    let mut parsed: Value = serde_json::from_str(rendered).expect("parse wav-input json snapshot");
    let path = parsed
        .get_mut("job_config")
        .and_then(|value| value.get_mut("input"))
        .and_then(|value| value.get_mut("audio"))
        .and_then(|value| value.get_mut("wav"))
        .and_then(|value| value.get_mut("storage"))
        .and_then(|value| value.get_mut("local"))
        .and_then(|value| value.get_mut("path"))
        .unwrap_or_else(|| panic!("missing job_config.input.audio.wav.storage.local.path"));
    *path = Value::String("\"Y:/fixture/testfiles\"".to_string());
    serde_json::to_string_pretty(&parsed).expect("serialize normalized wav-input json")
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
                let Some(stripped) = text.strip_prefix('\"').and_then(|s| s.strip_suffix('\"'))
                else {
                    return;
                };
                let Some(last_component) = stripped.rsplit('/').next() else {
                    return;
                };

                // Mixed-input fixtures use temp-generated storage roots for `inputs`/`stems`.
                // Normalize them by semantic leaf name instead of OS-specific temp prefixes.
                if matches!(last_component, "inputs" | "stems")
                    && stripped.starts_with("Y:/")
                    && stripped != format!("Y:/tmp/{last_component}")
                {
                    *text = format!("\"Y:/tmp/{last_component}\"");
                    return;
                }

                if stripped.contains("Y:/var/folders/") {
                    *text = format!("\"Y:/tmp/{last_component}\"");
                }
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
    let mut spec = read_job(Path::new("examples/thd_wav_list_single.mlp.yaml")).unwrap();
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;
    resolve_job_file(spec)
}

fn resolve_thd_atmos_wav_example() -> dee_config_gen::ResolvedJob {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_wav_temp, wav_storage, wav_files) = create_test_wav_inputs(6, 16, 1);
    let mut spec = read_job(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).unwrap();
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::spec::IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: Some(dee_config_gen::spec::IoSpec {
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
    let mut spec = read_job(Path::new("examples/thd_atmos_wav_list_single.mlp.yaml")).unwrap();
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(dee_config_gen::spec::IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: None,
        wav_list: Some(dee_config_gen::spec::IoSpec {
            storage_path: stem_storage,
            file_names: stem_files,
        }),
    });
    resolve_job_file(spec)
}

fn resolve_ac4_ims_atmos_example() -> dee_config_gen::ResolvedJob {
    resolve_from_example("examples/ac4_ims_atmos_single.ac4.yaml")
}

fn resolve_ac4_ims_atmos_mp4_example() -> dee_config_gen::ResolvedJob {
    resolve_from_example("examples/ac4_ims_atmos_single.mp4.yaml")
}

fn resolve_pcm_ddp_dd_example() -> dee_config_gen::ResolvedJob {
    let (_temp, storage_path, file_names) = create_named_test_wav_input(6, 16, "input_6ch.wav");
    let mut spec = read_job(Path::new("examples/pcm_ddp_single.dd.yaml")).unwrap();
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;
    resolve_job_file(spec)
}

fn resolve_ac4_ims_pcm_example() -> dee_config_gen::ResolvedJob {
    let (_temp, storage_path, file_names) = create_named_test_wav_input(6, 16, "input_6ch.wav");
    let mut spec = read_job(Path::new("examples/ac4_ims_pcm_single.ac4.yaml")).unwrap();
    spec.inputs.as_mut().expect("inputs").wav = Some(dee_config_gen::spec::IoSpec {
        storage_path,
        file_names,
    });
    resolve_job_file(spec)
}

fn resolve_ac4_ims_pcm_mp4_example() -> dee_config_gen::ResolvedJob {
    let (_temp, storage_path, file_names) = create_named_test_wav_input(6, 16, "input_6ch.wav");
    let mut spec = read_job(Path::new("examples/ac4_ims_pcm_single.mp4.yaml")).unwrap();
    spec.inputs.as_mut().expect("inputs").wav = Some(dee_config_gen::spec::IoSpec {
        storage_path,
        file_names,
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
    let rendered = normalize_wav_input_storage_path(&render_json(&resolve_pcm_ddp_dd_example()));
    let expected =
        normalize_wav_input_storage_path(include_str!("fixtures/pcm_ddp_single.dd.json"));
    assert_eq!(rendered, expected);
}

#[test]
fn pcm_ddp_dd_json_is_valid_json() {
    let rendered = render_json(&resolve_pcm_ddp_dd_example());
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

#[test]
fn ac4_ims_atmos_json_matches_snapshot() {
    let rendered = render_json(&resolve_ac4_ims_atmos_example());
    let expected = include_str!("fixtures/ac4_ims_atmos_single.ac4.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn ac4_ims_atmos_json_is_valid_json() {
    let rendered = render_json(&resolve_ac4_ims_atmos_example());
    assert_valid_json(&rendered);
}

#[test]
fn ac4_ims_atmos_mp4_json_matches_snapshot() {
    let rendered = render_json(&resolve_ac4_ims_atmos_mp4_example());
    let expected = include_str!("fixtures/ac4_ims_atmos_single.mp4.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn ac4_ims_pcm_json_matches_snapshot() {
    let rendered = normalize_wav_input_storage_path(&render_json(&resolve_ac4_ims_pcm_example()));
    let expected =
        normalize_wav_input_storage_path(include_str!("fixtures/ac4_ims_pcm_single.ac4.json"));
    assert_eq!(rendered, expected);
}

#[test]
fn ac4_ims_pcm_json_is_valid_json() {
    let rendered = render_json(&resolve_ac4_ims_pcm_example());
    assert_valid_json(&rendered);
}

#[test]
fn ac4_ims_pcm_mp4_json_matches_snapshot() {
    let rendered =
        normalize_wav_input_storage_path(&render_json(&resolve_ac4_ims_pcm_mp4_example()));
    let expected =
        normalize_wav_input_storage_path(include_str!("fixtures/ac4_ims_pcm_single.mp4.json"));
    assert_eq!(rendered, expected);
}
