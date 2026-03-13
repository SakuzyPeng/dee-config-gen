mod common;

use std::path::Path;

use dee_config_gen::{
    ResolveOptions, read_job, render_xml, resolve_job,
    spec::{FilterOverrides, InputsSpec},
};

use common::{create_mono_wav_stems, create_test_wav_inputs};

fn resolve_from_example(
    mutate: impl FnOnce(&mut dee_config_gen::JobSpec),
) -> dee_config_gen::ResolvedJob {
    let mut spec = read_job(Path::new("examples/thd_atmos_wav_list_single.mlp.yaml")).unwrap();
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
fn renders_thd_atmos_wav_list_example() {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_stems_temp, stem_storage, stem_files) = create_mono_wav_stems(6);
    let resolved = resolve_from_example(|spec| {
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
    });
    let xml = render_xml(&resolved);

    assert_eq!(resolved.template_id, "thd_atmos_wav_list_v1");
    assert!(xml.contains("<atmos_mezz version=\"1\">"));
    assert!(xml.contains("<wav_list version=\"1\">"));
    assert!(xml.contains("<file_name>input_00.wav</file_name>"));
    assert!(xml.contains("<file_name_L>stem_00.wav</file_name_L>"));
    assert!(xml.contains("<channel_configuration>5.1</channel_configuration>"));
}

#[test]
fn renders_thd_atmos_wav_list_explicit_field_overrides() {
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    let (_stems_temp, stem_storage, stem_files) = create_mono_wav_stems(6);
    let resolved = resolve_from_example(|spec| {
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
        spec.filter = FilterOverrides {
            channel_configuration: Some("5.1".to_string()),
            input_timecode_frame_rate: Some("24".to_string()),
            offset: Some("00:00:01.000".to_string()),
            ffoa: Some("00:00:02.000".to_string()),
            spatial_clusters: Some("14".to_string()),
            legacy_authoring_compatibility: Some(false),
            optimize_data_rate: Some(true),
            ..FilterOverrides::default()
        };
    });
    let xml = render_xml(&resolved);

    assert!(xml.contains("<channel_configuration>5.1</channel_configuration>"));
    assert!(xml.contains("<timecode_frame_rate>24</timecode_frame_rate>"));
    assert!(xml.contains("<offset>00:00:01.000</offset>"));
    assert!(xml.contains("<ffoa>00:00:02.000</ffoa>"));
    assert!(xml.contains("<spatial_clusters>14</spatial_clusters>"));
    assert!(xml.contains("<legacy_authoring_compatibility>false</legacy_authoring_compatibility>"));
    assert!(xml.contains("<optimize_data_rate>true</optimize_data_rate>"));
}
