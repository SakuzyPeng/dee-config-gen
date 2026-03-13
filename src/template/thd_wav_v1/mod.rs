use anyhow::{Result, bail};

use crate::{
    media::InputMediaInfo,
    render::XmlNode,
    resolve::{ResolvedFilter, ResolvedJob},
    schema::{
        ParamSchema, Value,
        validate::{ParamValue, ValidationContext, validate_mode_availability, validate_value},
    },
    spec::{EncodeMode, FilterOverrides, Profile},
    template::Template,
};

pub mod constraints;
pub mod defaults;
pub mod filter;
pub mod json;
pub mod params;
pub mod xml;

pub use filter::ThdWavV1Filter;

pub const THD_WAV_V1: ThdWavV1 = ThdWavV1;

pub struct ThdWavV1;

impl Template for ThdWavV1 {
    fn id(&self) -> &'static str {
        "thd_wav_v1"
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
        ResolvedFilter::ThdWavV1(defaults::defaults(profile, encode_mode))
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

        if let Some(v) = &overrides.input_timecode_frame_rate {
            filter.input_timecode_frame_rate =
                validate_string_param("input_timecode_frame_rate", v, &ctx)?;
        }
        if let Some(v) = &overrides.offset {
            filter.offset = validate_string_param("offset", v, &ctx)?;
        }
        if let Some(v) = &overrides.ffoa {
            filter.ffoa = validate_string_param("ffoa", v, &ctx)?;
        }
        if let Some(v) = &overrides.metering_mode {
            filter.metering_mode = validate_string_param("metering_mode", v, &ctx)?;
        }
        if let Some(v) = overrides.dialogue_intelligence {
            filter.dialogue_intelligence = validate_bool_param("dialogue_intelligence", v, &ctx)?;
        }
        if let Some(v) = overrides.speech_threshold {
            let validated = validate_int_param("speech_threshold", i64::from(v), &ctx)?;
            filter.speech_threshold = u8::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for speech_threshold"))?;
        }
        if let Some(v) = &overrides.timecode_frame_rate {
            filter.timecode_frame_rate = validate_string_param("timecode_frame_rate", v, &ctx)?;
        }
        if let Some(v) = &overrides.starting_timecode {
            let value = validate_string_param("starting_timecode", v, &ctx)?;
            filter.starting_timecode = validate_thd_starting_timecode(&value)?;
        }
        if let Some(v) = &overrides.frame_rate {
            filter.frame_rate = validate_string_param("frame_rate", v, &ctx)?;
        }
        if let Some(v) = &overrides.start {
            let value = validate_string_param("start", v, &ctx)?;
            filter.start = validate_thd_boundary_timecode("start", &value)?;
        }
        if let Some(v) = &overrides.end {
            let value = validate_string_param("end", v, &ctx)?;
            filter.end = validate_thd_boundary_timecode("end", &value)?;
        }
        if let Some(v) = &overrides.time_base {
            filter.time_base = validate_string_param("time_base", v, &ctx)?;
        }
        if let Some(v) = &overrides.prepend_silence_duration {
            let value = validate_string_param("prepend_silence_duration", v, &ctx)?;
            filter.prepend_silence_duration =
                validate_thd_silence_duration("prepend_silence_duration", &value)?;
        }
        if let Some(v) = &overrides.append_silence_duration {
            let value = validate_string_param("append_silence_duration", v, &ctx)?;
            filter.append_silence_duration =
                validate_thd_silence_duration("append_silence_duration", &value)?;
        }
        if let Some(v) = overrides.custom_dialnorm {
            let validated = validate_int_param("custom_dialnorm", i64::from(v), &ctx)?;
            filter.custom_dialnorm = i8::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for custom_dialnorm"))?;
        }
        if let Some(v) = &overrides.atmos_presentation_drc_profile {
            filter.atmos_presentation_drc_profile =
                validate_string_param("atmos_presentation_drc_profile", v, &ctx)?;
        }
        if let Some(v) = &overrides.spatial_clusters {
            filter.spatial_clusters = validate_string_param("spatial_clusters", v, &ctx)?;
        }
        if let Some(v) = overrides.legacy_authoring_compatibility {
            filter.legacy_authoring_compatibility =
                validate_bool_param("legacy_authoring_compatibility", v, &ctx)?;
        }
        if let Some(v) = &overrides.presentation_8ch_drc_profile {
            filter.presentation_8ch_drc_profile =
                validate_string_param("presentation_8ch_drc_profile", v, &ctx)?;
        }
        if let Some(v) = &overrides.presentation_6ch_drc_profile {
            filter.presentation_6ch_drc_profile =
                validate_string_param("presentation_6ch_drc_profile", v, &ctx)?;
        }
        if let Some(v) = &overrides.presentation_2ch_drc_profile {
            filter.presentation_2ch_drc_profile =
                validate_string_param("presentation_2ch_drc_profile", v, &ctx)?;
        }
        if let Some(v) = overrides.optimize_data_rate {
            filter.optimize_data_rate = validate_bool_param("optimize_data_rate", v, &ctx)?;
        }

