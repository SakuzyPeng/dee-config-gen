use std::mem::{align_of, offset_of, size_of};

use dee_config_gen::ffi::{
    DcgError, DcgGenerateOptions, DcgGenerateOutput, DcgOwnedString, DcgRenderFormat,
    DcgResolveOptions, DcgStatusCode, DcgStringView, DcgValidateOutput,
};

unsafe extern "C" {
    fn dcg_abi_version() -> u32;
}

#[test]
fn ffi_abi_version_is_v1() {
    // SAFETY: symbol is linked from the crate.
    let version = unsafe { dcg_abi_version() };
    assert_eq!(version, 1);
}

#[test]
fn ffi_status_code_values_are_frozen() {
    assert_eq!(DcgStatusCode::Ok as u32, 0);
    assert_eq!(DcgStatusCode::InvalidArgument as u32, 1);
    assert_eq!(DcgStatusCode::ParseError as u32, 2);
    assert_eq!(DcgStatusCode::ResolveError as u32, 3);
    assert_eq!(DcgStatusCode::RenderError as u32, 4);
    assert_eq!(DcgStatusCode::InternalError as u32, 5);
    assert_eq!(DcgStatusCode::Panic as u32, 6);
}

#[test]
fn ffi_render_format_values_are_frozen() {
    assert_eq!(DcgRenderFormat::XML.0, 0);
    assert_eq!(DcgRenderFormat::JSON.0, 1);
}

#[test]
fn ffi_c_repr_size_align_are_frozen() {
    assert_eq!(size_of::<DcgStringView>(), 16);
    assert_eq!(align_of::<DcgStringView>(), 8);

    assert_eq!(size_of::<DcgOwnedString>(), 16);
    assert_eq!(align_of::<DcgOwnedString>(), 8);

    assert_eq!(size_of::<DcgResolveOptions>(), 32);
    assert_eq!(align_of::<DcgResolveOptions>(), 8);

    assert_eq!(size_of::<DcgGenerateOptions>(), 40);
    assert_eq!(align_of::<DcgGenerateOptions>(), 8);

    assert_eq!(size_of::<DcgError>(), 24);
    assert_eq!(align_of::<DcgError>(), 8);

    assert_eq!(size_of::<DcgValidateOutput>(), 80);
    assert_eq!(align_of::<DcgValidateOutput>(), 8);

    assert_eq!(size_of::<DcgGenerateOutput>(), 56);
    assert_eq!(align_of::<DcgGenerateOutput>(), 8);
}

#[test]
fn ffi_field_offsets_are_frozen() {
    assert_eq!(offset_of!(DcgStringView, ptr), 0);
    assert_eq!(offset_of!(DcgStringView, len), 8);

    assert_eq!(offset_of!(DcgOwnedString, ptr), 0);
    assert_eq!(offset_of!(DcgOwnedString, len), 8);

    assert_eq!(offset_of!(DcgResolveOptions, has_template_override), 0);
    assert_eq!(offset_of!(DcgResolveOptions, template_override), 8);
    assert_eq!(offset_of!(DcgResolveOptions, allow_fixed_override), 24);
    assert_eq!(offset_of!(DcgResolveOptions, windows_drive), 25);

    assert_eq!(offset_of!(DcgGenerateOptions, resolve), 0);
    assert_eq!(offset_of!(DcgGenerateOptions, format), 32);

    assert_eq!(offset_of!(DcgError, code), 0);
    assert_eq!(offset_of!(DcgError, message), 8);

    assert_eq!(offset_of!(DcgValidateOutput, template_id), 0);
    assert_eq!(offset_of!(DcgValidateOutput, profile), 16);
    assert_eq!(offset_of!(DcgValidateOutput, job_mode), 32);
    assert_eq!(offset_of!(DcgValidateOutput, encode_mode), 48);
    assert_eq!(offset_of!(DcgValidateOutput, output_container), 64);

    assert_eq!(offset_of!(DcgGenerateOutput, rendered_config), 0);
    assert_eq!(offset_of!(DcgGenerateOutput, format), 16);
    assert_eq!(offset_of!(DcgGenerateOutput, template_id), 24);
    assert_eq!(offset_of!(DcgGenerateOutput, output_container), 40);
}
