use std::path::Path;

use dee_config_gen::{RenderFormat, ResolveOptions, load_job_file, render_config, resolve_job};
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

#[test]
fn atmos_streaming_json_matches_snapshot() {
    let rendered = render_config(
        &resolve_from_example("examples/atmos_ec3_single.streaming.yaml"),
        RenderFormat::Json,
    )
    .expect("render atmos json");
    let expected = include_str!("fixtures/atmos_ec3_single.streaming.json");
    assert_eq!(rendered.trim_end(), expected.trim_end());
}

#[test]
fn atmos_streaming_json_is_valid_json() {
    let rendered = render_config(
        &resolve_from_example("examples/atmos_ec3_single.streaming.yaml"),
        RenderFormat::Json,
    )
    .expect("render atmos json");
    let parsed: Value = serde_json::from_str(&rendered).expect("parse rendered json");
    assert!(parsed.get("job_config").is_some());
}

#[test]
fn pcm_ddp_json_output_is_rejected() {
    let err = render_config(
        &resolve_from_example("examples/pcm_ddp_single.dd.yaml"),
        RenderFormat::Json,
    )
    .expect_err("pcm json should be unsupported");
    assert!(err.to_string().contains("does not support JSON output"));
}
