use anyhow::{Result, bail};

use crate::{
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::Value,
    spec::{EncodeMode, FilterOverrides, JobMode, Profile},
    template::Template,
};

pub mod defaults;
pub mod filter;
pub mod params;
pub mod xml;

pub use filter::Ac4V1Filter;

pub const AC4_V1: Ac4V1 = Ac4V1;

pub struct Ac4V1;

impl Template for Ac4V1 {
    fn id(&self) -> &'static str {
        "ac4_v1"
    }

    fn param_schemas(&self) -> &'static [crate::schema::ParamSchema] {
        params::PARAM_SCHEMAS
    }

    fn constraints(&self) -> &'static [crate::schema::Constraint] {
        &[]
    }

    fn bitrate_sets(&self) -> &'static std::collections::BTreeMap<&'static str, &'static [u16]> {
        params::bitrate_sets()
    }

    fn bitrate_hard_max(&self) -> u16 {
        params::BITRATE_HARD_MAX
    }

    fn valid_profiles(&self) -> &'static [&'static str] {
        params::VALID_PROFILES
    }

    fn valid_encode_modes(&self) -> &'static [&'static str] {
        params::VALID_ENCODE_MODES
    }

    fn defaults(&self, profile: Profile, encode_mode: EncodeMode) -> ResolvedFilter {
        ResolvedFilter::Ac4V1(defaults::defaults(profile, encode_mode))
    }

    fn validate_io(
        &self,
        job_mode: JobMode,
        input_names: &[String],
        output_names: &[String],
    ) -> Result<()> {
        if !matches!(job_mode, JobMode::Single) {
            bail!("template_id 'ac4_v1' only supports job_mode=single");
        }

        Self::validate_single_io(input_names, output_names)
    }

    fn apply_overrides(
        &self,
        _filter: &mut ResolvedFilter,
        overrides: &FilterOverrides,
        _encode_mode: EncodeMode,
    ) -> Result<()> {
        if overrides.is_empty() {
            Ok(())
        } else {
            bail!("template_id 'ac4_v1' does not expose filter overrides yet")
        }
    }

    fn constraint_value(&self, _filter: &ResolvedFilter, _key: &str) -> Option<Value> {
        None
    }

    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode {
        xml::xml_structure(job)
    }
}

impl Ac4V1 {
    fn validate_single_io(input_names: &[String], output_names: &[String]) -> Result<()> {
        if input_names.is_empty() || output_names.is_empty() {
            bail!("input.file_names and output.file_names must not be empty");
        }

        if input_names.len() != 1 || output_names.len() != 1 {
            bail!("job_mode=single requires exactly one input and one output file name");
        }

        Ok(())
    }
}
