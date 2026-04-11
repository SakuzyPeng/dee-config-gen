use std::{
    any::Any,
    fmt::Display,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr, slice,
};

use crate::{RenderFormat, ResolveOptions, parse_job_str, render_config, validate_job};

pub const DCG_ABI_VERSION: u32 = 1;
pub const DCG_FFI_HEADER_VERSION: u32 = 10100;
pub const DCG_FFI_HEADER_VERSION_STR: &str = "1.1.0";

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DcgStatusCode {
    #[default]
    Ok = 0,
    InvalidArgument = 1,
    ParseError = 2,
    ResolveError = 3,
    RenderError = 4,
    InternalError = 5,
    Panic = 6,
}

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DcgRenderFormat(pub u32);

impl DcgRenderFormat {
    pub const XML: Self = Self(0);
    pub const JSON: Self = Self(1);
}

impl Default for DcgRenderFormat {
    fn default() -> Self {
        Self::XML
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DcgStringView {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgOwnedString {
    pub ptr: *mut u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgResolveOptions {
    pub has_template_override: bool,
    pub template_override: DcgStringView,
    pub allow_fixed_override: bool,
    pub windows_drive: u8,
}

impl DcgResolveOptions {
    fn with_defaults_applied(&self) -> Result<ResolveOptions, BridgeError> {
        let template_override = if self.has_template_override {
            Some(string_from_view(
                self.template_override,
                "template_override",
                false,
            )?)
        } else {
            None
        };

        Ok(ResolveOptions {
            template_override,
            allow_fixed_override: self.allow_fixed_override,
            windows_drive: parse_windows_drive(self.windows_drive)?,
        })
    }
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgGenerateOptions {
    pub resolve: DcgResolveOptions,
    pub format: DcgRenderFormat,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgError {
    pub code: DcgStatusCode,
    pub message: DcgOwnedString,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgValidateOutput {
    pub template_id: DcgOwnedString,
    pub profile: DcgOwnedString,
    pub job_mode: DcgOwnedString,
    pub encode_mode: DcgOwnedString,
    pub output_container: DcgOwnedString,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct DcgGenerateOutput {
    pub rendered_config: DcgOwnedString,
    pub format: DcgRenderFormat,
    pub template_id: DcgOwnedString,
    pub output_container: DcgOwnedString,
}

#[derive(Debug)]
struct BridgeError {
    code: DcgStatusCode,
    message: String,
}

impl BridgeError {
    fn invalid_argument(message: impl Into<String>) -> Self {
        Self {
            code: DcgStatusCode::InvalidArgument,
            message: message.into(),
        }
    }

    fn stage(code: DcgStatusCode, err: impl Display) -> Self {
        Self {
            code,
            message: err.to_string(),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dcg_abi_version() -> u32 {
    DCG_ABI_VERSION
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_validate_job(
    job: DcgStringView,
    options: *const DcgResolveOptions,
    out: *mut DcgValidateOutput,
    err: *mut DcgError,
) -> DcgStatusCode {
    if err.is_null() {
        return DcgStatusCode::InvalidArgument;
    }
    if out.is_null() {
        // SAFETY: `err` is checked non-null above.
        unsafe {
            write_error(
                err,
                DcgStatusCode::InvalidArgument,
                "validate output pointer is null",
            );
        }
        return DcgStatusCode::InvalidArgument;
    }

    // SAFETY: pointers are checked above.
    unsafe {
        reset_validate_output(&mut *out);
        clear_error(&mut *err);
    }

    with_panic_boundary(err, || {
        let job_text = string_from_view(job, "job", false)?;
        let resolve_options = resolve_options_from_raw(options)?;

        let spec = parse_job_str(&job_text)
            .map_err(|e| BridgeError::stage(DcgStatusCode::ParseError, e))?;
        let resolved = validate_job(spec, &resolve_options)
            .map_err(|e| BridgeError::stage(DcgStatusCode::ResolveError, e))?;

        // SAFETY: `out` is checked non-null above and lives for the duration of this call.
        unsafe {
            write_validate_output(&mut *out, &resolved);
        }
        Ok(())
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_generate_config(
    job: DcgStringView,
    options: *const DcgGenerateOptions,
    out: *mut DcgGenerateOutput,
    err: *mut DcgError,
) -> DcgStatusCode {
    if err.is_null() {
        return DcgStatusCode::InvalidArgument;
    }
    if out.is_null() {
        // SAFETY: `err` is checked non-null above.
        unsafe {
            write_error(
                err,
                DcgStatusCode::InvalidArgument,
                "generate output pointer is null",
            );
        }
        return DcgStatusCode::InvalidArgument;
    }

    // SAFETY: pointers are checked above.
    unsafe {
        reset_generate_output(&mut *out);
        clear_error(&mut *err);
    }

    with_panic_boundary(err, || {
        let job_text = string_from_view(job, "job", false)?;
        let (resolve_options, format) = generate_options_from_raw(options)?;

        let spec = parse_job_str(&job_text)
            .map_err(|e| BridgeError::stage(DcgStatusCode::ParseError, e))?;
        let resolved = validate_job(spec, &resolve_options)
            .map_err(|e| BridgeError::stage(DcgStatusCode::ResolveError, e))?;
        let rendered = render_config(&resolved, format)
            .map_err(|e| BridgeError::stage(DcgStatusCode::RenderError, e))?;

        // SAFETY: `out` is checked non-null above and lives for the duration of this call.
        unsafe {
            write_generate_output(
                &mut *out,
                format,
                &resolved.template_id,
                &resolved,
                &rendered,
            );
        }
        Ok(())
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_free_string(value: *mut DcgOwnedString) {
    if value.is_null() {
        return;
    }
    // SAFETY: caller provided a non-null pointer and this function only touches its own fields.
    unsafe {
        release_owned_string(&mut *value);
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_free_validate_output(out: *mut DcgValidateOutput) {
    if out.is_null() {
        return;
    }
    // SAFETY: caller provided a non-null pointer and this function only touches its own fields.
    unsafe {
        reset_validate_output(&mut *out);
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_free_generate_output(out: *mut DcgGenerateOutput) {
    if out.is_null() {
        return;
    }
    // SAFETY: caller provided a non-null pointer and this function only touches its own fields.
    unsafe {
        reset_generate_output(&mut *out);
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn dcg_free_error(err: *mut DcgError) {
    if err.is_null() {
        return;
    }
    // SAFETY: caller provided a non-null pointer and this function only touches its own fields.
    unsafe {
        clear_error(&mut *err);
    }
}

fn with_panic_boundary(
    err: *mut DcgError,
    f: impl FnOnce() -> Result<(), BridgeError>,
) -> DcgStatusCode {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(())) => DcgStatusCode::Ok,
        Ok(Err(bridge)) => {
            // SAFETY: caller guarantees `err` points to writable memory.
            unsafe {
                write_error(err, bridge.code, &bridge.message);
            }
            bridge.code
        }
        Err(payload) => {
            let panic_message = panic_payload_to_string(payload);
            // SAFETY: caller guarantees `err` points to writable memory.
            unsafe {
                write_error(err, DcgStatusCode::Panic, &panic_message);
            }
            DcgStatusCode::Panic
        }
    }
}

fn generate_options_from_raw(
    raw: *const DcgGenerateOptions,
) -> Result<(ResolveOptions, RenderFormat), BridgeError> {
    if raw.is_null() {
        return Ok((ResolveOptions::default(), RenderFormat::Xml));
    }

    // SAFETY: pointer nullity is checked above.
    let raw = unsafe { &*raw };
    let resolve = raw.resolve.with_defaults_applied()?;
    let format = parse_render_format(raw.format)?;
    Ok((resolve, format))
}

fn resolve_options_from_raw(raw: *const DcgResolveOptions) -> Result<ResolveOptions, BridgeError> {
    if raw.is_null() {
        return Ok(ResolveOptions::default());
    }

    // SAFETY: pointer nullity is checked above.
    unsafe { (&*raw).with_defaults_applied() }
}

fn parse_render_format(raw: DcgRenderFormat) -> Result<RenderFormat, BridgeError> {
    match raw.0 {
        0 => Ok(RenderFormat::Xml),
        1 => Ok(RenderFormat::Json),
        other => Err(BridgeError::invalid_argument(format!(
            "invalid format value {other}; expected 0 (xml) or 1 (json)"
        ))),
    }
}

fn parse_windows_drive(raw: u8) -> Result<char, BridgeError> {
    let value = if raw == 0 { b'Y' } else { raw };
    let ch = value as char;
    if !ch.is_ascii_alphabetic() {
        return Err(BridgeError::invalid_argument(format!(
            "invalid windows_drive '{}'; expected ASCII drive letter",
            ch.escape_default()
        )));
    }
    Ok(ch.to_ascii_uppercase())
}

fn string_from_view(
    view: DcgStringView,
    field_name: &str,
    allow_empty: bool,
) -> Result<String, BridgeError> {
    if view.ptr.is_null() {
        return Err(BridgeError::invalid_argument(format!(
            "{field_name} pointer is null"
        )));
    }

    // SAFETY: caller passes pointer+len for a readable memory range.
    let bytes = unsafe { slice::from_raw_parts(view.ptr, view.len) };
    if !allow_empty && bytes.is_empty() {
        return Err(BridgeError::invalid_argument(format!(
            "{field_name} must not be empty"
        )));
    }

    std::str::from_utf8(bytes)
        .map(|s| s.to_string())
        .map_err(|_| BridgeError::invalid_argument(format!("{field_name} must be UTF-8")))
}

fn panic_payload_to_string(payload: Box<dyn Any + Send>) -> String {
    let any = payload.as_ref();
    if let Some(value) = any.downcast_ref::<&str>() {
        return format!("panic across FFI boundary: {value}");
    }
    if let Some(value) = any.downcast_ref::<String>() {
        return format!("panic across FFI boundary: {value}");
    }
    "panic across FFI boundary".to_string()
}

unsafe fn write_validate_output(out: &mut DcgValidateOutput, resolved: &crate::ResolvedJob) {
    out.template_id = owned_string(&resolved.template_id);
    out.profile = owned_string(resolved.profile.as_str());
    out.job_mode = owned_string(resolved.job_mode.as_str());
    out.encode_mode = owned_string(resolved.encode_mode.as_str());
    out.output_container = owned_string(resolved.output.container.as_str());
}

unsafe fn write_generate_output(
    out: &mut DcgGenerateOutput,
    format: RenderFormat,
    template_id: &str,
    resolved: &crate::ResolvedJob,
    rendered: &str,
) {
    out.rendered_config = owned_string(rendered);
    out.format = match format {
        RenderFormat::Xml => DcgRenderFormat::XML,
        RenderFormat::Json => DcgRenderFormat::JSON,
    };
    out.template_id = owned_string(template_id);
    out.output_container = owned_string(resolved.output.container.as_str());
}

unsafe fn clear_error(err: &mut DcgError) {
    unsafe {
        release_owned_string(&mut err.message);
    }
    err.code = DcgStatusCode::Ok;
}

unsafe fn write_error(err: *mut DcgError, code: DcgStatusCode, message: &str) {
    let err = unsafe { &mut *err };
    unsafe {
        clear_error(err);
    }
    err.code = code;
    err.message = owned_string(message);
}

unsafe fn reset_validate_output(out: &mut DcgValidateOutput) {
    unsafe {
        release_owned_string(&mut out.template_id);
        release_owned_string(&mut out.profile);
        release_owned_string(&mut out.job_mode);
        release_owned_string(&mut out.encode_mode);
        release_owned_string(&mut out.output_container);
    }
}

unsafe fn reset_generate_output(out: &mut DcgGenerateOutput) {
    unsafe {
        release_owned_string(&mut out.rendered_config);
        release_owned_string(&mut out.template_id);
        release_owned_string(&mut out.output_container);
    }
    out.format = DcgRenderFormat::XML;
}

fn owned_string(value: &str) -> DcgOwnedString {
    if value.is_empty() {
        return DcgOwnedString::default();
    }

    let bytes = value.as_bytes().to_vec().into_boxed_slice();
    let len = bytes.len();
    let ptr = Box::into_raw(bytes) as *mut u8;

    DcgOwnedString { ptr, len }
}

unsafe fn release_owned_string(value: &mut DcgOwnedString) {
    if !value.ptr.is_null() && value.len > 0 {
        let raw = ptr::slice_from_raw_parts_mut(value.ptr, value.len);
        let _ = unsafe { Box::from_raw(raw) };
    }
    value.ptr = ptr::null_mut();
    value.len = 0;
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

    fn view_from_str(value: &str) -> DcgStringView {
        DcgStringView {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }

    unsafe fn owned_to_string(value: &DcgOwnedString) -> String {
        if value.ptr.is_null() || value.len == 0 {
            return String::new();
        }
        let bytes = unsafe { slice::from_raw_parts(value.ptr, value.len) };
        String::from_utf8(bytes.to_vec()).expect("valid utf8 output")
    }

    #[test]
    fn validate_returns_metadata() {
        let job = sample_job("atmos_ec3_v1");
        let mut out = DcgValidateOutput::default();
        let mut err = DcgError::default();

        let code = dcg_validate_job(
            view_from_str(&job),
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::Ok);
        // SAFETY: FFI result pointers are valid until freed below.
        unsafe {
            assert_eq!(owned_to_string(&out.template_id), "atmos_ec3_v1");
            assert_eq!(owned_to_string(&out.profile), "standard");
            assert_eq!(owned_to_string(&out.job_mode), "single");
            assert_eq!(owned_to_string(&out.encode_mode), "streaming");
            assert_eq!(owned_to_string(&out.output_container), "ac4");
        }

        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn generate_returns_xml() {
        let job = sample_job("atmos_ec3_v1");
        let mut out = DcgGenerateOutput::default();
        let mut err = DcgError::default();
        let options = DcgGenerateOptions {
            resolve: DcgResolveOptions {
                windows_drive: b'Y',
                ..DcgResolveOptions::default()
            },
            format: DcgRenderFormat::XML,
        };

        let code = dcg_generate_config(
            view_from_str(&job),
            &options as *const _,
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::Ok);
        // SAFETY: FFI result pointers are valid until freed below.
        unsafe {
            let rendered = owned_to_string(&out.rendered_config);
            assert!(rendered.starts_with("<?xml version=\"1.0\"?>"));
            assert_eq!(out.format.0, DcgRenderFormat::XML.0);
        }

        dcg_free_generate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn generate_returns_json_for_supported_template() {
        let mut out = DcgGenerateOutput::default();
        let mut err = DcgError::default();
        let options = DcgGenerateOptions {
            format: DcgRenderFormat::JSON,
            ..DcgGenerateOptions::default()
        };

        let code = dcg_generate_config(
            view_from_str(&sample_job("atmos_ec3_v1")),
            &options as *const _,
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::Ok);
        // SAFETY: FFI result pointers are valid until freed below.
        unsafe {
            let rendered = owned_to_string(&out.rendered_config);
            assert!(rendered.contains("\"job_config\""));
            assert_eq!(out.format.0, DcgRenderFormat::JSON.0);
        }

        dcg_free_generate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn parse_errors_are_mapped() {
        let invalid = "template_id: [";
        let mut out = DcgValidateOutput::default();
        let mut err = DcgError::default();

        let code = dcg_validate_job(
            view_from_str(invalid),
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::ParseError);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(
                owned_to_string(&err.message).contains("failed to parse input as YAML or JSON")
            );
        }

        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn resolve_errors_are_mapped() {
        let job = sample_job("unsupported_template");
        let mut out = DcgValidateOutput::default();
        let mut err = DcgError::default();

        let code = dcg_validate_job(
            view_from_str(&job),
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::ResolveError);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(owned_to_string(&err.message).contains("unsupported template_id"));
        }

        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn render_errors_are_mapped() {
        let spec = parse_job_str(&sample_job("atmos_ec3_v1")).expect("parse");
        let mut resolved = validate_job(spec, &ResolveOptions::default()).expect("resolve");
        resolved.template_id = "__render_break__".to_string();

        let render_err = render_config(&resolved, RenderFormat::Json).expect_err("render fails");
        let mapped = BridgeError::stage(DcgStatusCode::RenderError, render_err);
        assert_eq!(mapped.code, DcgStatusCode::RenderError);
        assert!(mapped.message.contains("unsupported template_id"));
    }

    #[test]
    fn invalid_utf8_is_invalid_argument() {
        let bytes = [0xff_u8];
        let mut out = DcgValidateOutput::default();
        let mut err = DcgError::default();

        let code = dcg_validate_job(
            DcgStringView {
                ptr: bytes.as_ptr(),
                len: bytes.len(),
            },
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::InvalidArgument);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(owned_to_string(&err.message).contains("job must be UTF-8"));
        }

        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn invalid_enum_values_are_invalid_argument() {
        let mut out = DcgGenerateOutput::default();
        let mut err = DcgError::default();
        let options = DcgGenerateOptions {
            format: DcgRenderFormat(99),
            ..DcgGenerateOptions::default()
        };

        let code = dcg_generate_config(
            view_from_str(&sample_job("atmos_ec3_v1")),
            &options as *const _,
            &mut out as *mut _,
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::InvalidArgument);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(owned_to_string(&err.message).contains("invalid format value"));
        }

        dcg_free_generate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn panic_boundary_maps_to_panic_status() {
        let mut err = DcgError::default();
        let code = with_panic_boundary(&mut err as *mut _, || -> Result<(), BridgeError> {
            panic!("boom");
        });

        assert_eq!(code, DcgStatusCode::Panic);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(owned_to_string(&err.message).contains("panic across FFI boundary"));
        }
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn free_functions_are_idempotent_on_zero_values() {
        let mut value = DcgOwnedString::default();
        dcg_free_string(&mut value as *mut _);
        dcg_free_string(&mut value as *mut _);

        let mut output = DcgValidateOutput::default();
        dcg_free_validate_output(&mut output as *mut _);
        dcg_free_validate_output(&mut output as *mut _);

        let mut gen_output = DcgGenerateOutput::default();
        dcg_free_generate_output(&mut gen_output as *mut _);
        dcg_free_generate_output(&mut gen_output as *mut _);

        let mut err = DcgError::default();
        dcg_free_error(&mut err as *mut _);
        dcg_free_error(&mut err as *mut _);
    }

    #[test]
    fn null_output_pointer_is_invalid_argument() {
        let mut err = DcgError::default();
        let code = dcg_validate_job(
            view_from_str("template_id: atmos_ec3_v1"),
            ptr::null(),
            ptr::null_mut(),
            &mut err as *mut _,
        );

        assert_eq!(code, DcgStatusCode::InvalidArgument);
        // SAFETY: error payload pointers are valid until freed below.
        unsafe {
            assert!(owned_to_string(&err.message).contains("output pointer is null"));
        }
        dcg_free_error(&mut err as *mut _);
    }
}
