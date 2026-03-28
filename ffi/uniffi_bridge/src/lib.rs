use std::panic::{AssertUnwindSafe, catch_unwind};

use dee_config_gen::{
    RenderFormat as CoreRenderFormat, ResolveOptions as CoreResolveOptions, parse_job_str,
    render_config, validate_job as core_validate_job,
};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum RenderFormat {
    XML {},
    JSON {},
}

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    pub template_override: Option<String>,
    pub allow_fixed_override: bool,
    pub windows_drive: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub resolve: Option<ResolveOptions>,
    pub format: Option<RenderFormat>,
}

#[derive(Debug, Clone)]
pub struct ValidateOutput {
    pub template_id: String,
    pub profile: String,
    pub job_mode: String,
    pub encode_mode: String,
    pub output_container: String,
}

#[derive(Debug, Clone)]
pub struct GenerateOutput {
    pub rendered_config: String,
    pub format: RenderFormat,
    pub template_id: String,
    pub output_container: String,
}

#[allow(non_camel_case_types)]
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("INVALID_ARGUMENT: {message}")]
    INVALID_ARGUMENT { message: String },
    #[error("PARSE_ERROR: {message}")]
    PARSE_ERROR { message: String },
    #[error("RESOLVE_ERROR: {message}")]
    RESOLVE_ERROR { message: String },
    #[error("RENDER_ERROR: {message}")]
    RENDER_ERROR { message: String },
    #[error("INTERNAL_ERROR: {message}")]
    INTERNAL_ERROR { message: String },
    #[error("PANIC: {message}")]
    PANIC { message: String },
}

pub fn validate_job(
    job_text: String,
    options: Option<ResolveOptions>,
) -> Result<ValidateOutput, BridgeError> {
    with_panic_boundary(|| {
        if job_text.is_empty() {
            return Err(invalid_argument("job_text must not be empty"));
        }

        let resolve_options = to_core_resolve_options(options)?;
        let spec = parse_job_str(&job_text).map_err(|e| parse_error(e.to_string()))?;
        let resolved =
            core_validate_job(spec, &resolve_options).map_err(|e| resolve_error(e.to_string()))?;

        Ok(ValidateOutput {
            template_id: resolved.template_id,
            profile: resolved.profile.as_str().to_string(),
            job_mode: resolved.job_mode.as_str().to_string(),
            encode_mode: resolved.encode_mode.as_str().to_string(),
            output_container: resolved.output.container.as_str().to_string(),
        })
    })
}

pub fn generate_config(
    job_text: String,
    options: Option<GenerateOptions>,
) -> Result<GenerateOutput, BridgeError> {
    with_panic_boundary(|| {
        if job_text.is_empty() {
            return Err(invalid_argument("job_text must not be empty"));
        }

        let (resolve_options, format) = to_core_generate_options(options)?;
        let spec = parse_job_str(&job_text).map_err(|e| parse_error(e.to_string()))?;
        let resolved =
            core_validate_job(spec, &resolve_options).map_err(|e| resolve_error(e.to_string()))?;
        let rendered = render_config(&resolved, core_render_format(format))
            .map_err(|e| render_error(e.to_string()))?;

        Ok(GenerateOutput {
            rendered_config: rendered,
            format,
            template_id: resolved.template_id,
            output_container: resolved.output.container.as_str().to_string(),
        })
    })
}

fn to_core_generate_options(
    input: Option<GenerateOptions>,
) -> Result<(CoreResolveOptions, RenderFormat), BridgeError> {
    let Some(input) = input else {
        return Ok((CoreResolveOptions::default(), RenderFormat::XML {}));
    };

    let resolve = to_core_resolve_options(input.resolve)?;
    let format = input.format.unwrap_or(RenderFormat::XML {});
    Ok((resolve, format))
}

fn to_core_resolve_options(
    input: Option<ResolveOptions>,
) -> Result<CoreResolveOptions, BridgeError> {
    let Some(input) = input else {
        return Ok(CoreResolveOptions::default());
    };

    Ok(CoreResolveOptions {
        template_override: input.template_override,
        allow_fixed_override: input.allow_fixed_override,
        windows_drive: parse_windows_drive(input.windows_drive)?,
    })
}

fn parse_windows_drive(value: Option<String>) -> Result<char, BridgeError> {
    let Some(raw) = value else {
        return Ok('Y');
    };

    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok('Y');
    }
    if trimmed.len() != 1 {
        return Err(invalid_argument(format!(
            "windows_drive must be a single ASCII letter, got '{raw}'"
        )));
    }
    let mut chars = trimmed.chars();
    let ch = chars.next().unwrap_or('Y');
    if !ch.is_ascii_alphabetic() {
        return Err(invalid_argument(format!(
            "windows_drive must be a single ASCII letter, got '{raw}'"
        )));
    }
    Ok(ch.to_ascii_uppercase())
}

