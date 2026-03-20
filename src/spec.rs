use std::collections::BTreeMap;
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer};

use crate::render::RenderFormat;
pub use crate::schema::{
    Constraint, FixedValue, ModeAvailability, OverridePolicy, ParamRule, ParamSchema, SourceTag,
    Value,
};
use crate::template::TemplateRegistry;

pub const DEFAULT_TEMPLATE_ID: &str = "atmos_ec3_v1";

/// Input model for a DEE job spec loaded from YAML or JSON.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobSpec {
    pub template_id: Option<String>,
    #[serde(default)]
    pub profile: Profile,
    #[serde(default)]
    pub job_mode: JobMode,
    #[serde(default)]
    pub encode_mode: EncodeMode,
    #[serde(default)]
    pub input: IoSpec,
    #[serde(default)]
    pub inputs: Option<InputsSpec>,
    pub output: OutputSpec,
    pub misc: MiscSpec,
    #[serde(default)]
    pub filter: FilterOverrides,
    #[serde(default)]
    pub run: RunSpec,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IoSpec {
    pub storage_path: String,
    #[serde(
        alias = "file_name",
        alias = "files",
        deserialize_with = "deserialize_string_or_vec"
    )]
    pub file_names: Vec<String>,
}

impl IoSpec {
    pub fn is_empty(&self) -> bool {
        self.storage_path.trim().is_empty() && self.file_names.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct OutputSpec {
    pub storage_path: String,
    #[serde(
        alias = "file_name",
        alias = "files",
        deserialize_with = "deserialize_string_or_vec"
    )]
    pub file_names: Vec<String>,
    #[serde(default)]
    pub container: OutputContainer,
    #[serde(default)]
    pub ac4_output_mode: Ac4OutputMode,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputContainer {
    #[default]
    Ac4,
    Mp4,
}

impl OutputContainer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ac4 => "ac4",
            Self::Mp4 => "mp4",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Ac4OutputMode {
    #[default]
    Single,
    Multi3,
}

impl Ac4OutputMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multi3 => "multi3",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct InputsSpec {
    #[serde(default)]
    pub atmos_mezz: Option<IoSpec>,
    #[serde(default)]
    pub wav: Option<IoSpec>,
    #[serde(default)]
    pub wav_list: Option<IoSpec>,
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
    pub channel_configuration: Option<String>,
    pub input_timecode_frame_rate: Option<String>,
    pub offset: Option<String>,
    pub ffoa: Option<String>,
    pub metering_mode: Option<String>,
    pub dialogue_intelligence: Option<bool>,
    pub speech_threshold: Option<u8>,
    pub data_rate: Option<u16>,
    pub ac4_frame_rate: Option<String>,
    pub ims_legacy_presentation: Option<bool>,
    pub iframe_interval: Option<u16>,
    pub language: Option<String>,
    pub encoding_profile: Option<String>,
    pub bitstream_mode: Option<String>,
    pub downmix_config: Option<String>,
    pub timecode_frame_rate: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
    pub time_base: Option<String>,
    pub prepend_silence_duration: Option<String>,
    pub append_silence_duration: Option<String>,
    pub lfe_on: Option<bool>,
    pub dolby_surround_mode: Option<String>,
    pub dolby_surround_ex_mode: Option<String>,
    pub user_data: Option<i32>,
    pub line_mode_drc_profile: Option<String>,
    pub rf_mode_drc_profile: Option<String>,
    pub ddp_drc_profile: Option<String>,
    pub flat_panel_drc_profile: Option<String>,
    pub home_theatre_drc_profile: Option<String>,
    pub portable_hp_drc_profile: Option<String>,
    pub portable_spkr_drc_profile: Option<String>,
    pub lfe_lowpass_filter: Option<bool>,
    pub surround_90_degree_phase_shift: Option<bool>,
    pub surround_3db_attenuation: Option<bool>,
    pub loro_center_mix_level: Option<String>,
    pub loro_surround_mix_level: Option<String>,
    pub ltrt_center_mix_level: Option<String>,
    pub ltrt_surround_mix_level: Option<String>,
    pub preferred_downmix_mode: Option<String>,
    pub allow_hybrid_downmix: Option<bool>,
    pub starting_timecode: Option<String>,
    pub frame_rate: Option<String>,
    pub surround_trim_5_1: Option<String>,
    pub height_trim_5_1: Option<String>,
    pub custom_dialnorm: Option<i8>,
    pub encoding_backend: Option<String>,
    pub encoder_mode: Option<String>,
    pub atmos_presentation_drc_profile: Option<String>,
    pub spatial_clusters: Option<String>,
    pub legacy_authoring_compatibility: Option<bool>,
    pub presentation_8ch_drc_profile: Option<String>,
    pub presentation_6ch_drc_profile: Option<String>,
    pub presentation_2ch_drc_profile: Option<String>,
    pub optimize_data_rate: Option<bool>,
}

impl FilterOverrides {
    pub fn is_empty(&self) -> bool {
        self.channel_configuration.is_none()
            && self.input_timecode_frame_rate.is_none()
            && self.offset.is_none()
            && self.ffoa.is_none()
            && self.metering_mode.is_none()
            && self.dialogue_intelligence.is_none()
            && self.speech_threshold.is_none()
            && self.data_rate.is_none()
            && self.ac4_frame_rate.is_none()
            && self.ims_legacy_presentation.is_none()
            && self.iframe_interval.is_none()
            && self.language.is_none()
            && self.encoding_profile.is_none()
            && self.bitstream_mode.is_none()
            && self.downmix_config.is_none()
            && self.timecode_frame_rate.is_none()
            && self.start.is_none()
            && self.end.is_none()
            && self.time_base.is_none()
            && self.prepend_silence_duration.is_none()
            && self.append_silence_duration.is_none()
            && self.lfe_on.is_none()
            && self.dolby_surround_mode.is_none()
            && self.dolby_surround_ex_mode.is_none()
            && self.user_data.is_none()
            && self.line_mode_drc_profile.is_none()
            && self.rf_mode_drc_profile.is_none()
            && self.ddp_drc_profile.is_none()
            && self.flat_panel_drc_profile.is_none()
            && self.home_theatre_drc_profile.is_none()
            && self.portable_hp_drc_profile.is_none()
            && self.portable_spkr_drc_profile.is_none()
            && self.lfe_lowpass_filter.is_none()
            && self.surround_90_degree_phase_shift.is_none()
            && self.surround_3db_attenuation.is_none()
            && self.loro_center_mix_level.is_none()
            && self.loro_surround_mix_level.is_none()
            && self.ltrt_center_mix_level.is_none()
            && self.ltrt_surround_mix_level.is_none()
            && self.preferred_downmix_mode.is_none()
            && self.allow_hybrid_downmix.is_none()
            && self.starting_timecode.is_none()
            && self.frame_rate.is_none()
            && self.surround_trim_5_1.is_none()
            && self.height_trim_5_1.is_none()
            && self.custom_dialnorm.is_none()
            && self.encoding_backend.is_none()
            && self.encoder_mode.is_none()
            && self.atmos_presentation_drc_profile.is_none()
            && self.spatial_clusters.is_none()
            && self.legacy_authoring_compatibility.is_none()
            && self.presentation_8ch_drc_profile.is_none()
            && self.presentation_6ch_drc_profile.is_none()
            && self.presentation_2ch_drc_profile.is_none()
            && self.optimize_data_rate.is_none()
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    #[default]
    Standard,
    Music,
}

impl Profile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Music => "music",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobMode {
    #[default]
    Single,
    Album,
}

impl JobMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Album => "album",
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EncodeMode {
    #[default]
    Streaming,
    Dd,
    Ddp,
    Mlp,
    Bluray,
    Ddp71,
    Ac4,
}

impl EncodeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::Dd => "dd",
            Self::Ddp => "ddp",
            Self::Mlp => "mlp",
            Self::Bluray => "bluray",
            Self::Ddp71 => "ddp71",
            Self::Ac4 => "ac4",
        }
    }
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

