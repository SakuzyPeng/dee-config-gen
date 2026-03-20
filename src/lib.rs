//! `dee-config-gen` can be used as a Rust library for parsing job specs,
//! resolving defaults and constraints, rendering DEE XML/JSON, and invoking
//! an external runner.
//!
//! Recommended file-based flow:
//!
//! ```rust
//! use dee_config_gen::{GenerateOptions, generate_config, read_job};
//! use std::{fs, time::{SystemTime, UNIX_EPOCH}};
//!
//! let unique = SystemTime::now()
//!     .duration_since(UNIX_EPOCH)?
//!     .as_nanos();
//! let path = std::env::temp_dir().join(format!("dee-config-gen-doc-{unique}.yaml"));
//! fs::write(
//!     &path,
//!     r#"
//! template_id: atmos_ec3_v1
//! profile: standard
//! job_mode: single
//! encode_mode: streaming
//! input:
//!   storage_path: ./input
//!   file_names: [testADM.wav]
//! output:
//!   storage_path: ./output
//!   file_names: [output.ec3]
//! misc:
//!   temp_dir: ./tmp
//! "#,
//! )?;
//!
//! let spec = read_job(&path)?;
//! let generated = generate_config(spec, &GenerateOptions::default())?;
//! assert!(generated.rendered.starts_with("<?xml version=\"1.0\"?>"));
//! fs::remove_file(path)?;
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! Recommended in-memory flow:
//!
//! ```rust
//! use dee_config_gen::{GenerateOptions, RenderFormat, generate_config, parse_job_str};
//!
//! let spec = parse_job_str(
//!     r#"{
//!       "template_id": "atmos_ec3_v1",
//!       "profile": "standard",
//!       "job_mode": "single",
//!       "encode_mode": "streaming",
//!       "input": {"storage_path": "./input", "file_names": ["input.wav"]},
//!       "output": {"storage_path": "./output", "file_names": ["output.ec3"]},
//!       "misc": {"temp_dir": "./tmp"}
//!     }"#,
//! )?;
//! let generated = generate_config(
//!     spec,
//!     &GenerateOptions {
//!         format: RenderFormat::Json,
//!         ..GenerateOptions::default()
//!     },
//! )?;
//! assert!(generated.rendered.contains("\"job_config\""));
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;

pub mod ffi;
pub mod render;
pub mod resolve;
pub mod runner;
pub mod spec;

mod media;
mod schema;
mod template;
#[cfg(test)]
mod test_support;

pub use render::{RenderFormat, render_config, render_xml};
pub use resolve::{
    InputMediaInfo, ResolveOptions, ResolvedFilter, ResolvedJob, ResolvedOutput, resolve_job,
};
pub use runner::{RunOptions, run_with_runner};
pub use spec::{
    Ac4OutputMode, DEFAULT_TEMPLATE_ID, EncodeMode, FilterOverrides, InputsSpec, IoSpec, JobMode,
    JobSpec, MiscSpec, OutputContainer, OutputSpec, Profile, RunSpec, parse_job_str, read_job,
};

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub resolve: ResolveOptions,
    pub format: RenderFormat,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            resolve: ResolveOptions::default(),
            format: RenderFormat::Xml,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeneratedConfig {
    pub resolved: ResolvedJob,
    pub rendered: String,
    pub format: RenderFormat,
}

pub fn validate_job(spec: JobSpec, options: &ResolveOptions) -> Result<ResolvedJob> {
    resolve_job(spec, options)
}

pub fn generate_config(spec: JobSpec, options: &GenerateOptions) -> Result<GeneratedConfig> {
    let resolved = validate_job(spec, &options.resolve)?;
    let rendered = render_config(&resolved, options.format)?;
    Ok(GeneratedConfig {
        resolved,
        rendered,
        format: options.format,
    })
}

pub fn run_job(spec: JobSpec, generate: &GenerateOptions, run: &RunOptions) -> Result<i32> {
    let generated = generate_config(spec, generate)?;
    run_with_runner(&generated.resolved, &generated.rendered, run)
}
