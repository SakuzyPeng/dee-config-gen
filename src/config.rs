use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Deserializer};

pub const DEFAULT_TEMPLATE_ID: &str = "atmos_ec3_v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobFile {
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
    pub output: IoSpec,
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

pub fn normalize_drive(drive: char) -> Result<char> {
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

#[cfg(test)]
mod tests {
    use super::{normalize_drive, normalize_windows_path};

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
}
