use std::fs;

fn uniffi_udl() -> String {
    fs::read_to_string("ffi/uniffi_bridge/src/dcg_uniffi.udl")
        .expect("read ffi/uniffi_bridge/src/dcg_uniffi.udl")
}

fn normalized_text(text: &str) -> String {
    text.replace("\r\n", "\n")
}

fn extract_block<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
    let after_start = text
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1;
    after_start
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker: {end}"))
        .0
}

fn quoted_variants(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('"'))
        .map(|line| {
            line.trim_start_matches('"')
                .split('"')
                .next()
                .expect("quoted variant value")
                .to_string()
        })
        .collect()
}

fn error_variants(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with("(string message);"))
        .map(|line| {
            line.split('(')
                .next()
                .expect("error variant prefix")
                .to_string()
        })
        .collect()
}

fn normalized_lines(block: &str) -> Vec<String> {
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[test]
fn uniffi_exports_are_frozen() {
    let udl = normalized_text(&uniffi_udl());
    let namespace = extract_block(&udl, "namespace dcg_uniffi {", "};");

    assert!(
        namespace.contains("u32 contract_version();"),
        "contract_version export must remain stable"
    );
    assert!(
        namespace
            .contains("ValidateOutput validate_job(string job_text, ResolveOptions? options);"),
        "validate_job export must remain stable"
    );
    assert!(
        namespace
            .contains("GenerateOutput generate_config(string job_text, GenerateOptions? options);"),
        "generate_config export must remain stable"
    );
    assert!(
        !namespace.contains("run_job("),
        "run_job must not be exposed in UniFFI v1"
    );
}

#[test]
fn uniffi_error_variants_are_frozen() {
    let udl = normalized_text(&uniffi_udl());
    let error_interface = extract_block(&udl, "[Error]\ninterface BridgeError {", "};");
    let actual = error_variants(error_interface);
    let expected = vec![
        "INVALID_ARGUMENT".to_string(),
        "PARSE_ERROR".to_string(),
        "RESOLVE_ERROR".to_string(),
        "RENDER_ERROR".to_string(),
        "INTERNAL_ERROR".to_string(),
        "PANIC".to_string(),
    ];
    assert_eq!(actual, expected);
}

#[test]
fn uniffi_render_format_variants_are_frozen() {
    let udl = normalized_text(&uniffi_udl());
    let render_enum = extract_block(&udl, "enum RenderFormat {", "};");
    let actual = quoted_variants(render_enum);
    assert_eq!(actual, vec!["XML".to_string(), "JSON".to_string()]);
}

#[test]
fn uniffi_record_shapes_are_frozen() {
    let udl = normalized_text(&uniffi_udl());

    let resolve_options = extract_block(&udl, "dictionary ResolveOptions {", "};");
    assert_eq!(
        normalized_lines(resolve_options),
        vec![
            "string? template_override;".to_string(),
            "boolean allow_fixed_override;".to_string(),
            "string? windows_drive;".to_string(),
        ]
    );

    let generate_options = extract_block(&udl, "dictionary GenerateOptions {", "};");
    assert_eq!(
        normalized_lines(generate_options),
        vec![
            "ResolveOptions? resolve;".to_string(),
            "RenderFormat? format;".to_string(),
        ]
    );

    let validate_output = extract_block(&udl, "dictionary ValidateOutput {", "};");
    assert_eq!(
        normalized_lines(validate_output),
        vec![
            "string template_id;".to_string(),
            "string profile;".to_string(),
            "string job_mode;".to_string(),
            "string encode_mode;".to_string(),
            "string output_container;".to_string(),
        ]
    );

    let generate_output = extract_block(&udl, "dictionary GenerateOutput {", "};");
    assert_eq!(
        normalized_lines(generate_output),
        vec![
            "string rendered_config;".to_string(),
            "RenderFormat format;".to_string(),
            "string template_id;".to_string(),
            "string output_container;".to_string(),
        ]
    );
}
