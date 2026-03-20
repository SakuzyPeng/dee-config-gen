mod common;

use std::path::{Path, PathBuf};

use common::{create_mono_wav_stems, resolve_with_defaults};
use dee_config_gen::{
    RenderFormat, ResolvedFilter, read_job, render_config, render_xml, resolve_job,
    spec::{Ac4OutputMode, IoSpec, JobMode, OutputContainer, Profile},
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn load_pcm_example() -> dee_config_gen::JobSpec {
    let mut spec = read_job(Path::new("examples/ac4_ims_pcm_single.ac4.yaml"))
        .expect("load ac4 ims pcm example");
    spec.inputs
        .as_mut()
        .expect("inputs")
        .wav
        .as_mut()
        .expect("wav")
        .storage_path = repo_root().join("testfiles").display().to_string();
    spec
}

#[test]
fn resolves_ac4_ims_atmos_example() {
    let spec = read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos");
    assert_eq!(resolved.template_id, "ac4_ims_atmos_v1");
    assert_eq!(resolved.encode_mode.as_str(), "ac4");
}

#[test]
fn resolves_ac4_ims_atmos_mp4_example() {
    let spec = read_job(Path::new("examples/ac4_ims_atmos_single.mp4.yaml"))
        .expect("load ac4 ims atmos mp4 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos mp4");
    assert_eq!(resolved.template_id, "ac4_ims_atmos_v1");
    assert_eq!(resolved.output.container, OutputContainer::Mp4);
    assert_eq!(resolved.output.file_names, vec!["output.mp4".to_string()]);
}

#[test]
fn resolves_ac4_ims_atmos_multi3_example() {
    let spec = read_job(Path::new("examples/ac4_ims_atmos_multi3.ac4.yaml"))
        .expect("load ac4 ims atmos multi3 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos multi3");
    assert_eq!(resolved.template_id, "ac4_ims_atmos_v1");
    assert_eq!(resolved.output.ac4_output_mode, Ac4OutputMode::Multi3);
    assert_eq!(
        resolved.output.file_names,
        vec![
            "output_a.ac4".to_string(),
            "output_b.ac4".to_string(),
            "output_c.ac4".to_string()
        ]
    );
}

#[test]
fn resolves_ac4_ims_pcm_example() {
    let spec = load_pcm_example();
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims pcm");
    assert_eq!(resolved.template_id, "ac4_ims_pcm_v1");
    assert_eq!(resolved.encode_mode.as_str(), "ac4");
}

#[test]
fn resolves_ac4_ims_pcm_mp4_example() {
    let mut spec = read_job(Path::new("examples/ac4_ims_pcm_single.mp4.yaml"))
        .expect("load ac4 ims pcm mp4 example");
    spec.inputs
        .as_mut()
        .expect("inputs")
        .wav
        .as_mut()
        .expect("wav")
        .storage_path = repo_root().join("testfiles").display().to_string();
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims pcm mp4");
    assert_eq!(resolved.template_id, "ac4_ims_pcm_v1");
    assert_eq!(resolved.output.container, OutputContainer::Mp4);
    assert_eq!(resolved.output.file_names, vec!["output.mp4".to_string()]);
}

#[test]
fn renders_ac4_ims_atmos_xml_structure() {
    let spec = read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos");
    let xml = render_xml(&resolved);

    assert!(xml.contains("<atmos_mezz version=\"1\">"));
    assert!(xml.contains("<encode_to_ims_ac4 version=\"1\">"));
    assert!(xml.contains("<output>"));
    assert!(xml.contains("<ac4 version=\"1\">"));
}

#[test]
fn renders_ac4_ims_atmos_mp4_xml_structure() {
    let mut spec = read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    spec.output.container = OutputContainer::Mp4;
    spec.output.file_names = vec!["output.mp4".to_string()];

    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos");
    let xml = render_xml(&resolved);

    assert!(xml.contains("<mp4 version=\"1\">"));
    assert!(xml.contains("<output_format>mp4</output_format>"));
    assert!(xml.contains("<override_frame_rate>no</override_frame_rate>"));
    assert!(xml.contains("<fill_video>false</fill_video>"));
}

#[test]
fn renders_ac4_ims_atmos_multi3_xml_structure() {
    let spec = read_job(Path::new("examples/ac4_ims_atmos_multi3.ac4.yaml"))
        .expect("load ac4 ims atmos multi3 example");
    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims atmos multi3");
    let xml = render_xml(&resolved);

    assert!(xml.contains("<ac4 version=\"1\">"));
    assert!(xml.contains("<file_name>output_a.ac4 output_b.ac4 output_c.ac4</file_name>"));
    assert!(xml.contains("<local_multi_path>"));
}

#[test]
fn renders_ac4_ims_pcm_wav_list_xml_structure() {
    let (_temp, storage_path, stems) = create_mono_wav_stems(6);
    let mut spec = read_job(Path::new("examples/ac4_ims_pcm_single.ac4.yaml"))
        .expect("load ac4 ims pcm example");
    let inputs = spec.inputs.as_mut().expect("inputs");
    inputs.wav = None;
    inputs.wav_list = Some(IoSpec {
        storage_path,
        file_names: stems,
    });

    let resolved = resolve_job(spec, &Default::default()).expect("resolve ac4 ims pcm wav_list");
    let xml = render_xml(&resolved);

    assert!(xml.contains("<wav_list version=\"1\">"));
    assert!(xml.contains("<file_name_L>"));
    assert!(xml.contains("<channel_configuration>5.1</channel_configuration>"));
}

#[test]
fn rejects_invalid_input_family_for_atmos_template() {
    let mut spec = read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    spec.inputs.as_mut().expect("inputs").wav = Some(IoSpec {
        storage_path: "/tmp".to_string(),
        file_names: vec!["input.wav".to_string()],
    });

    let err = resolve_job(spec, &Default::default())
        .expect_err("wav should fail for atmos template")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_atmos_v1' does not support inputs.wav"
    );
}

