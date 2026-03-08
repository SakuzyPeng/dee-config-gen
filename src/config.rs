use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer};

use crate::registry::{canonicalize_enum, validate_data_rate, validate_int};

pub const DEFAULT_TEMPLATE_ID: &str = "atmos_ec3_v1";

#[derive(Debug, Clone, Deserialize)]
pub struct JobFile {
    pub template_id: Option<String>,
    #[serde(default)]
    pub profile: Profile,
    #[serde(default)]
    pub job_mode: JobMode,
    #[serde(default)]
    pub atmos_mode: AtmosMode,
    pub input: IoSpec,
    pub output: IoSpec,
    pub misc: MiscSpec,
    #[serde(default)]
    pub filter: FilterOverrides,
    #[serde(default)]
    pub run: RunSpec,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IoSpec {
    pub storage_path: String,
    #[serde(
        alias = "file_name",
        alias = "files",
        deserialize_with = "deserialize_string_or_vec"
    )]
    pub file_names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiscSpec {
    pub temp_dir: String,
    #[serde(default = "default_clean_temp")]
    pub clean_temp: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RunSpec {
    #[serde(default)]
    pub runner_args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FilterOverrides {
    pub metering_mode: Option<String>,
    pub dialogue_intelligence: Option<bool>,
    pub speech_threshold: Option<u8>,
    pub data_rate: Option<u16>,
    pub timecode_frame_rate: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub time_base: Option<String>,
    pub prepend_silence_duration: Option<String>,
    pub append_silence_duration: Option<String>,
    pub line_mode_drc_profile: Option<String>,
    pub rf_mode_drc_profile: Option<String>,
    pub loro_center_mix_level: Option<String>,
    pub loro_surround_mix_level: Option<String>,
    pub ltrt_center_mix_level: Option<String>,
    pub ltrt_surround_mix_level: Option<String>,
    pub preferred_downmix_mode: Option<String>,
    pub surround_trim_5_1: Option<String>,
    pub height_trim_5_1: Option<String>,
    pub custom_dialnorm: Option<i8>,
    pub encoding_backend: Option<String>,
    pub encoder_mode: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[default]
    Standard,
    Music,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobMode {
    #[default]
    Single,
    Album,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AtmosMode {
    #[default]
    Streaming,
    Bluray,
}

impl AtmosMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::Bluray => "bluray",
        }
    }
}

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
    pub atmos_mode: AtmosMode,
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
pub struct ResolvedFilter {
    pub metering_mode: String,
    pub dialogue_intelligence: bool,
    pub speech_threshold: u8,
    pub data_rate: u16,
    pub timecode_frame_rate: String,
    pub start: String,
    pub end: String,
    pub time_base: String,
    pub prepend_silence_duration: String,
    pub append_silence_duration: String,
    pub line_mode_drc_profile: String,
    pub rf_mode_drc_profile: String,
    pub loro_center_mix_level: String,
    pub loro_surround_mix_level: String,
    pub ltrt_center_mix_level: String,
    pub ltrt_surround_mix_level: String,
    pub preferred_downmix_mode: String,
    pub surround_trim_5_1: String,
    pub height_trim_5_1: String,
    pub custom_dialnorm: i8,
    pub encoding_backend: Option<String>,
    pub encoder_mode: Option<String>,
}

fn default_clean_temp() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StringOrVec {
    One(String),
    Many(Vec<String>),
}

fn deserialize_string_or_vec<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match StringOrVec::deserialize(deserializer)? {
        StringOrVec::One(v) => Ok(vec![v]),
        StringOrVec::Many(v) => Ok(v),
    }
}

pub fn load_job_file(path: &Path) -> Result<JobFile> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read job file: {}", path.display()))?;

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "json" => serde_json::from_str(&text)
            .or_else(|_| serde_yaml::from_str(&text))
            .context("failed to parse input as JSON or YAML"),
        _ => serde_yaml::from_str(&text)
            .or_else(|_| serde_json::from_str(&text))
            .context("failed to parse input as YAML or JSON"),
    }
}

