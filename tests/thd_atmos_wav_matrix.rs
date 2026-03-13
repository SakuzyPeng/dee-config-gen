mod common;

use std::path::Path;

use dee_config_gen::{
    ResolveOptions,
    config::{InputsSpec, IoSpec},
    load_job_file, resolve_job,
};

use common::create_test_wav_inputs;

fn resolve_with_generated_inputs(
    atmos_channels: usize,
    atmos_bits: u16,
    wav_channels: usize,
    wav_bits: u16,
    mutate: impl FnOnce(&mut dee_config_gen::JobFile),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let (_atmos_temp, atmos_storage, atmos_files) =
        create_test_wav_inputs(atmos_channels, atmos_bits, 1);
    let (_wav_temp, wav_storage, wav_files) = create_test_wav_inputs(wav_channels, wav_bits, 1);
    let mut spec =
        load_job_file(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).expect("load example");
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
    mutate(&mut spec);
    resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
}

#[test]
fn resolves_default_thd_atmos_wav_values() {
    let resolved =
        resolve_with_generated_inputs(2, 16, 6, 16, |_| {}).expect("resolve mixed thd defaults");
    let dee_config_gen::ResolvedFilter::ThdAtmosWavV1(filter) = resolved.filter else {
        panic!("expected ThdAtmosWavV1 filter");
    };

    assert_eq!(filter.input_timecode_frame_rate, "not_indicated");
    assert_eq!(filter.offset, "auto");
    assert_eq!(filter.ffoa, "auto");
    assert_eq!(filter.starting_timecode, "off");
    assert_eq!(filter.frame_rate, "auto");
}

#[test]
fn accepts_documented_thd_atmos_wav_overrides() {
    resolve_with_generated_inputs(2, 16, 6, 16, |spec| {
        spec.filter.input_timecode_frame_rate = Some("24".to_string());
        spec.filter.offset = Some("00:00:00.000".to_string());
        spec.filter.ffoa = Some("00:00:00.000".to_string());
        spec.filter.starting_timecode = Some("auto".to_string());
        spec.filter.frame_rate = Some("23.976".to_string());
        spec.filter.start = Some("00:00:00:00".to_string());
        spec.filter.end = Some("00:00:01.000".to_string());
        spec.filter.prepend_silence_duration = Some("1.25".to_string());
        spec.filter.append_silence_duration = Some("0.005333".to_string());
        spec.filter.custom_dialnorm = Some(-9);
        spec.filter.spatial_clusters = Some("16".to_string());
        spec.filter.legacy_authoring_compatibility = Some(false);
        spec.filter.optimize_data_rate = Some(true);
    })
    .expect("documented thd_atmos_wav overrides should resolve");
}

#[test]
fn rejects_missing_groups_and_old_input_shape() {
    let mut spec =
        load_job_file(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).expect("load example");
    spec.inputs = None;
    spec.input.storage_path = "testfiles".to_string();
    spec.input.file_names = vec!["testADM.wav".to_string()];

    let err = resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires the 'inputs' block"));
}

#[test]
fn rejects_invalid_groups_and_inconsistent_media() {
    let mut spec =
        load_job_file(Path::new("examples/thd_atmos_wav_single.mlp.yaml")).expect("load example");
    let (_atmos_temp, atmos_storage, atmos_files) = create_test_wav_inputs(2, 16, 1);
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: Some(IoSpec {
            storage_path: "wav".to_string(),
            file_names: vec!["input.wav".to_string()],
        }),
        wav_list: Some(IoSpec {
            storage_path: "stems".to_string(),
            file_names: vec!["stem_00.wav".to_string()],
        }),
    });
    let err = resolve_job(
        spec,
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("does not support inputs.wav_list"));

    let err = resolve_with_generated_inputs(2, 24, 6, 16, |_| {})
        .unwrap_err()
        .to_string();
    assert!(err.contains("requires matching bits_per_sample"));
}