#[test]
fn rejects_mp4_container_with_non_mp4_extension() {
    let mut spec = load_pcm_example();
    spec.output.container = OutputContainer::Mp4;
    spec.output.file_names = vec!["output.ac4".to_string()];

    let err = resolve_job(spec, &Default::default())
        .expect_err("mismatched mp4 extension should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_pcm_v1' requires output.file_names[0] to end with '.mp4' when output.container='mp4'"
    );
}

#[test]
fn rejects_ac4_multi3_with_non_ac4_container() {
    let mut spec = read_job(Path::new("examples/ac4_ims_atmos_single.mp4.yaml"))
        .expect("load ac4 ims atmos mp4 example");
    spec.output.ac4_output_mode = Ac4OutputMode::Multi3;
    spec.output.file_names = vec![
        "output_a.mp4".to_string(),
        "output_b.mp4".to_string(),
        "output_c.mp4".to_string(),
    ];

    let err = resolve_job(spec, &Default::default())
        .expect_err("multi3 + mp4 should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_atmos_v1' only supports output.ac4_output_mode='multi3' when output.container='ac4'"
    );
}

#[test]
fn rejects_ac4_multi3_with_wrong_output_count() {
    let mut spec = read_job(Path::new("examples/ac4_ims_atmos_multi3.ac4.yaml"))
        .expect("load ac4 ims atmos multi3 example");
    spec.output.file_names = vec!["output_a.ac4".to_string(), "output_b.ac4".to_string()];

    let err = resolve_job(spec, &Default::default())
        .expect_err("multi3 output count should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_atmos_v1' requires exactly 3 output file name(s) when output.ac4_output_mode='multi3'"
    );
}

#[test]
fn rejects_ac4_container_with_non_ac4_extension() {
    let mut spec = load_pcm_example();
    spec.output.file_names = vec!["output.mp4".to_string()];

    let err = resolve_job(spec, &Default::default())
        .expect_err("mismatched ac4 extension should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_pcm_v1' requires output.file_names[0] to end with '.ac4' when output.container='ac4'"
    );
}

#[test]
fn rejects_ac4_multi3_for_pcm_template() {
    let mut spec = load_pcm_example();
    spec.output.ac4_output_mode = Ac4OutputMode::Multi3;
    spec.output.file_names = vec![
        "output_a.ac4".to_string(),
        "output_b.ac4".to_string(),
        "output_c.ac4".to_string(),
    ];

    let err = resolve_job(spec, &Default::default())
        .expect_err("pcm multi3 should fail")
        .to_string();
    assert_eq!(
        err,
        "output.ac4_output_mode='multi3' is only supported by template_id 'ac4_ims_atmos_v1'"
    );
}

#[test]
fn rejects_both_pcm_input_shapes() {
    let mut spec = load_pcm_example();
    spec.inputs.as_mut().expect("inputs").wav_list = Some(IoSpec {
        storage_path: repo_root().join("testfiles").display().to_string(),
        file_names: vec![
            "input_6ch.wav".to_string(),
            "input_6ch.wav".to_string(),
            "input_6ch.wav".to_string(),
            "input_6ch.wav".to_string(),
            "input_6ch.wav".to_string(),
            "input_6ch.wav".to_string(),
        ],
    });

    let err = resolve_job(spec, &Default::default())
        .expect_err("dual pcm inputs should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_pcm_v1' accepts either inputs.wav or inputs.wav_list, but not both"
    );
}

#[test]
fn rejects_non_51_pcm_wav_input() {
    let mut spec = load_pcm_example();
    spec.inputs.as_mut().expect("inputs").wav = Some(IoSpec {
        storage_path: repo_root().join("testfiles").display().to_string(),
        file_names: vec!["16ch.wav".to_string()],
    });

    let err = resolve_job(spec, &Default::default())
        .expect_err("16ch wav should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_pcm_v1' requires inputs.wav to be a single 5.1 WAV; got 16 channels"
    );
}

