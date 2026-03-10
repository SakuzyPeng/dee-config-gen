use anyhow::{Result, bail};

use crate::{
    config::{
        DEFAULT_TEMPLATE_ID, EncodeMode, JobFile, JobMode, Profile, RunSpec, normalize_drive,
        normalize_windows_path,
    },
    media::{InputMediaInfo, probe_audio_inputs, validate_consistent_audio_inputs},
    schema::validate::{ConstraintContext, evaluate_constraints},
    template::{
        Template, TemplateRegistry, atmos_ec3_v1::AtmosEc3V1Filter, pcm_ddp_v1::PcmDdpV1Filter,
    },
};

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    pub template_override: Option<String>,
    pub allow_fixed_override: bool,
    pub windows_drive: char,
}

#[derive(Debug, Clone)]
pub struct ResolvedJob {
    pub template_id: String,
    pub profile: Profile,
    pub job_mode: JobMode,
    pub encode_mode: EncodeMode,
    pub input_media: Vec<InputMediaInfo>,
    pub input: ResolvedIo,
    pub output: ResolvedIo,
    pub misc: ResolvedMisc,
    pub filter: ResolvedFilter,
    pub run: RunSpec,
}

#[derive(Debug, Clone)]
pub struct ResolvedIo {
    pub storage_path: String,
    pub file_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedMisc {
    pub temp_dir: String,
    pub clean_temp: bool,
}

#[derive(Debug, Clone)]
pub enum ResolvedFilter {
    AtmosEc3V1(AtmosEc3V1Filter),
    PcmDdpV1(PcmDdpV1Filter),
}

impl ResolvedFilter {
    pub fn param_some(&self, key: &str) -> bool {
        match self {
            Self::AtmosEc3V1(filter) => match key {
                "encoding_backend" => filter.encoding_backend.is_some(),
                "encoder_mode" => filter.encoder_mode.is_some(),
                _ => false,
            },
            Self::PcmDdpV1(filter) => match key {
                "encoder_mode" => !filter.encoder_mode.is_empty(),
                _ => false,
            },
        }
    }
}

pub fn resolve_job(spec: JobFile, options: &ResolveOptions) -> Result<ResolvedJob> {
    let drive = normalize_drive(options.windows_drive)?;

    let template_id = options
        .template_override
        .clone()
        .or(spec.template_id.clone())
        .unwrap_or_else(|| DEFAULT_TEMPLATE_ID.to_string());

    let template = TemplateRegistry::get(&template_id)?;

    if template_id == "atmos_ec3_v1"
        && matches!(
            spec.encode_mode,
            EncodeMode::Dd | EncodeMode::Ddp | EncodeMode::Ddp71
        )
    {
        bail!(
            "template_id 'atmos_ec3_v1' does not support encode_mode '{}'; use template_id 'pcm_ddp_v1'",
            spec.encode_mode.as_str()
        );
    }

    if !template.valid_profiles().contains(&spec.profile.as_str()) {
        bail!(
            "unsupported profile '{}'; allowed: {}",
            spec.profile.as_str(),
            template.valid_profiles().join(", ")
        );
    }

    if !template
        .valid_encode_modes()
        .contains(&spec.encode_mode.as_str())
    {
        bail!(
            "unsupported encode_mode '{}'; allowed: {}",
            spec.encode_mode.as_str(),
            template.valid_encode_modes().join(", ")
        );
    }

    validate_file_lists(
        spec.job_mode,
        &spec.input.file_names,
        &spec.output.file_names,
    )?;

    let input_media = if template.requires_input_media() {
        let input_media = probe_audio_inputs(&spec.input.storage_path, &spec.input.file_names)?;
        if matches!(spec.job_mode, JobMode::Album) {
            validate_consistent_audio_inputs(&input_media)?;
        }
        input_media
    } else {
        Vec::new()
    };

    let input = ResolvedIo {
        storage_path: normalize_windows_path(&spec.input.storage_path, drive),
        file_names: normalize_file_names(&spec.input.file_names)?,
    };
    let output = ResolvedIo {
        storage_path: normalize_windows_path(&spec.output.storage_path, drive),
        file_names: normalize_file_names(&spec.output.file_names)?,
    };
    let misc = ResolvedMisc {
        temp_dir: normalize_windows_path(&spec.misc.temp_dir, drive),
        clean_temp: spec.misc.clean_temp,
    };

    let mut filter = template.defaults(spec.profile, spec.encode_mode);
    template.apply_overrides(&mut filter, &spec.filter, spec.encode_mode)?;
    evaluate_template_constraints(template, &filter, spec.profile, spec.encode_mode, options)?;
    template.validate_runtime_compatibility(&filter, spec.encode_mode, &input_media)?;

    Ok(ResolvedJob {
        template_id,
        profile: spec.profile,
        job_mode: spec.job_mode,
        encode_mode: spec.encode_mode,
        input_media,
        input,
        output,
        misc,
        filter,
        run: spec.run,
    })
}

fn evaluate_template_constraints(
    template: &dyn Template,
    filter: &ResolvedFilter,
    profile: Profile,
    encode_mode: EncodeMode,
    options: &ResolveOptions,
) -> Result<()> {
    evaluate_constraints(
        template.constraints(),
        &ConstraintContext {
            profile: profile.as_str(),
            encode_mode: encode_mode.as_str(),
            allow_fixed_override: options.allow_fixed_override,
        },
        |key| template.constraint_value(filter, key),
    )
}

fn validate_file_lists(
    job_mode: JobMode,
    input_names: &[String],
    output_names: &[String],
) -> Result<()> {
    if input_names.is_empty() || output_names.is_empty() {
        bail!("input.file_names and output.file_names must not be empty");
    }

    match job_mode {
        JobMode::Single => {
            if input_names.len() != 1 || output_names.len() != 1 {
                bail!("job_mode=single requires exactly one input and one output file name");
            }
        }
        JobMode::Album => {
            if input_names.len() != output_names.len() {
                bail!(
                    "job_mode=album requires equal input/output counts; got {} and {}",
                    input_names.len(),
                    output_names.len()
                );
            }
        }
    }

    Ok(())
}

fn normalize_file_names(names: &[String]) -> Result<Vec<String>> {
    names
        .iter()
        .map(|n| {
            let trimmed = n.trim();
            if trimmed.is_empty() {
                bail!("file_name entries must not be empty")
            }
            Ok(trimmed.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::{ResolveOptions, resolve_job};
    use crate::{
        config::{EncodeMode, JobFile, Profile},
        test_support::sample_job_file,
    };

    fn resolve_with_default_options(job: JobFile) -> Result<super::ResolvedJob> {
        resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
    }

    fn set_enum_override(job: &mut JobFile, key: &str, value: &str) {
        let value = Some(value.to_string());
        match key {
            "metering_mode" => job.filter.metering_mode = value,
            "bitstream_mode" => job.filter.bitstream_mode = value,
            "downmix_config" => job.filter.downmix_config = value,
            "timecode_frame_rate" => job.filter.timecode_frame_rate = value,
            "time_base" => job.filter.time_base = value,
            "dolby_surround_mode" => job.filter.dolby_surround_mode = value,
            "dolby_surround_ex_mode" => job.filter.dolby_surround_ex_mode = value,
            "line_mode_drc_profile" => job.filter.line_mode_drc_profile = value,
            "rf_mode_drc_profile" => job.filter.rf_mode_drc_profile = value,
            "loro_center_mix_level" => job.filter.loro_center_mix_level = value,
            "loro_surround_mix_level" => job.filter.loro_surround_mix_level = value,
            "ltrt_center_mix_level" => job.filter.ltrt_center_mix_level = value,
            "ltrt_surround_mix_level" => job.filter.ltrt_surround_mix_level = value,
            "preferred_downmix_mode" => job.filter.preferred_downmix_mode = value,
            "starting_timecode" => job.filter.starting_timecode = value,
            "frame_rate" => job.filter.frame_rate = value,
            "surround_trim_5_1" => job.filter.surround_trim_5_1 = value,
            "surround_trim_7_1" => job.filter.surround_trim_7_1 = value,
            "height_trim_5_1" => job.filter.height_trim_5_1 = value,
            "encoding_backend" => job.filter.encoding_backend = value,
            "encoder_mode" => job.filter.encoder_mode = value,
            other => panic!("unsupported enum parameter in test: {other}"),
        }
    }

    #[test]
    fn injects_bluray_defaults() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Bluray;

        let resolved = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap();

        let super::ResolvedFilter::AtmosEc3V1(filter) = &resolved.filter else {
            panic!("expected AtmosEc3V1 filter");
        };
        assert_eq!(filter.data_rate, 1280);
        assert_eq!(filter.encoding_backend.as_deref(), Some("atmosprocessor"));
        assert_eq!(filter.encoder_mode.as_deref(), Some("bluray"));
    }

    #[test]
    fn rejects_atmos_ddp71_with_migration_message() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Ddp71;

        let err = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            err,
            "template_id 'atmos_ec3_v1' does not support encode_mode 'ddp71'; use template_id 'pcm_ddp_v1'"
        );
    }

    #[test]
    fn locks_music_fixed_values_without_override() {
        let mut job = sample_job_file();
        job.profile = Profile::Music;
        job.filter.line_mode_drc_profile = Some("film_light".to_string());

        let err = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap_err()
        .to_string();

        assert!(err.contains("profile=music"));
    }

    #[test]
    fn accepts_music_override_when_flag_enabled() {
        let mut job = sample_job_file();
        job.profile = Profile::Music;
        job.filter.line_mode_drc_profile = Some("film_light".to_string());

        let resolved = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: true,
                windows_drive: 'Y',
            },
        )
        .unwrap();

