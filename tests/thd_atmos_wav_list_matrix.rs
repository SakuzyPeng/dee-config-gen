mod common;

use std::path::Path;

use dee_config_gen::{
    ResolveOptions, read_job, resolve_job,
    spec::{InputsSpec, IoSpec, Profile},
};

use common::{create_mono_wav_stems, create_test_wav_inputs};

fn resolve_with_generated_inputs(
    atmos_channels: usize,
    atmos_bits: u16,
    stem_count: usize,
    mutate: impl FnOnce(&mut dee_config_gen::JobSpec),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let (_atmos_temp, atmos_storage, atmos_files) =
        create_test_wav_inputs(atmos_channels, atmos_bits, 1);
    let (_stems_temp, stem_storage, stem_files) = create_mono_wav_stems(stem_count);
    let mut spec =
        read_job(Path::new("examples/thd_atmos_wav_list_single.mlp.yaml")).expect("load example");
    spec.inputs = Some(InputsSpec {
        atmos_mezz: Some(IoSpec {
            storage_path: atmos_storage,
            file_names: atmos_files,
        }),
        wav: None,
        wav_list: Some(IoSpec {
            storage_path: stem_storage,
            file_names: stem_files,
        }),
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
fn resolves_default_thd_atmos_wav_list_values() {
    let resolved =
        resolve_with_generated_inputs(2, 16, 6, |_| {}).expect("resolve mixed thd defaults");
    let dee_config_gen::ResolvedFilter::ThdAtmosWavListV1(filter) = resolved.filter else {
        panic!("expected ThdAtmosWavListV1 filter");
    };

    assert_eq!(filter.channel_configuration, "auto");
    assert_eq!(filter.input_timecode_frame_rate, "not_indicated");
    assert_eq!(filter.offset, "auto");
    assert_eq!(filter.ffoa, "auto");
}

#[test]
fn auto_channel_configuration_resolves_from_wav_list_input_count() {
    for (count, expected) in [(2, "stereo"), (6, "5.1"), (8, "7.1")] {
        let resolved = resolve_with_generated_inputs(2, 16, count, |_| {})
            .unwrap_or_else(|err| panic!("auto {expected} should resolve: {err}"));
        let xml = dee_config_gen::render_xml(&resolved);
        assert!(xml.contains(&format!(
            "<channel_configuration>{expected}</channel_configuration>"
        )));
    }
}

#[test]
fn rejects_missing_groups_and_old_input_shape() {
    let mut spec =
        read_job(Path::new("examples/thd_atmos_wav_list_single.mlp.yaml")).expect("load example");
    let (_temp, storage_path, file_names) = create_test_wav_inputs(2, 16, 1);
    spec.inputs = None;
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;

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
fn rejects_invalid_group_lengths_and_channel_configuration_mismatch() {
    let err = resolve_with_generated_inputs(2, 16, 6, |spec| {
        spec.filter.channel_configuration = Some("7.1".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires 8 input.file_names entries"));

    let err = resolve_with_generated_inputs(2, 16, 2, |spec| {
        spec.filter.channel_configuration = Some("5.1".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires 6 input.file_names entries"));
}

#[test]
fn rejects_inconsistent_atmos_and_wav_list_media() {
    let err = resolve_with_generated_inputs(2, 24, 6, |_| {})
        .unwrap_err()
        .to_string();
    assert!(err.contains("requires matching bits_per_sample"));
}

#[test]
fn allows_explicit_wav_list_offsets_with_default_start_boundary() {
    let resolved = resolve_with_generated_inputs(2, 16, 6, |spec| {
        spec.filter.offset = Some("00:00:01.000".to_string());
        spec.filter.ffoa = Some("00:00:02.000".to_string());
    })
    .expect("mixed thd wav_list should keep explicit offset/ffoa with default start");
    let xml = dee_config_gen::render_xml(&resolved);
    assert!(xml.contains("<offset>00:00:01.000</offset>"));
    assert!(xml.contains("<ffoa>00:00:02.000</ffoa>"));
    assert!(xml.contains("<start>first_frame_of_action</start>"));
}

#[test]
fn resolves_music_profile_without_extra_locks() {
    let resolved = resolve_with_generated_inputs(2, 16, 6, |spec| {
        spec.profile = Profile::Music;
    })
    .expect("mixed thd wav_list music profile should resolve unchanged");
    let dee_config_gen::ResolvedFilter::ThdAtmosWavListV1(filter) = resolved.filter else {
        panic!("expected ThdAtmosWavListV1 filter");
    };

    assert_eq!(filter.atmos_presentation_drc_profile, "film_light");
    assert_eq!(filter.presentation_8ch_drc_profile, "film_light");
    assert_eq!(filter.presentation_6ch_drc_profile, "film_light");
    assert_eq!(filter.presentation_2ch_drc_profile, "film_light");
}