        Ok(())
    }

    fn constraint_value(&self, filter: &ResolvedFilter, key: &str) -> Option<Value> {
        let filter = as_filter(filter);
        match key {
            "input_timecode_frame_rate" => {
                Some(Value::Str(filter.input_timecode_frame_rate.clone()))
            }
            "offset" => Some(Value::Str(filter.offset.clone())),
            "ffoa" => Some(Value::Str(filter.ffoa.clone())),
            "metering_mode" => Some(Value::Str(filter.metering_mode.clone())),
            "dialogue_intelligence" => Some(Value::Bool(filter.dialogue_intelligence)),
            "speech_threshold" => Some(Value::Int(i64::from(filter.speech_threshold))),
            "timecode_frame_rate" => Some(Value::Str(filter.timecode_frame_rate.clone())),
            "starting_timecode" => Some(Value::Str(filter.starting_timecode.clone())),
            "frame_rate" => Some(Value::Str(filter.frame_rate.clone())),
            "start" => Some(Value::Str(filter.start.clone())),
            "end" => Some(Value::Str(filter.end.clone())),
            "time_base" => Some(Value::Str(filter.time_base.clone())),
            "prepend_silence_duration" => Some(Value::Str(filter.prepend_silence_duration.clone())),
            "append_silence_duration" => Some(Value::Str(filter.append_silence_duration.clone())),
            "custom_dialnorm" => Some(Value::Int(i64::from(filter.custom_dialnorm))),
            "atmos_presentation_drc_profile" => {
                Some(Value::Str(filter.atmos_presentation_drc_profile.clone()))
            }
            "spatial_clusters" => Some(Value::Str(filter.spatial_clusters.clone())),
            "legacy_authoring_compatibility" => {
                Some(Value::Bool(filter.legacy_authoring_compatibility))
            }
            "presentation_8ch_drc_profile" => {
                Some(Value::Str(filter.presentation_8ch_drc_profile.clone()))
            }
            "presentation_6ch_drc_profile" => {
                Some(Value::Str(filter.presentation_6ch_drc_profile.clone()))
            }
            "presentation_2ch_drc_profile" => {
                Some(Value::Str(filter.presentation_2ch_drc_profile.clone()))
            }
            "optimize_data_rate" => Some(Value::Bool(filter.optimize_data_rate)),
            _ => None,
        }
    }

    fn validate_runtime_compatibility(
        &self,
        _filter: &ResolvedFilter,
        _encode_mode: EncodeMode,
        _input_media: &[InputMediaInfo],
        _input_file_names: &[String],
    ) -> Result<()> {
        Ok(())
    }

    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode {
        xml::xml_structure(job)
    }

    fn json_structure(&self, job: &ResolvedJob) -> Result<serde_json::Value> {
        Ok(json::json_structure(job))
    }
}

