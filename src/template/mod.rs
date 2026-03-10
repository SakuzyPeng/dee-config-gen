use std::collections::BTreeMap;

use anyhow::{Result, bail};

use crate::{
    config::{EncodeMode, FilterOverrides, Profile},
    media::InputMediaInfo,
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::{Constraint, ParamSchema, Value},
};

pub mod atmos_ec3_v1;
pub mod pcm_ddp_v1;

pub trait Template: Send + Sync {
    fn id(&self) -> &'static str;
    fn param_schemas(&self) -> &'static [ParamSchema];
    fn constraints(&self) -> &'static [Constraint];
    fn bitrate_sets(&self) -> &'static BTreeMap<&'static str, &'static [u16]>;
    fn bitrate_hard_max(&self) -> u16;
    fn valid_profiles(&self) -> &'static [&'static str];
    fn valid_encode_modes(&self) -> &'static [&'static str];
    fn defaults(&self, profile: Profile, encode_mode: EncodeMode) -> ResolvedFilter;
    fn requires_input_media(&self) -> bool {
        false
    }
    fn apply_overrides(
        &self,
        filter: &mut ResolvedFilter,
        overrides: &FilterOverrides,
        encode_mode: EncodeMode,
    ) -> Result<()>;
    fn constraint_value(&self, filter: &ResolvedFilter, key: &str) -> Option<Value>;
    fn validate_runtime_compatibility(
        &self,
        filter: &ResolvedFilter,
        encode_mode: EncodeMode,
        input_media: &[InputMediaInfo],
    ) -> Result<()> {
        let _ = (filter, encode_mode, input_media);
        Ok(())
    }
    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode;
}

pub struct TemplateRegistry;

impl TemplateRegistry {
    pub fn get(template_id: &str) -> Result<&'static dyn Template> {
        if template_id == atmos_ec3_v1::ATMOS_EC3_V1.id() {
            Ok(&atmos_ec3_v1::ATMOS_EC3_V1)
        } else if template_id == pcm_ddp_v1::PCM_DDP_V1.id() {
            Ok(&pcm_ddp_v1::PCM_DDP_V1)
        } else {
            bail!(
                "unsupported template_id '{template_id}'; supported templates: 'atmos_ec3_v1', 'pcm_ddp_v1'"
            )
        }
    }
}
