use anyhow::{Result, bail};

use crate::{
    config::{EncodeMode, FilterOverrides, Profile},
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::{
        ParamSchema, Value,
        validate::{ParamValue, ValidationContext, validate_mode_availability, validate_value},
    },
    template::Template,
};

pub mod constraints;
pub mod defaults;
pub mod filter;
pub mod params;
pub mod xml;

pub use filter::PcmDdpV1Filter;

pub const PCM_DDP_V1: PcmDdpV1 = PcmDdpV1;

pub struct PcmDdpV1;

impl Template for PcmDdpV1 {
    fn id(&self) -> &'static str {
        "pcm_ddp_v1"
    }

    fn param_schemas(&self) -> &'static [ParamSchema] {
        params::PARAM_SCHEMAS
    }

    fn constraints(&self) -> &'static [crate::schema::Constraint] {
        constraints::CONSTRAINTS
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
        ResolvedFilter::PcmDdpV1(defaults::defaults(profile, encode_mode))
    }

    fn apply_overrides(
        &self,
        filter: &mut ResolvedFilter,
        overrides: &FilterOverrides,
        encode_mode: EncodeMode,
    ) -> Result<()> {
        reject_unsupported_overrides(overrides)?;

        let filter = as_filter_mut(filter)?;
        let ctx = ValidationContext {
            encode_mode: encode_mode.as_str(),
            bitrate_sets: self.bitrate_sets(),
            bitrate_hard_max: self.bitrate_hard_max(),
        };

        if let Some(v) = &overrides.metering_mode {
            filter.metering_mode = validate_string_param("metering_mode", v, &ctx)?;
        }
        if let Some(v) = overrides.dialogue_intelligence {
            filter.dialogue_intelligence = validate_bool_param("dialogue_intelligence", v, &ctx)?;
        }
        if let Some(v) = overrides.speech_threshold {
            let validated = validate_int_param("speech_threshold", i64::from(v), &ctx)?;
            filter.speech_threshold = u8::try_from(validated).map_err(|_| {
                anyhow::anyhow!("invalid value '{}' for speech_threshold", validated)
            })?;
        }
        if let Some(v) = overrides.data_rate {
            let validated = validate_int_param("data_rate", i64::from(v), &ctx)?;
            filter.data_rate = u16::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{}' for data_rate", validated))?;
        }
        if let Some(v) = &overrides.timecode_frame_rate {
            filter.timecode_frame_rate = validate_string_param("timecode_frame_rate", v, &ctx)?;
        }
        if let Some(v) = &overrides.start {
            filter.start = validate_string_param("start", v, &ctx)?;
        }
        if let Some(v) = &overrides.end {
            filter.end = validate_string_param("end", v, &ctx)?;
        }
        if let Some(v) = &overrides.time_base {
            filter.time_base = validate_string_param("time_base", v, &ctx)?;
        }
        if let Some(v) = &overrides.prepend_silence_duration {
            filter.prepend_silence_duration =
                validate_string_param("prepend_silence_duration", v, &ctx)?;
        }
        if let Some(v) = &overrides.append_silence_duration {
            filter.append_silence_duration =
                validate_string_param("append_silence_duration", v, &ctx)?;
        }
        if let Some(v) = &overrides.line_mode_drc_profile {
            filter.line_mode_drc_profile = validate_string_param("line_mode_drc_profile", v, &ctx)?;
        }
        if let Some(v) = &overrides.rf_mode_drc_profile {
            filter.rf_mode_drc_profile = validate_string_param("rf_mode_drc_profile", v, &ctx)?;
        }
        if let Some(v) = &overrides.loro_center_mix_level {
            filter.loro_center_mix_level = validate_string_param("loro_center_mix_level", v, &ctx)?;
        }
        if let Some(v) = &overrides.loro_surround_mix_level {
            filter.loro_surround_mix_level =
                validate_string_param("loro_surround_mix_level", v, &ctx)?;
        }
        if let Some(v) = &overrides.ltrt_center_mix_level {
            filter.ltrt_center_mix_level = validate_string_param("ltrt_center_mix_level", v, &ctx)?;
        }
        if let Some(v) = &overrides.ltrt_surround_mix_level {
            filter.ltrt_surround_mix_level =
                validate_string_param("ltrt_surround_mix_level", v, &ctx)?;
        }
        if let Some(v) = &overrides.preferred_downmix_mode {
            filter.preferred_downmix_mode =
                validate_string_param("preferred_downmix_mode", v, &ctx)?;
        }
        if let Some(v) = overrides.custom_dialnorm {
            let validated = validate_int_param("custom_dialnorm", i64::from(v), &ctx)?;
            filter.custom_dialnorm = i8::try_from(validated).map_err(|_| {
                anyhow::anyhow!("invalid value '{}' for custom_dialnorm", validated)
            })?;
        }
        if let Some(v) = &overrides.encoder_mode {
            filter.encoder_mode = validate_string_param("encoder_mode", v, &ctx)?;
        }

        let validated = validate_int_param("data_rate", i64::from(filter.data_rate), &ctx)?;
        filter.data_rate = u16::try_from(validated)
            .map_err(|_| anyhow::anyhow!("invalid value '{}' for data_rate", validated))?;

        Ok(())
    }

    fn constraint_value(&self, filter: &ResolvedFilter, key: &str) -> Option<Value> {
        let filter = as_filter(filter);
        match key {
            "metering_mode" => Some(Value::Str(filter.metering_mode.clone())),
            "dialogue_intelligence" => Some(Value::Bool(filter.dialogue_intelligence)),
            "speech_threshold" => Some(Value::Int(i64::from(filter.speech_threshold))),
            "data_rate" => Some(Value::Int(i64::from(filter.data_rate))),
            "timecode_frame_rate" => Some(Value::Str(filter.timecode_frame_rate.clone())),
            "start" => Some(Value::Str(filter.start.clone())),
            "end" => Some(Value::Str(filter.end.clone())),
            "time_base" => Some(Value::Str(filter.time_base.clone())),
            "prepend_silence_duration" => Some(Value::Str(filter.prepend_silence_duration.clone())),
            "append_silence_duration" => Some(Value::Str(filter.append_silence_duration.clone())),
            "line_mode_drc_profile" => Some(Value::Str(filter.line_mode_drc_profile.clone())),
            "rf_mode_drc_profile" => Some(Value::Str(filter.rf_mode_drc_profile.clone())),
            "loro_center_mix_level" => Some(Value::Str(filter.loro_center_mix_level.clone())),
            "loro_surround_mix_level" => Some(Value::Str(filter.loro_surround_mix_level.clone())),
            "ltrt_center_mix_level" => Some(Value::Str(filter.ltrt_center_mix_level.clone())),
            "ltrt_surround_mix_level" => Some(Value::Str(filter.ltrt_surround_mix_level.clone())),
            "preferred_downmix_mode" => Some(Value::Str(filter.preferred_downmix_mode.clone())),
            "custom_dialnorm" => Some(Value::Int(i64::from(filter.custom_dialnorm))),
            "encoder_mode" => Some(Value::Str(filter.encoder_mode.clone())),
            _ => None,
        }
    }

    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode {
        xml::xml_structure(job)
    }
}