pub fn resolve_job(spec: JobFile, options: &ResolveOptions) -> Result<ResolvedJob> {
    let drive = normalize_drive(options.windows_drive)?;

    let template_id = options
        .template_override
        .clone()
        .or(spec.template_id.clone())
        .unwrap_or_else(|| DEFAULT_TEMPLATE_ID.to_string());

    if template_id != DEFAULT_TEMPLATE_ID {
        bail!("unsupported template_id '{template_id}'; only '{DEFAULT_TEMPLATE_ID}' is supported");
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

    let mut filter = default_filter(spec.profile, spec.atmos_mode);
    apply_overrides(&mut filter, &spec.filter, spec.atmos_mode)?;

    enforce_music_fixed_values(spec.profile, &filter, options.allow_fixed_override)?;

    if spec.atmos_mode == AtmosMode::Streaming {
        if spec.filter.encoding_backend.is_some() || spec.filter.encoder_mode.is_some() {
            bail!(
                "encoding_backend/encoder_mode are bluray extensions and cannot be set for streaming mode"
            );
        }
        filter.encoding_backend = None;
        filter.encoder_mode = None;
    } else {
        if filter.encoding_backend.is_none() {
            filter.encoding_backend = Some("atmosprocessor".to_string());
        }
        if filter.encoder_mode.is_none() {
            filter.encoder_mode = Some("bluray".to_string());
        }
    }

    Ok(ResolvedJob {
        template_id,
        profile: spec.profile,
        job_mode: spec.job_mode,
        atmos_mode: spec.atmos_mode,
        input,
        output,
        misc,
        filter,
        run: spec.run,
    })
}

fn default_filter(profile: Profile, atmos_mode: AtmosMode) -> ResolvedFilter {
    let (dialogue_intelligence, speech_threshold, drc) = match profile {
        Profile::Standard => (true, 15, "film_light"),
        Profile::Music => (false, 100, "music_light"),
    };

    ResolvedFilter {
        metering_mode: "1770-4".to_string(),
        dialogue_intelligence,
        speech_threshold,
        data_rate: match atmos_mode {
            AtmosMode::Streaming => 448,
            AtmosMode::Bluray => 1280,
        },
        timecode_frame_rate: "not_indicated".to_string(),
        start: "first_frame_of_action".to_string(),
        end: "end_of_file".to_string(),
        time_base: "file_position".to_string(),
        prepend_silence_duration: "0.0".to_string(),
        append_silence_duration: "0.0".to_string(),
        line_mode_drc_profile: drc.to_string(),
        rf_mode_drc_profile: drc.to_string(),
        loro_center_mix_level: "-3".to_string(),
        loro_surround_mix_level: "-3".to_string(),
        ltrt_center_mix_level: "-3".to_string(),
        ltrt_surround_mix_level: "-3".to_string(),
        preferred_downmix_mode: "loro".to_string(),
        surround_trim_5_1: "auto".to_string(),
        height_trim_5_1: "auto".to_string(),
        custom_dialnorm: 0,
        encoding_backend: match atmos_mode {
            AtmosMode::Streaming => None,
            AtmosMode::Bluray => Some("atmosprocessor".to_string()),
        },
        encoder_mode: match atmos_mode {
            AtmosMode::Streaming => None,
            AtmosMode::Bluray => Some("bluray".to_string()),
        },
    }
}

fn apply_overrides(
    filter: &mut ResolvedFilter,
    overrides: &FilterOverrides,
    atmos_mode: AtmosMode,
) -> Result<()> {
    if let Some(v) = &overrides.metering_mode {
        filter.metering_mode = canonicalize_enum("metering_mode", v)?;
    }
    if let Some(v) = overrides.dialogue_intelligence {
        filter.dialogue_intelligence = v;
    }
    if let Some(v) = overrides.speech_threshold {
        validate_int("speech_threshold", i64::from(v))?;
        filter.speech_threshold = v;
    }
    if let Some(v) = overrides.data_rate {
        validate_data_rate(v, atmos_mode.as_str())?;
        filter.data_rate = v;
    }
    if let Some(v) = &overrides.timecode_frame_rate {
        filter.timecode_frame_rate = canonicalize_enum("timecode_frame_rate", v)?;
    }
    if let Some(v) = &overrides.start {
        filter.start = v.clone();
    }
    if let Some(v) = &overrides.end {
        filter.end = v.clone();
    }
    if let Some(v) = &overrides.time_base {
        filter.time_base = canonicalize_enum("time_base", v)?;
    }
    if let Some(v) = &overrides.prepend_silence_duration {
        filter.prepend_silence_duration = v.clone();
    }
    if let Some(v) = &overrides.append_silence_duration {
        filter.append_silence_duration = v.clone();
    }
    if let Some(v) = &overrides.line_mode_drc_profile {
        filter.line_mode_drc_profile = canonicalize_enum("line_mode_drc_profile", v)?;
    }
    if let Some(v) = &overrides.rf_mode_drc_profile {
        filter.rf_mode_drc_profile = canonicalize_enum("rf_mode_drc_profile", v)?;
    }
    if let Some(v) = &overrides.loro_center_mix_level {
        filter.loro_center_mix_level = canonicalize_enum("loro_center_mix_level", v)?;
    }
    if let Some(v) = &overrides.loro_surround_mix_level {
        filter.loro_surround_mix_level = canonicalize_enum("loro_surround_mix_level", v)?;
    }
    if let Some(v) = &overrides.ltrt_center_mix_level {
        filter.ltrt_center_mix_level = canonicalize_enum("ltrt_center_mix_level", v)?;
    }
    if let Some(v) = &overrides.ltrt_surround_mix_level {
        filter.ltrt_surround_mix_level = canonicalize_enum("ltrt_surround_mix_level", v)?;
    }
    if let Some(v) = &overrides.preferred_downmix_mode {
        filter.preferred_downmix_mode = canonicalize_enum("preferred_downmix_mode", v)?;
    }
    if let Some(v) = &overrides.surround_trim_5_1 {
        filter.surround_trim_5_1 = canonicalize_enum("surround_trim_5_1", v)?;
    }
    if let Some(v) = &overrides.height_trim_5_1 {
        filter.height_trim_5_1 = canonicalize_enum("height_trim_5_1", v)?;
    }
    if let Some(v) = overrides.custom_dialnorm {
        validate_int("custom_dialnorm", i64::from(v))?;
        filter.custom_dialnorm = v;
    }
    if let Some(v) = &overrides.encoding_backend {
        filter.encoding_backend = Some(canonicalize_enum("encoding_backend", v)?);
    }
    if let Some(v) = &overrides.encoder_mode {
        filter.encoder_mode = Some(canonicalize_enum("encoder_mode", v)?);
    }

    validate_data_rate(filter.data_rate, atmos_mode.as_str())?;
    Ok(())
}

fn enforce_music_fixed_values(
    profile: Profile,
    filter: &ResolvedFilter,
    allow_fixed_override: bool,
) -> Result<()> {
    if profile != Profile::Music || allow_fixed_override {
        return Ok(());
    }

    let expected = [
        (
            "dialogue_intelligence",
            filter.dialogue_intelligence == false,
        ),
        ("speech_threshold", filter.speech_threshold == 100),
        (
            "line_mode_drc_profile",
            filter.line_mode_drc_profile == "music_light",
        ),
        (
            "rf_mode_drc_profile",
            filter.rf_mode_drc_profile == "music_light",
        ),
    ];

    if expected.iter().all(|(_, ok)| *ok) {
        return Ok(());
    }

    let mut messages = Vec::new();
    for (field, ok) in expected {
        if !ok {
            messages.push(field);
        }
    }

    bail!(
        "profile=music locks fixed fields by default. conflicting fields: {}. Use --allow-fixed-override to bypass.",
        messages.join(", ")
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

fn normalize_drive(drive: char) -> Result<char> {
    if !drive.is_ascii_alphabetic() {
        bail!("win_drive must be an ASCII drive letter, got '{drive}'");
    }
    Ok(drive.to_ascii_uppercase())
}

pub fn normalize_windows_path(path: &str, drive: char) -> String {
    let trimmed = path.trim().trim_matches('"').replace('\\', "/");

    if trimmed.len() >= 3 {
        let bytes = trimmed.as_bytes();
        if bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/' {
            return format!(
                "{}:{}",
                (bytes[0] as char).to_ascii_uppercase(),
                &trimmed[2..]
            );
        }
    }

    if trimmed.starts_with('/') {
        format!("{drive}:{trimmed}")
    } else {
        format!("{drive}:/{trimmed}")
    }
}

pub fn write_xml_output(path: &Path, xml: &str) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory: {}", parent.display()))?;
    }
    fs::write(path, xml).with_context(|| format!("failed to write XML: {}", path.display()))
}

