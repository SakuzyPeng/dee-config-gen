use anyhow::{Result, bail};

use crate::{
    config::{
        DEFAULT_TEMPLATE_ID, EncodeMode, JobFile, JobMode, Profile, RunSpec, normalize_drive,
        normalize_windows_path,
    },
    schema::validate::{ConstraintContext, evaluate_constraints},
    template::{Template, TemplateRegistry, atmos_ec3_v1::AtmosEc3V1Filter},
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
}

impl ResolvedFilter {
    pub fn param_some(&self, key: &str) -> bool {
        match self {
            Self::AtmosEc3V1(filter) => match key {
                "encoding_backend" => filter.encoding_backend.is_some(),
                "encoder_mode" => filter.encoder_mode.is_some(),
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

    Ok(ResolvedJob {
        template_id,
        profile: spec.profile,
        job_mode: spec.job_mode,
        encode_mode: spec.encode_mode,
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
            } else {
                Ok(trimmed.to_string())
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ResolveOptions, resolve_job};
    use crate::config::{
        DEFAULT_TEMPLATE_ID, EncodeMode, FilterOverrides, IoSpec, JobFile, JobMode, MiscSpec,
        Profile, RunSpec,
    };

    fn base_job() -> JobFile {
        JobFile {
            template_id: Some(DEFAULT_TEMPLATE_ID.to_string()),
            profile: Profile::Standard,
            job_mode: JobMode::Single,
            encode_mode: EncodeMode::Streaming,
            input: IoSpec {
                storage_path: "/tmp/in".to_string(),
                file_names: vec!["a.wav".to_string()],
            },
            output: IoSpec {
                storage_path: "/tmp/out".to_string(),
                file_names: vec!["a.ec3".to_string()],
            },
            misc: MiscSpec {
                temp_dir: "/tmp/dee".to_string(),
                clean_temp: true,
            },
            filter: FilterOverrides::default(),
            run: RunSpec::default(),
        }
    }

    #[test]
    fn injects_bluray_defaults() {
        let mut job = base_job();
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

        let super::ResolvedFilter::AtmosEc3V1(filter) = &resolved.filter;
        assert_eq!(filter.data_rate, 1280);
        assert_eq!(filter.encoding_backend.as_deref(), Some("atmosprocessor"));
        assert_eq!(filter.encoder_mode.as_deref(), Some("bluray"));
    }

    #[test]
    fn injects_ddp71_defaults() {
        let mut job = base_job();
        job.encode_mode = EncodeMode::Ddp71;

        let resolved = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap();

        let super::ResolvedFilter::AtmosEc3V1(filter) = &resolved.filter;
        assert_eq!(filter.data_rate, 1024);
        assert_eq!(filter.encoding_backend, None);
        assert_eq!(filter.encoder_mode.as_deref(), Some("ddp71"));
        assert_eq!(filter.surround_trim_7_1, "auto");
    }

    #[test]
    fn locks_music_fixed_values_without_override() {
        let mut job = base_job();
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
        let mut job = base_job();
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

        let super::ResolvedFilter::AtmosEc3V1(filter) = &resolved.filter;
        assert_eq!(filter.line_mode_drc_profile, "film_light");
    }

    #[test]
    fn rejects_bluray_data_rate_over_hard_max() {
        let mut job = base_job();
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
    fn rejects_encoding_backend_in_ddp71_mode() {
        let mut job = base_job();
        job.encode_mode = EncodeMode::Ddp71;
        job.filter.encoding_backend = Some("atmosprocessor".to_string());

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

        assert!(err.contains("encoding_backend"));
    }
}