fn as_filter(filter: &ResolvedFilter) -> &PcmDdpV1Filter {
    match filter {
        ResolvedFilter::PcmDdpV1(value) => value,
        _ => panic!("pcm_ddp_v1 received wrong ResolvedFilter variant"),
    }
}

fn as_filter_mut(filter: &mut ResolvedFilter) -> Result<&mut PcmDdpV1Filter> {
    match filter {
        ResolvedFilter::PcmDdpV1(value) => Ok(value),
        _ => bail!("pcm_ddp_v1 received wrong ResolvedFilter variant"),
    }
}

fn validate_string_param(key: &str, value: &str, ctx: &ValidationContext<'_>) -> Result<String> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Str(value.to_string()), ctx)? {
        ParamValue::Str(v) => Ok(v),
        _ => bail!("parameter {key} returned non-string value"),
    }
}

fn validate_int_param(key: &str, value: i64, ctx: &ValidationContext<'_>) -> Result<i64> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Int(value), ctx)? {
        ParamValue::Int(v) => Ok(v),
        _ => bail!("parameter {key} returned non-integer value"),
    }
}

fn validate_bool_param(key: &str, value: bool, ctx: &ValidationContext<'_>) -> Result<bool> {
    let schema = find_schema_required(key)?;
    validate_mode_availability(schema.key, schema.mode_availability, ctx.encode_mode)?;

    match validate_value(key, schema.rule, ParamValue::Bool(value), ctx)? {
        ParamValue::Bool(v) => Ok(v),
        _ => bail!("parameter {key} returned non-bool value"),
    }
}

fn find_schema_required(key: &str) -> Result<&'static ParamSchema> {
    params::find_schema(key).ok_or_else(|| anyhow::anyhow!("unknown parameter: {key}"))
}

fn reject_unsupported_overrides(overrides: &FilterOverrides) -> Result<()> {
    let unsupported = [
        ("surround_trim_5_1", overrides.surround_trim_5_1.is_some()),
        ("surround_trim_7_1", overrides.surround_trim_7_1.is_some()),
        ("height_trim_5_1", overrides.height_trim_5_1.is_some()),
        ("encoding_backend", overrides.encoding_backend.is_some()),
    ];

    if let Some((field, _)) = unsupported.into_iter().find(|(_, present)| *present) {
        bail!(
            "parameter '{}' is not supported by template_id 'pcm_ddp_v1'",
            field
        );
    }

    Ok(())
}
