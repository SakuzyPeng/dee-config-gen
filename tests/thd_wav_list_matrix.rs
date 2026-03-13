mod common;

use std::path::Path;

use dee_config_gen::{ResolveOptions, read_job, resolve_job};

use common::create_mono_wav_stems;

fn resolve_with_generated_stems(
    channel_count: usize,
    mutate: impl FnOnce(&mut dee_config_gen::JobSpec),
) -> anyhow::Result<dee_config_gen::ResolvedJob> {
    let (_temp, storage_path, file_names) = create_mono_wav_stems(channel_count);
    let mut spec =
        read_job(Path::new("examples/thd_wav_list_single.mlp.yaml")).expect("load example");
    spec.input.storage_path = storage_path;
    spec.input.file_names = file_names;
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
fn resolves_default_thd_wav_list_filter_values() {
    let resolved = resolve_with_generated_stems(6, |_| {}).expect("resolve thd wav_list defaults");
    let dee_config_gen::ResolvedFilter::ThdWavListV1(filter) = resolved.filter else {
        panic!("expected ThdWavListV1 filter");
    };

    assert_eq!(filter.channel_configuration, "auto");
    assert_eq!(filter.input_timecode_frame_rate, "not_indicated");
    assert_eq!(filter.offset, "auto");
    assert_eq!(filter.ffoa, "auto");
    assert_eq!(filter.starting_timecode, "off");
    assert_eq!(filter.frame_rate, "auto");
}

#[test]
fn accepts_documented_thd_wav_list_overrides() {
    resolve_with_generated_stems(6, |spec| {
        spec.filter.channel_configuration = Some("5.1".to_string());
        spec.filter.input_timecode_frame_rate = Some("24".to_string());
        spec.filter.offset = Some("auto".to_string());
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
    .expect("documented thd_wav_list overrides should resolve");
}

#[test]
fn auto_channel_configuration_resolves_from_input_count() {
    for (count, expected) in [(2, "stereo"), (6, "5.1"), (8, "7.1")] {
        let resolved = resolve_with_generated_stems(count, |_| {})
            .unwrap_or_else(|err| panic!("auto {expected} should resolve: {err}"));
        let xml = dee_config_gen::render_xml(&resolved);
        assert!(xml.contains(&format!(
            "<channel_configuration>{expected}</channel_configuration>"
        )));
    }
}

#[test]
fn rejects_invalid_wav_list_lengths_and_channel_configuration_mismatch() {
    for invalid_len in [1, 3, 4, 5, 7] {
        let mut spec =
            read_job(Path::new("examples/thd_wav_list_single.mlp.yaml")).expect("load example");
        spec.input.file_names = (0..invalid_len)
            .map(|idx| format!("stem_{idx:02}.wav"))
            .collect();
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
        assert!(err.contains("requires 2, 6 or 8 input.file_names entries"));
    }

    let err = resolve_with_generated_stems(6, |spec| {
        spec.filter.channel_configuration = Some("7.1".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires 8 input.file_names entries"));

    let err = resolve_with_generated_stems(2, |spec| {
        spec.filter.channel_configuration = Some("5.1".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("requires 6 input.file_names entries"));
}

#[test]
fn rejects_dash_placeholders() {
    let (_temp, storage_path, stems) = create_mono_wav_stems(6);
    let err = resolve_job(
        {
            let mut spec =
                read_job(Path::new("examples/thd_wav_list_single.mlp.yaml")).expect("load example");
            spec.input.storage_path = storage_path;
            spec.input.file_names = vec![
                stems[0].clone(),
                stems[1].clone(),
                "-".to_string(),
                stems[3].clone(),
                stems[4].clone(),
                stems[5].clone(),
            ];
            spec.filter.channel_configuration = Some("5.1".to_string());
            spec
        },
        &ResolveOptions {
            template_override: None,
            allow_fixed_override: false,
            windows_drive: 'Y',
        },
    )
    .unwrap_err()
    .to_string();
    assert!(err.contains("does not support '-' placeholders"));
}

#[test]
fn rejects_invalid_thd_wav_list_formats_and_values() {
    let err = resolve_with_generated_stems(6, |spec| {
        spec.filter.offset = Some("bogus".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("expected auto, HH:MM:SS:FF[df], or HH:MM:SS.xx"));

    let err = resolve_with_generated_stems(6, |spec| {
        spec.filter.prepend_silence_duration = Some("1f".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("seconds.milliseconds"));

    let err = resolve_with_generated_stems(6, |spec| {
        spec.filter.frame_rate = Some("bogus".to_string());
    })
    .unwrap_err()
    .to_string();
    assert!(err.contains("invalid value 'bogus' for frame_rate"));
}
