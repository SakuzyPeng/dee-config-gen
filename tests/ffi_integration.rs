use std::{ptr, slice};

use dee_config_gen::ffi::{
    DcgError, DcgGenerateOptions, DcgGenerateOutput, DcgRenderFormat, DcgResolveOptions,
    DcgStatusCode, DcgStringView, DcgValidateOutput,
};

unsafe extern "C" {
    fn dcg_abi_version() -> u32;
    fn dcg_validate_job(
        job: DcgStringView,
        options: *const DcgResolveOptions,
        out: *mut DcgValidateOutput,
        err: *mut DcgError,
    ) -> DcgStatusCode;
    fn dcg_generate_config(
        job: DcgStringView,
        options: *const DcgGenerateOptions,
        out: *mut DcgGenerateOutput,
        err: *mut DcgError,
    ) -> DcgStatusCode;
    fn dcg_free_validate_output(out: *mut DcgValidateOutput);
    fn dcg_free_generate_output(out: *mut DcgGenerateOutput);
    fn dcg_free_error(err: *mut DcgError);
}

fn atmos_job_yaml() -> &'static str {
    r#"
template_id: atmos_ec3_v1
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
}

fn string_view(value: &str) -> DcgStringView {
    DcgStringView {
        ptr: value.as_ptr(),
        len: value.len(),
    }
}

unsafe fn owned_to_string(value: &dee_config_gen::ffi::DcgOwnedString) -> String {
    if value.ptr.is_null() || value.len == 0 {
        return String::new();
    }
    let bytes = unsafe { slice::from_raw_parts(value.ptr, value.len) };
    String::from_utf8(bytes.to_vec()).expect("ffi string should be utf8")
}

#[test]
fn ffi_abi_version_is_stable() {
    // SAFETY: symbol is linked from the library.
    let version = unsafe { dcg_abi_version() };
    assert_eq!(version, 1);
}

#[test]
fn ffi_validate_returns_compact_metadata() {
    let mut out = DcgValidateOutput::default();
    let mut err = DcgError::default();

    // SAFETY: all pointers target valid writable memory for the duration of the call.
    let code = unsafe {
        dcg_validate_job(
            string_view(atmos_job_yaml()),
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        )
    };

    assert_eq!(code, DcgStatusCode::Ok);
    // SAFETY: output is owned by Rust and valid until we call free below.
    unsafe {
        assert_eq!(owned_to_string(&out.template_id), "atmos_ec3_v1");
        assert_eq!(owned_to_string(&out.profile), "standard");
        assert_eq!(owned_to_string(&out.job_mode), "single");
        assert_eq!(owned_to_string(&out.encode_mode), "streaming");
    }

    // SAFETY: freeing objects returned by FFI.
    unsafe {
        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }
}

#[test]
fn ffi_generate_json_works_for_supported_template() {
    let mut out = DcgGenerateOutput::default();
    let mut err = DcgError::default();
    let options = DcgGenerateOptions {
        resolve: DcgResolveOptions::default(),
        format: DcgRenderFormat::JSON,
    };

    // SAFETY: all pointers target valid writable memory for the duration of the call.
    let code = unsafe {
        dcg_generate_config(
            string_view(atmos_job_yaml()),
            &options as *const _,
            &mut out as *mut _,
            &mut err as *mut _,
        )
    };

    assert_eq!(code, DcgStatusCode::Ok);
    // SAFETY: output is owned by Rust and valid until we call free below.
    unsafe {
        let rendered = owned_to_string(&out.rendered_config);
        assert!(rendered.contains("\"job_config\""));
    }

    // SAFETY: freeing objects returned by FFI.
    unsafe {
        dcg_free_generate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }
}

#[test]
fn ffi_invalid_utf8_maps_to_invalid_argument() {
    let invalid = [0xff_u8, 0xfe_u8];
    let mut out = DcgValidateOutput::default();
    let mut err = DcgError::default();

    // SAFETY: all pointers target valid writable memory for the duration of the call.
    let code = unsafe {
        dcg_validate_job(
            DcgStringView {
                ptr: invalid.as_ptr(),
                len: invalid.len(),
            },
            ptr::null(),
            &mut out as *mut _,
            &mut err as *mut _,
        )
    };

    assert_eq!(code, DcgStatusCode::InvalidArgument);
    // SAFETY: output is owned by Rust and valid until we call free below.
    unsafe {
        let msg = owned_to_string(&err.message);
        assert!(msg.contains("UTF-8"));
    }

    // SAFETY: freeing objects returned by FFI.
    unsafe {
        dcg_free_validate_output(&mut out as *mut _);
        dcg_free_error(&mut err as *mut _);
    }
}
