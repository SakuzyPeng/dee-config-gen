mod common;

use std::path::Path;

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, InputsSpec, IoSpec},
};

use common::create_test_wav_inputs;

fn resolve_from_example(
    mutate: impl FnOnce(&mut dee_config_gen::JobSpec),
) -> dee_config_gen::ResolvedJob {
    let mut spec = read_job(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).unwrap();
    mutate(&mut spec);
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
fn renders_thd_atmos_wav_example() {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_wav_temp, wav_storage, wav_files) = create_test_wav_inputs(6, 16, 1);
    let resolved = resolve_from_example(|spec| {
        spec.inputs = Some(InputsSpec {
            atmos_mezz: Some(IoSpec {
                storage_path: atmos_storage,
                file_names: atmos_files,
            }),
            wav: Some(IoSpec {
                storage_path: wav_storage,
                file_names: wav_files,
            }),
            wav_list: None,
        });
    });
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "thd_atmos_wav_v1");
    assert!(xml.contains("<atmos_mezz version=\"1\">"));
    assert!(xml.contains("<wav version=\"1\">"));
    assert!(xml.contains("<file_name>input_00.wav</file_name>"));
}

#[test]
fn renders_thd_atmos_wav_explicit_field_overrides() {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_wav_temp, wav_storage, wav_files) = create_test_wav_inputs(6, 16, 1);
    let resolved = resolve_from_example(|spec| {
        spec.inputs = Some(InputsSpec {
            atmos_mezz: Some(IoSpec {
                storage_path: atmos_storage,
                file_names: atmos_files,
            }),
            wav: Some(IoSpec {
                storage_path: wav_storage,
                file_names: wav_files,
            }),
            wav_list: None,
        });
        spec.filter = FilterOverrides {
            input_timecode_frame_rate: Some("24".to_string()),
            offset: Some("00:00:01.000".to_string()),
            ffoa: Some("00:00:02.000".to_string()),
            start: Some("00:00:00.000".to_string()),
            spatial_clusters: Some("14".to_string()),
            legacy_authoring_compatibility: Some(false),
            optimize_data_rate: Some(true),
            ..FilterOverrides::default()
        };
    });
    let xml = render_xml(&resolved);

    assert!(xml.contains("<timecode_frame_rate>24</timecode_frame_rate>"));
    assert!(xml.contains("<offset>00:00:01.000</offset>"));
    assert!(xml.contains("<ffoa>00:00:02.000</ffoa>"));
    assert!(xml.contains("<start>00:00:00.000</start>"));
    assert!(xml.contains("<spatial_clusters>14</spatial_clusters>"));
    assert!(xml.contains("<legacy_authoring_compatibility>false</legacy_authoring_compatibility>"));
    assert!(xml.contains("<optimize_data_rate>true</optimize_data_rate>"));
}