fn as_filter(filter: &ResolvedFilter) -> &ThdWavV1Filter {
    match filter {
        ResolvedFilter::ThdWavV1(value) => value,
        ResolvedFilter::AtmosEc3V1(_) => {
            panic!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::PcmDdpV1(_) => panic!("thd_wav_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdV1(_) => panic!("thd_wav_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdWavListV1(_) => {
            panic!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavV1(_) => {
            panic!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavListV1(_) => {
            panic!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
    }
}

fn as_filter_mut(filter: &mut ResolvedFilter) -> Result<&mut ThdWavV1Filter> {
    match filter {
        ResolvedFilter::ThdWavV1(value) => Ok(value),
        ResolvedFilter::AtmosEc3V1(_) => bail!("thd_wav_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::PcmDdpV1(_) => bail!("thd_wav_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdV1(_) => bail!("thd_wav_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdWavListV1(_) => {
            bail!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavV1(_) => {
            bail!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavListV1(_) => {
            bail!("thd_wav_v1 received wrong ResolvedFilter variant")
        }
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

pub(crate) fn reject_unsupported_overrides(overrides: &FilterOverrides) -> Result<()> {
    let unsupported = [
        ("data_rate", overrides.data_rate.is_some()),
        ("bitstream_mode", overrides.bitstream_mode.is_some()),
        ("downmix_config", overrides.downmix_config.is_some()),
        ("lfe_on", overrides.lfe_on.is_some()),
        (
            "dolby_surround_mode",
            overrides.dolby_surround_mode.is_some(),
        ),
        (
            "dolby_surround_ex_mode",
            overrides.dolby_surround_ex_mode.is_some(),
        ),
        ("user_data", overrides.user_data.is_some()),
        (
            "line_mode_drc_profile",
            overrides.line_mode_drc_profile.is_some(),
        ),
        (
            "rf_mode_drc_profile",
            overrides.rf_mode_drc_profile.is_some(),
        ),
        ("lfe_lowpass_filter", overrides.lfe_lowpass_filter.is_some()),
        (
            "surround_90_degree_phase_shift",
            overrides.surround_90_degree_phase_shift.is_some(),
        ),
        (
            "surround_3db_attenuation",
            overrides.surround_3db_attenuation.is_some(),
        ),
        (
            "loro_center_mix_level",
            overrides.loro_center_mix_level.is_some(),
        ),
        (
            "loro_surround_mix_level",
            overrides.loro_surround_mix_level.is_some(),
        ),
        (
            "ltrt_center_mix_level",
            overrides.ltrt_center_mix_level.is_some(),
        ),
        (
            "ltrt_surround_mix_level",
            overrides.ltrt_surround_mix_level.is_some(),
        ),
        (
            "preferred_downmix_mode",
            overrides.preferred_downmix_mode.is_some(),
        ),
        (
            "allow_hybrid_downmix",
            overrides.allow_hybrid_downmix.is_some(),
        ),
        ("surround_trim_5_1", overrides.surround_trim_5_1.is_some()),
        ("height_trim_5_1", overrides.height_trim_5_1.is_some()),
        ("encoding_backend", overrides.encoding_backend.is_some()),
        ("encoder_mode", overrides.encoder_mode.is_some()),
    ];

    if let Some((field, _)) = unsupported.into_iter().find(|(_, present)| *present) {
        bail!("parameter '{field}' is not supported by template_id 'thd_wav_v1'");
    }

    Ok(())
}

pub(crate) fn validate_thd_boundary_timecode(key: &str, value: &str) -> Result<String> {
    let special = match key {
        "start" => "first_frame_of_action",
        "end" => "end_of_file",
        other => bail!("unsupported boundary timecode key '{other}'"),
    };

    if value == special || is_valid_timecode(value) {
        Ok(value.to_string())
    } else {
        bail!(
            "invalid value '{value}' for {key}; expected {special}, HH:MM:SS:FF[df], or HH:MM:SS.xx"
        )
    }
}

pub(crate) fn validate_thd_silence_duration(key: &str, value: &str) -> Result<String> {
    if is_valid_decimal_duration(value) {
        Ok(value.to_string())
    } else {
        bail!("invalid value '{value}' for {key}; expected seconds.milliseconds")
    }
}

pub(crate) fn validate_thd_starting_timecode(value: &str) -> Result<String> {
    if matches!(value, "off" | "auto") || is_valid_timecode(value) {
        Ok(value.to_string())
    } else {
        bail!(
            "invalid starting_timecode '{value}': expected off, auto, HH:MM:SS:FF[df], or HH:MM:SS.xx"
        )
    }
}

fn is_valid_timecode(value: &str) -> bool {
    parse_hh_mm_ss_ff(value).is_some() || parse_hh_mm_ss_decimal(value)
}

fn parse_hh_mm_ss_ff(value: &str) -> Option<()> {
    let core = value.strip_suffix("df").unwrap_or(value);
    let mut parts = core.split(':');
    let hours = parts.next()?;
    let minutes = parts.next()?;
    let seconds = parts.next()?;
    let frames = parts.next()?;
    if parts.next().is_some() {
        return None;
    }

    if hours.len() != 2
        || !hours.chars().all(|c| c.is_ascii_digit())
        || minutes.len() != 2
        || !minutes.chars().all(|c| c.is_ascii_digit())
        || seconds.len() != 2
        || !seconds.chars().all(|c| c.is_ascii_digit())
        || frames.len() != 2
        || !frames.chars().all(|c| c.is_ascii_digit())
    {
        return None;
    }

    Some(())
}

fn parse_hh_mm_ss_decimal(value: &str) -> bool {
    let mut parts = value.split(':');
    let Some(hours) = parts.next() else {
        return false;
    };
    let Some(minutes) = parts.next() else {
        return false;
    };
    let Some(seconds_and_fraction) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }

    if hours.len() != 2
        || !hours.chars().all(|c| c.is_ascii_digit())
        || minutes.len() != 2
        || !minutes.chars().all(|c| c.is_ascii_digit())
    {
        return false;
    }

    let Some((seconds, fraction)) = seconds_and_fraction.split_once('.') else {
        return false;
    };
    !seconds.is_empty()
        && seconds.len() == 2
        && seconds.chars().all(|c| c.is_ascii_digit())
        && !fraction.is_empty()
        && fraction.chars().all(|c| c.is_ascii_digit())
}

fn is_valid_decimal_duration(value: &str) -> bool {
    let Some((seconds, fraction)) = value.split_once('.') else {
        return !value.is_empty() && value.chars().all(|c| c.is_ascii_digit());
    };

    !seconds.is_empty()
        && seconds.chars().all(|c| c.is_ascii_digit())
        && !fraction.is_empty()
        && fraction.chars().all(|c| c.is_ascii_digit())
}