pub fn read_job(path: &Path) -> Result<JobSpec> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read job file: {}", path.display()))?;

    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "json" => parse_job_str_json_first(&text),
        _ => parse_job_str_yaml_first(&text),
    }
}

pub fn parse_job_str(input: &str) -> Result<JobSpec> {
    parse_job_str_yaml_first(input)
}

fn parse_job_str_yaml_first(input: &str) -> Result<JobSpec> {
    serde_yaml::from_str(input)
        .or_else(|_| serde_json::from_str(input))
        .context("failed to parse input as YAML or JSON")
}

fn parse_job_str_json_first(input: &str) -> Result<JobSpec> {
    serde_json::from_str(input)
        .or_else(|_| serde_yaml::from_str(input))
        .context("failed to parse input as JSON or YAML")
}

pub fn write_config_output(path: &Path, content: &str, format: RenderFormat) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory: {}", parent.display()))?;
    }
    fs::write(path, content).with_context(|| {
        format!(
            "failed to write {}: {}",
            format.as_str().to_ascii_uppercase(),
            path.display()
        )
    })
}

pub fn default_config_path_from_input(input_path: &Path, format: RenderFormat) -> PathBuf {
    let mut out = input_path.to_path_buf();
    out.set_extension(format.as_str());
    out
}

pub fn write_xml_output(path: &Path, xml: &str) -> Result<()> {
    write_config_output(path, xml, RenderFormat::Xml)
}

pub fn default_xml_path_from_input(input_path: &Path) -> PathBuf {
    default_config_path_from_input(input_path, RenderFormat::Xml)
}

