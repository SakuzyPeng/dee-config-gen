use anyhow::{Result, bail};

use crate::{
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::Value,
    spec::{EncodeMode, FilterOverrides, Profile},
    template::{Template, ac4_ims_shared},
};

pub mod defaults;
pub mod filter;
pub mod json;
pub mod params;
pub mod xml;

pub use filter::Ac4ImsAtmosV1Filter;

pub const AC4_IMS_ATMOS_V1: Ac4ImsAtmosV1 = Ac4ImsAtmosV1;

pub struct Ac4ImsAtmosV1;

impl Template for Ac4ImsAtmosV1 {
    fn id(&self) -> &'static str {
        "ac4_ims_atmos_v1"
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
        ResolvedFilter::Ac4ImsAtmosV1(defaults::defaults(profile, encode_mode))
    }

    fn apply_overrides(
        &self,
        filter: &mut ResolvedFilter,
        overrides: &FilterOverrides,
        encode_mode: EncodeMode,
    ) -> Result<()> {
        let filter = as_filter_mut(filter)?;
        ac4_ims_shared::apply_overrides(filter, overrides, encode_mode)
    }

    fn constraint_value(&self, filter: &ResolvedFilter, key: &str) -> Option<Value> {
        ac4_ims_shared::constraint_value(as_filter(filter), key)
    }

    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode {
        xml::xml_structure(job)
    }

    fn json_structure(&self, job: &ResolvedJob) -> Result<serde_json::Value> {
        Ok(json::json_structure(job))
    }
}

fn as_filter(filter: &ResolvedFilter) -> &Ac4ImsAtmosV1Filter {
    match filter {
        ResolvedFilter::Ac4ImsAtmosV1(value) => value,
        _ => panic!("ac4_ims_atmos_v1 received wrong ResolvedFilter variant"),
    }
}

fn as_filter_mut(filter: &mut ResolvedFilter) -> Result<&mut Ac4ImsAtmosV1Filter> {
    match filter {
        ResolvedFilter::Ac4ImsAtmosV1(value) => Ok(value),
        _ => bail!("ac4_ims_atmos_v1 received wrong ResolvedFilter variant"),
    }
}
