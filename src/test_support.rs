use crate::{
    media::InputMediaInfo,
    resolve::{ResolvedFilter, ResolvedIo, ResolvedJob, ResolvedMisc, ResolvedOutput},
    spec::{
        Ac4OutputMode, DEFAULT_TEMPLATE_ID, EncodeMode, FilterOverrides, IoSpec, JobMode, JobSpec,
        MiscSpec, OutputContainer, OutputSpec, Profile, RunSpec,
    },
    template::atmos_ec3_v1::AtmosEc3V1Filter,
};

pub fn sample_job_file() -> JobSpec {
    JobSpec {
        template_id: Some(DEFAULT_TEMPLATE_ID.to_string()),
        profile: Profile::Standard,
        job_mode: JobMode::Single,
        encode_mode: EncodeMode::Streaming,
        input: IoSpec {
            storage_path: "/tmp/in".to_string(),
            file_names: vec!["a.wav".to_string()],
        },
        inputs: None,
        output: OutputSpec {
            storage_path: "/tmp/out".to_string(),
            file_names: vec!["a.ec3".to_string()],
            container: OutputContainer::Ac4,
            ac4_output_mode: Ac4OutputMode::Single,
        },
        misc: MiscSpec {
            temp_dir: "/tmp/dee".to_string(),
            clean_temp: true,
        },
        filter: FilterOverrides::default(),
        run: RunSpec::default(),
    }
}

pub fn sample_atmos_filter() -> AtmosEc3V1Filter {
    AtmosEc3V1Filter {
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence: true,
        speech_threshold: 15,
        data_rate: 448,
        timecode_frame_rate: "not_indicated".to_string(),
        start: "first_frame_of_action".to_string(),
        end: "end_of_file".to_string(),
        time_base: "file_position".to_string(),
        prepend_silence_duration: "0.0".to_string(),
        append_silence_duration: "0.0".to_string(),
        line_mode_drc_profile: "film_light".to_string(),
        rf_mode_drc_profile: "film_light".to_string(),
        loro_center_mix_level: "-3".to_string(),
        loro_surround_mix_level: "-3".to_string(),
        ltrt_center_mix_level: "-3".to_string(),
        ltrt_surround_mix_level: "-3".to_string(),
        preferred_downmix_mode: "loro".to_string(),
        surround_trim_5_1: "auto".to_string(),
        height_trim_5_1: "auto".to_string(),
        custom_dialnorm: 0,
        encoding_backend: None,
        encoder_mode: None,
    }
}

pub fn sample_resolved_job() -> ResolvedJob {
    ResolvedJob {
        template_id: DEFAULT_TEMPLATE_ID.to_string(),
        profile: Profile::Standard,
        job_mode: JobMode::Single,
        encode_mode: EncodeMode::Streaming,
        input_media: vec![InputMediaInfo {
            path: "/tmp/in/in.wav".to_string(),
            channels: 2,
            sample_rate: 48_000,
            bits_per_sample: 24,
            channel_layout: "stereo".to_string(),
            codec_name: "pcm_s24le".to_string(),
        }],
        input: ResolvedIo {
            storage_path: "Y:/in".to_string(),
            file_names: vec!["in.wav".to_string()],
        },
        input_groups: None,
        output: ResolvedOutput {
            storage_path: "Y:/out".to_string(),
            file_names: vec!["out.ec3".to_string()],
            container: OutputContainer::Ac4,
            ac4_output_mode: Ac4OutputMode::Single,
        },
        misc: ResolvedMisc {
            temp_dir: "Y:/tmp".to_string(),
            clean_temp: true,
        },
        filter: ResolvedFilter::AtmosEc3V1(sample_atmos_filter()),
        run: RunSpec::default(),
    }
}