        let super::ResolvedFilter::AtmosEc3V1(filter) = &resolved.filter else {
            panic!("expected AtmosEc3V1 filter");
        };
        assert_eq!(filter.line_mode_drc_profile, "film_light");
    }

    #[test]
    fn rejects_bluray_data_rate_over_hard_max() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Bluray;
        job.filter.data_rate = Some(1800);

        assert!(
            resolve_job(
                job,
                &ResolveOptions {
                    template_override: None,
                    allow_fixed_override: false,
                    windows_drive: 'Y',
                },
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_invalid_encode_mode_for_pcm_template() {
        let mut job = sample_job_file();
        job.template_id = Some("pcm_ddp_v1".to_string());
        job.encode_mode = EncodeMode::Streaming;

        let err = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            err,
            "unsupported encode_mode 'streaming'; allowed: dd, ddp, bluray, ddp71"
        );
    }

    #[test]
    fn rejects_invalid_values_for_all_enum_filter_params() {
        let cases = [
            ("metering_mode", EncodeMode::Streaming),
            ("timecode_frame_rate", EncodeMode::Streaming),
            ("time_base", EncodeMode::Streaming),
            ("line_mode_drc_profile", EncodeMode::Streaming),
            ("rf_mode_drc_profile", EncodeMode::Streaming),
            ("loro_center_mix_level", EncodeMode::Streaming),
            ("loro_surround_mix_level", EncodeMode::Streaming),
            ("ltrt_center_mix_level", EncodeMode::Streaming),
            ("ltrt_surround_mix_level", EncodeMode::Streaming),
            ("preferred_downmix_mode", EncodeMode::Streaming),
            ("surround_trim_5_1", EncodeMode::Streaming),
            ("surround_trim_7_1", EncodeMode::Streaming),
            ("height_trim_5_1", EncodeMode::Streaming),
            ("encoding_backend", EncodeMode::Bluray),
            ("encoder_mode", EncodeMode::Bluray),
        ];

        for (key, encode_mode) in cases {
            let mut job = sample_job_file();
            job.encode_mode = encode_mode;
            set_enum_override(&mut job, key, "__invalid__");

            let err = resolve_with_default_options(job).unwrap_err().to_string();
            assert!(
                err.contains(&format!("invalid value '__invalid__' for {key}")),
                "expected invalid enum error for {key}, got: {err}"
            );
        }
    }

    #[test]
    fn validates_custom_dialnorm_boundaries() {
        for ok in [-31_i8, 0_i8] {
            let mut job = sample_job_file();
            job.filter.custom_dialnorm = Some(ok);
            assert!(
                resolve_with_default_options(job).is_ok(),
                "expected custom_dialnorm={ok} to be accepted"
            );
        }

        for invalid in [-32_i8, 1_i8] {
            let mut job = sample_job_file();
            job.filter.custom_dialnorm = Some(invalid);
            let err = resolve_with_default_options(job).unwrap_err().to_string();
            assert!(err.contains("custom_dialnorm"));
            assert!(err.contains("expected -31..0"));
        }
    }

    #[test]
    fn validates_streaming_bitrate_boundaries() {
        for ok in [384_u16, 1024_u16] {
            let mut job = sample_job_file();
            job.encode_mode = EncodeMode::Streaming;
            job.filter.data_rate = Some(ok);
            assert!(
                resolve_with_default_options(job).is_ok(),
                "expected streaming data_rate={ok} to be accepted"
            );
        }

        for invalid in [383_u16, 1025_u16] {
            let mut job = sample_job_file();
            job.encode_mode = EncodeMode::Streaming;
            job.filter.data_rate = Some(invalid);
            let err = resolve_with_default_options(job).unwrap_err().to_string();
            assert!(err.contains("invalid data_rate"));
            assert!(err.contains("mode 'streaming'"));
        }
    }

    #[test]
    fn rejects_bluray_bitrates_below_runtime_minimum() {
        for invalid in [768_u16, 1024_u16] {
            let mut job = sample_job_file();
            job.encode_mode = EncodeMode::Bluray;
            job.filter.data_rate = Some(invalid);

            let err = resolve_with_default_options(job).unwrap_err().to_string();
            assert!(err.contains("invalid data_rate"));
            assert!(err.contains("mode 'bluray'"));
            assert!(err.contains("1152, 1280, 1408, 1512, 1536, 1664"));
        }
    }

    #[test]
    fn rejects_atmos_bluray_ltrt_pl2_preferred_downmix_mode() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Bluray;
        job.filter.preferred_downmix_mode = Some("ltrt-pl2".to_string());

        let err = resolve_with_default_options(job).unwrap_err().to_string();
        assert_eq!(
            err,
            "Preferred Downmix mode Pro Logic II is not supported in Blu-ray Mode"
        );
    }

    #[test]
    fn rejects_encoding_backend_in_streaming_mode() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Streaming;
        job.filter.encoding_backend = Some("atmosprocessor".to_string());

        let err = resolve_with_default_options(job).unwrap_err().to_string();
        assert_eq!(
            err,
            "encoding_backend/encoder_mode are mode extensions and cannot be set for streaming mode"
        );
    }
}