#[test]
fn defaults_follow_official_values_for_ac4_ims() {
    let resolved = resolve_job(
        read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml")).expect("load example"),
        &Default::default(),
    )
    .expect("resolve");

    let ResolvedFilter::Ac4ImsAtmosV1(filter) = resolved.filter else {
        panic!("expected ac4 ims atmos filter");
    };
    assert_eq!(filter.data_rate, 256);
    assert_eq!(filter.ac4_frame_rate, "native");
    assert_eq!(filter.encoding_profile, "ims");
    assert_eq!(filter.iframe_interval, 0);
}

#[test]
fn music_profile_defaults_to_ims_music() {
    let mut spec = load_pcm_example();
    spec.profile = Profile::Music;
    let resolved = resolve_with_defaults(spec).expect("resolve music profile");

    let ResolvedFilter::Ac4ImsPcmV1(filter) = resolved.filter else {
        panic!("expected ac4 ims pcm filter");
    };
    assert_eq!(filter.encoding_profile, "ims_music");
}

#[test]
fn rejects_invalid_ac4_parameter_values() {
    let mut spec = load_pcm_example();
    spec.filter.data_rate = Some(999);
    let err = resolve_job(spec, &Default::default())
        .expect_err("invalid data_rate should fail")
        .to_string();
    assert!(err.contains("invalid data_rate"));
}

#[test]
fn rejects_reserved_language_tags_for_ac4() {
    let mut spec = load_pcm_example();
    spec.filter.language = Some("und".to_string());
    let err = resolve_job(spec, &Default::default())
        .expect_err("reserved language should fail")
        .to_string();
    assert!(err.contains("reserved language tags"));
}

#[test]
fn rejects_iframe_interval_1_for_ac4_ims_atmos() {
    let mut spec = read_job(Path::new("examples/ac4_ims_atmos_single.ac4.yaml"))
        .expect("load ac4 ims atmos example");
    spec.filter.iframe_interval = Some(1);

    let err = resolve_job(spec, &Default::default())
        .expect_err("iframe_interval=1 should fail for atmos lane")
        .to_string();
    assert_eq!(
        err,
        "invalid iframe_interval 1: local DEE 5.2.1 runtime rejects this value for AC-4 even though the official docs describe 0-1000"
    );
}

#[test]
fn rejects_iframe_interval_1_for_ac4_ims_pcm() {
    let mut spec = load_pcm_example();
    spec.filter.iframe_interval = Some(1);

    let err = resolve_job(spec, &Default::default())
        .expect_err("iframe_interval=1 should fail for pcm lane")
        .to_string();
    assert_eq!(
        err,
        "invalid iframe_interval 1: local DEE 5.2.1 runtime rejects this value for AC-4 even though the official docs describe 0-1000"
    );
}

#[test]
fn renders_json_output_for_ac4_ims_pcm_templates() {
    let resolved = resolve_job(load_pcm_example(), &Default::default()).expect("resolve");
    let rendered = render_config(&resolved, RenderFormat::Json).expect("render json");
    assert!(rendered.contains("\"encode_to_ims_ac4\""));
    assert!(rendered.contains("\"wav\""));
    assert!(rendered.contains("\"ac4\""));
}

#[test]
fn renders_json_output_for_ac4_mp4_container() {
    let mut spec = load_pcm_example();
    spec.output.container = OutputContainer::Mp4;
    spec.output.file_names = vec!["output.mp4".to_string()];

    let resolved = resolve_job(spec, &Default::default()).expect("resolve");
    let rendered = render_config(&resolved, RenderFormat::Json).expect("render json");
    assert!(rendered.contains("\"mp4\""));
    assert!(rendered.contains("\"output_format\": \"mp4\""));
    assert!(rendered.contains("\"override_frame_rate\": \"no\""));
    assert!(rendered.contains("\"fill_video\": false"));
}

#[test]
fn rejects_legacy_ac4_template_id_with_migration_message() {
    let mut spec = load_pcm_example();
    spec.template_id = Some("ac4_v1".to_string());
    let err = resolve_job(spec, &Default::default())
        .expect_err("legacy template should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_v1' was removed; use 'ac4_ims_atmos_v1' for atmos_mezz inputs or 'ac4_ims_pcm_v1' for wav/wav_list inputs"
    );
}

#[test]
fn still_rejects_album_mode_for_ac4_ims_templates() {
    let mut spec = load_pcm_example();
    spec.job_mode = JobMode::Album;
    let err = resolve_job(spec, &Default::default())
        .expect_err("album should fail")
        .to_string();
    assert_eq!(
        err,
        "template_id 'ac4_ims_pcm_v1' only supports job_mode=single"
    );
}