pub fn default_xml_path_from_input(input_path: &Path) -> PathBuf {
    let mut out = input_path.to_path_buf();
    out.set_extension("xml");
    out
}

#[cfg(test)]
mod tests {
    use super::{
        AtmosMode, DEFAULT_TEMPLATE_ID, FilterOverrides, IoSpec, JobFile, JobMode, MiscSpec,
        Profile, ResolveOptions, RunSpec, resolve_job,
    };

    fn base_job() -> JobFile {
        JobFile {
            template_id: Some(DEFAULT_TEMPLATE_ID.to_string()),
            profile: Profile::Standard,
            job_mode: JobMode::Single,
            atmos_mode: AtmosMode::Streaming,
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
        job.atmos_mode = AtmosMode::Bluray;

        let resolved = resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .unwrap();

        assert_eq!(resolved.filter.data_rate, 1280);
        assert_eq!(
            resolved.filter.encoding_backend.as_deref(),
            Some("atmosprocessor")
        );
        assert_eq!(resolved.filter.encoder_mode.as_deref(), Some("bluray"));
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

        assert_eq!(resolved.filter.line_mode_drc_profile, "film_light");
    }

    #[test]
    fn rejects_bluray_data_rate_over_hard_max() {
        let mut job = base_job();
        job.atmos_mode = AtmosMode::Bluray;
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
}
