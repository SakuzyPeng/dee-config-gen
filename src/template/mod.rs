use std::collections::BTreeMap;

use anyhow::{Result, bail};
use serde_json::Value as JsonValue;

use crate::{
    media::InputMediaInfo,
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::{Constraint, ParamSchema, Value},
    spec::{EncodeMode, FilterOverrides, JobMode, Profile},
};

pub mod ac4_v1;
pub mod atmos_ec3_v1;
pub mod pcm_ddp_v1;
pub mod thd_atmos_wav_list_v1;
pub mod thd_atmos_wav_v1;
pub(crate) mod thd_json;
pub(crate) mod thd_mixed;
pub mod thd_v1;
pub mod thd_wav_list_v1;
pub mod thd_wav_v1;

pub trait Template: Send + Sync {
    fn id(&self) -> &'static str;
    fn param_schemas(&self) -> &'static [ParamSchema];
    fn constraints(&self) -> &'static [Constraint];
    fn bitrate_sets(&self) -> &'static BTreeMap<&'static str, &'static [u16]>;
    fn bitrate_hard_max(&self) -> u16;
    fn valid_profiles(&self) -> &'static [&'static str];
    fn valid_encode_modes(&self) -> &'static [&'static str];
    fn defaults(&self, profile: Profile, encode_mode: EncodeMode) -> ResolvedFilter;
    fn validate_io(
        &self,
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
    fn requires_input_media(&self) -> bool {
        false
    }
    fn input_media_file_names(&self, file_names: &[String]) -> Vec<String> {
        file_names.to_vec()
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
        input_file_names: &[String],
    ) -> Result<()> {
        let _ = (filter, encode_mode, input_media, input_file_names);
        Ok(())
    }
    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode;
    fn json_structure(&self, _job: &ResolvedJob) -> Result<JsonValue> {
        bail!("template '{}' does not support JSON output", self.id())
    }
}

pub struct TemplateRegistry;

impl TemplateRegistry {
    pub fn get(template_id: &str) -> Result<&'static dyn Template> {
        if template_id == atmos_ec3_v1::ATMOS_EC3_V1.id() {
            Ok(&atmos_ec3_v1::ATMOS_EC3_V1)
        } else if template_id == ac4_v1::AC4_V1.id() {
            Ok(&ac4_v1::AC4_V1)
        } else if template_id == pcm_ddp_v1::PCM_DDP_V1.id() {
            Ok(&pcm_ddp_v1::PCM_DDP_V1)
        } else if template_id == thd_v1::THD_V1.id() {
            Ok(&thd_v1::THD_V1)
        } else if template_id == thd_wav_v1::THD_WAV_V1.id() {
            Ok(&thd_wav_v1::THD_WAV_V1)
        } else if template_id == thd_wav_list_v1::THD_WAV_LIST_V1.id() {
            Ok(&thd_wav_list_v1::THD_WAV_LIST_V1)
        } else if template_id == thd_atmos_wav_v1::THD_ATMOS_WAV_V1.id() {
            Ok(&thd_atmos_wav_v1::THD_ATMOS_WAV_V1)
        } else if template_id == thd_atmos_wav_list_v1::THD_ATMOS_WAV_LIST_V1.id() {
            Ok(&thd_atmos_wav_list_v1::THD_ATMOS_WAV_LIST_V1)
        } else {
            bail!(
                "unsupported template_id '{template_id}'; supported templates: 'ac4_v1', 'atmos_ec3_v1', 'pcm_ddp_v1', 'thd_v1', 'thd_wav_v1', 'thd_wav_list_v1', 'thd_atmos_wav_v1', 'thd_atmos_wav_list_v1'"
            )
        }
    }
}