pub fn template_metadata(template_id: &str) -> Result<TemplateMetadata> {
    let template = TemplateRegistry::get(template_id)?;
    Ok(TemplateMetadata {
        template_id: template.id(),
        valid_profiles: template.valid_profiles(),
        valid_encode_modes: template.valid_encode_modes(),
        param_schemas: template.param_schemas(),
        constraints: template.constraints(),
        bitrate_sets: template.bitrate_sets(),
        bitrate_hard_max: template.bitrate_hard_max(),
    })
}

pub fn param_schemas(template_id: &str) -> Result<&'static [ParamSchema]> {
    Ok(template_metadata(template_id)?.param_schemas)
}

pub fn constraints(template_id: &str) -> Result<&'static [Constraint]> {
    Ok(template_metadata(template_id)?.constraints)
}

pub fn find_param_schema(template_id: &str, key: &str) -> Result<Option<&'static ParamSchema>> {
    Ok(param_schemas(template_id)?
        .iter()
        .find(|schema| schema.key == key))
}

#[derive(Debug, Clone, Copy)]
pub struct TemplateMetadata {
    pub template_id: &'static str,
    pub valid_profiles: &'static [&'static str],
    pub valid_encode_modes: &'static [&'static str],
    pub param_schemas: &'static [ParamSchema],
    pub constraints: &'static [Constraint],
    pub bitrate_sets: &'static BTreeMap<&'static str, &'static [u16]>,
    pub bitrate_hard_max: u16,
}

pub(crate) fn normalize_drive(drive: char) -> Result<char> {
    if !drive.is_ascii_alphabetic() {
        bail!("win_drive must be an ASCII drive letter, got '{drive}'");
    }
    Ok(drive.to_ascii_uppercase())
}

pub(crate) fn normalize_windows_path(path: &str, drive: char) -> String {
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

#[cfg(test)]
mod tests {
    use super::{
        Ac4OutputMode, DEFAULT_TEMPLATE_ID, OutputContainer, OutputSpec, find_param_schema,
        normalize_drive, normalize_windows_path, parse_job_str, read_job, template_metadata,
    };
    use std::fs;

    #[test]
    fn normalizes_drive_letter() {
        assert_eq!(normalize_drive('y').unwrap(), 'Y');
        assert!(normalize_drive('1').is_err());
    }

    #[test]
    fn normalizes_windows_paths() {
        assert_eq!(normalize_windows_path("/tmp/in", 'Y'), "Y:/tmp/in");
        assert_eq!(normalize_windows_path("tmp/in", 'Y'), "Y:/tmp/in");
        assert_eq!(normalize_windows_path("z:/tmp/in", 'Y'), "Z:/tmp/in");
    }

    #[test]
    fn parses_yaml_and_json_strings() {
        let yaml = r#"
template_id: atmos_ec3_v1
output:
  storage_path: ./output
  file_names: [output.ec3]
misc:
  temp_dir: ./tmp
"#;
        let json = r#"{
  "template_id": "atmos_ec3_v1",
  "output": {"storage_path": "./output", "file_names": ["output.ec3"]},
  "misc": {"temp_dir": "./tmp"}
}"#;

        assert_eq!(
            parse_job_str(yaml).unwrap().template_id.as_deref(),
            Some(DEFAULT_TEMPLATE_ID)
        );
        assert_eq!(
            parse_job_str(json).unwrap().template_id.as_deref(),
            Some(DEFAULT_TEMPLATE_ID)
        );
    }

    #[test]
    fn read_job_prefers_extension_hint() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("job.json");
        fs::write(
            &path,
            r#"{"template_id":"atmos_ec3_v1","output":{"storage_path":"./out","file_names":["out.ec3"]},"misc":{"temp_dir":"./tmp"}}"#,
        )
        .unwrap();

        assert_eq!(
            read_job(&path).unwrap().template_id.as_deref(),
            Some(DEFAULT_TEMPLATE_ID)
        );
    }

    #[test]
    fn exposes_template_metadata_without_template_module() {
        let metadata = template_metadata("atmos_ec3_v1").unwrap();
        assert_eq!(metadata.template_id, "atmos_ec3_v1");
        assert!(metadata.valid_encode_modes.contains(&"streaming"));
        assert!(
            find_param_schema("atmos_ec3_v1", "data_rate")
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn output_spec_defaults_container_to_ac4() {
        let spec: OutputSpec = serde_yaml::from_str(
            r#"
storage_path: ./output
file_name: demo.ac4
"#,
        )
        .unwrap();

        assert_eq!(spec.container, OutputContainer::Ac4);
        assert_eq!(spec.ac4_output_mode, Ac4OutputMode::Single);
        assert_eq!(spec.file_names, vec!["demo.ac4"]);
    }

    #[test]
    fn output_spec_parses_ac4_output_mode_multi3() {
        let spec: OutputSpec = serde_yaml::from_str(
            r#"
storage_path: ./output
file_names: [a.ac4, b.ac4, c.ac4]
ac4_output_mode: multi3
"#,
        )
        .unwrap();

        assert_eq!(spec.ac4_output_mode, Ac4OutputMode::Multi3);
    }
}