fn core_render_format(value: RenderFormat) -> CoreRenderFormat {
    match value {
        RenderFormat::XML { .. } => CoreRenderFormat::Xml,
        RenderFormat::JSON { .. } => CoreRenderFormat::Json,
    }
}

fn with_panic_boundary<T>(f: impl FnOnce() -> Result<T, BridgeError>) -> Result<T, BridgeError> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(result) => result,
        Err(payload) => Err(panic_error(panic_payload_to_string(payload))),
    }
}

fn panic_payload_to_string(payload: Box<dyn std::any::Any + Send>) -> String {
    let any = payload.as_ref();
    if let Some(value) = any.downcast_ref::<&str>() {
        return format!("panic across UniFFI boundary: {value}");
    }
    if let Some(value) = any.downcast_ref::<String>() {
        return format!("panic across UniFFI boundary: {value}");
    }
    "panic across UniFFI boundary".to_string()
}

fn invalid_argument(message: impl Into<String>) -> BridgeError {
    BridgeError::INVALID_ARGUMENT {
        message: message.into(),
    }
}

fn parse_error(message: impl Into<String>) -> BridgeError {
    BridgeError::PARSE_ERROR {
        message: message.into(),
    }
}

fn resolve_error(message: impl Into<String>) -> BridgeError {
    BridgeError::RESOLVE_ERROR {
        message: message.into(),
    }
}

fn render_error(message: impl Into<String>) -> BridgeError {
    BridgeError::RENDER_ERROR {
        message: message.into(),
    }
}

fn panic_error(message: impl Into<String>) -> BridgeError {
    BridgeError::PANIC {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job(template_id: &str) -> String {
        format!(
            r#"
template_id: {template_id}
profile: standard
job_mode: single
encode_mode: streaming
input:
  storage_path: ./input
  file_names: [testADM.wav]
output:
  storage_path: ./output
  file_names: [output.ec3]
misc:
  temp_dir: ./tmp
"#
        )
    }

    #[test]
    fn validate_defaults_follow_core_defaults() {
        let out = validate_job(sample_job("atmos_ec3_v1"), None).expect("validate should succeed");
        assert_eq!(out.template_id, "atmos_ec3_v1");
        assert_eq!(out.profile, "standard");
    }

    #[test]
    fn generate_json_succeeds_for_supported_template() {
        let out = generate_config(
            sample_job("atmos_ec3_v1"),
            Some(GenerateOptions {
                resolve: None,
                format: Some(RenderFormat::JSON {}),
            }),
        )
        .expect("generate should succeed");
        assert!(out.rendered_config.contains("\"job_config\""));
        assert!(matches!(out.format, RenderFormat::JSON { .. }));
    }

    #[test]
    fn parse_error_is_mapped() {
        let err = validate_job("template_id: [".to_string(), None).expect_err("parse should fail");
        assert!(matches!(err, BridgeError::PARSE_ERROR { .. }));
    }

    #[test]
    fn resolve_error_is_mapped() {
        let err = validate_job(sample_job("definitely_unknown_template"), None)
            .expect_err("resolve should fail");
        assert!(matches!(err, BridgeError::RESOLVE_ERROR { .. }));
    }

    #[test]
    fn render_error_is_mapped() {
        let err = render_error("render failed");
        assert!(matches!(err, BridgeError::RENDER_ERROR { .. }));
    }

    #[test]
    fn panic_containment_is_mapped() {
        let err = with_panic_boundary::<()>(|| panic!("boom")).expect_err("panic should map");
        assert!(matches!(err, BridgeError::PANIC { .. }));
    }

    #[test]
    fn invalid_windows_drive_is_invalid_argument() {
        let err = validate_job(
            sample_job("atmos_ec3_v1"),
            Some(ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: Some("YY".to_string()),
            }),
        )
        .expect_err("invalid windows drive should fail");
        assert!(matches!(err, BridgeError::INVALID_ARGUMENT { .. }));
    }

    #[test]
    fn generate_defaults_to_xml() {
        let out = generate_config(sample_job("atmos_ec3_v1"), None)
            .expect("generate default should work");
        assert!(matches!(out.format, RenderFormat::XML { .. }));
        assert!(out.rendered_config.contains("<job_config>"));
    }
}

uniffi::include_scaffolding!("dcg_uniffi");
