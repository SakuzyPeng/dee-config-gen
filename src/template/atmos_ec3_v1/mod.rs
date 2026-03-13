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

pub use filter::AtmosEc3V1Filter;

pub const ATMOS_EC3_V1: AtmosEc3V1 = AtmosEc3V1;

pub struct AtmosEc3V1;

impl Template for AtmosEc3V1 {
    fn id(&self) -> &'static str {
        "atmos_ec3_v1"
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
        ResolvedFilter::AtmosEc3V1(defaults::defaults(profile, encode_mode))
    }

    fn apply_overrides(
        &self,
        filter: &mut ResolvedFilter,
        overrides: &FilterOverrides,
        encode_mode: EncodeMode,
    ) -> Result<()> {
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
            filter.speech_threshold = u8::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for speech_threshold"))?;
        }
        if let Some(v) = overrides.data_rate {
            let validated = validate_int_param("data_rate", i64::from(v), &ctx)?;
            filter.data_rate = u16::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for data_rate"))?;
        }
        if let Some(v) = &overrides.timecode_frame_rate {
            filter.timecode_frame_rate = validate_string_param("timecode_frame_rate", v, &ctx)?;
        }
        if let Some(v) = &overrides.start {
            let value = validate_string_param("start", v, &ctx)?;
            filter.start = validate_atmos_boundary_timecode("start", &value)?;
        }
        if let Some(v) = &overrides.end {
            let value = validate_string_param("end", v, &ctx)?;
            filter.end = validate_atmos_boundary_timecode("end", &value)?;
        }
        if let Some(v) = &overrides.time_base {
            filter.time_base = validate_string_param("time_base", v, &ctx)?;
        }
        if let Some(v) = &overrides.prepend_silence_duration {
            let value = validate_string_param("prepend_silence_duration", v, &ctx)?;
            filter.prepend_silence_duration = validate_atmos_silence_duration(
                "prepend_silence_duration",
                &value,
                &filter.timecode_frame_rate,
                &filter.time_base,
            )?;
        }
        if let Some(v) = &overrides.append_silence_duration {
            let value = validate_string_param("append_silence_duration", v, &ctx)?;
            filter.append_silence_duration = validate_atmos_silence_duration(
                "append_silence_duration",
                &value,
                &filter.timecode_frame_rate,
                &filter.time_base,
            )?;
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
        if let Some(v) = &overrides.surround_trim_5_1 {
            filter.surround_trim_5_1 = validate_string_param("surround_trim_5_1", v, &ctx)?;
        }
        if let Some(v) = &overrides.height_trim_5_1 {
            filter.height_trim_5_1 = validate_string_param("height_trim_5_1", v, &ctx)?;
        }
        if let Some(v) = overrides.custom_dialnorm {
            let validated = validate_int_param("custom_dialnorm", i64::from(v), &ctx)?;
            filter.custom_dialnorm = i8::try_from(validated)
                .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for custom_dialnorm"))?;
        }
        if let Some(v) = &overrides.encoding_backend {
            filter.encoding_backend = Some(validate_string_param("encoding_backend", v, &ctx)?);
        }
        if let Some(v) = &overrides.encoder_mode {
            filter.encoder_mode = Some(validate_string_param("encoder_mode", v, &ctx)?);
        }

        let validated = validate_int_param("data_rate", i64::from(filter.data_rate), &ctx)?;
        filter.data_rate = u16::try_from(validated)
            .map_err(|_| anyhow::anyhow!("invalid value '{validated}' for data_rate"))?;

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
            "surround_trim_5_1" => Some(Value::Str(filter.surround_trim_5_1.clone())),
            "height_trim_5_1" => Some(Value::Str(filter.height_trim_5_1.clone())),
            "custom_dialnorm" => Some(Value::Int(i64::from(filter.custom_dialnorm))),
            "encoding_backend" => filter.encoding_backend.clone().map(Value::Str),
            "encoder_mode" => filter.encoder_mode.clone().map(Value::Str),
            _ => None,
        }
    }

    fn xml_structure(&self, job: &ResolvedJob) -> XmlNode {
        xml::xml_structure(job)
    }

    fn validate_runtime_compatibility(
        &self,
        filter: &ResolvedFilter,
        encode_mode: EncodeMode,
        _input_media: &[crate::media::InputMediaInfo],
        _input_file_names: &[String],
    ) -> Result<()> {
        let filter = as_filter(filter);
        if matches!(encode_mode, EncodeMode::Bluray) && filter.preferred_downmix_mode == "ltrt-pl2"
        {
            bail!("Preferred Downmix mode Pro Logic II is not supported in Blu-ray Mode");
        }

        Ok(())
    }
}

fn as_filter(filter: &ResolvedFilter) -> &AtmosEc3V1Filter {
    match filter {
        ResolvedFilter::AtmosEc3V1(value) => value,
        ResolvedFilter::PcmDdpV1(_) => panic!("atmos_ec3_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdV1(_) => panic!("atmos_ec3_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdWavV1(_) => {
            panic!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdWavListV1(_) => {
            panic!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavV1(_) => {
            panic!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavListV1(_) => {
            panic!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
    }
}

fn as_filter_mut(filter: &mut ResolvedFilter) -> Result<&mut AtmosEc3V1Filter> {
    match filter {
        ResolvedFilter::AtmosEc3V1(value) => Ok(value),
        ResolvedFilter::PcmDdpV1(_) => bail!("atmos_ec3_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdV1(_) => bail!("atmos_ec3_v1 received wrong ResolvedFilter variant"),
        ResolvedFilter::ThdWavV1(_) => {
            bail!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdWavListV1(_) => {
            bail!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavV1(_) => {
            bail!("atmos_ec3_v1 received wrong ResolvedFilter variant")
        }
        ResolvedFilter::ThdAtmosWavListV1(_) => {
            bail!("atmos_ec3_v1 received wrong ResolvedFilter variant")
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

fn validate_atmos_boundary_timecode(key: &str, value: &str) -> Result<String> {
    let special = match key {
        "start" => "first_frame_of_action",
        "end" => "end_of_file",
        other => bail!("unsupported boundary timecode key '{other}'"),
    };

    if value == special || is_valid_atmos_timecode(value) {
        Ok(value.to_string())
    } else {
        bail!(
            "invalid value '{value}' for {key}; expected {special}, HH:MM:SS:FF[df], or HH:MM:SS.xx"
        )
    }
}

fn validate_atmos_silence_duration(
    key: &str,
    value: &str,
    timecode_frame_rate: &str,
    time_base: &str,
) -> Result<String> {
    if is_valid_decimal_duration(value) {
        return Ok(value.to_string());
    }

    if is_valid_frame_duration(value) {
        if timecode_frame_rate != "not_indicated" || time_base == "embedded_timecode" {
            return Ok(value.to_string());
        }

        bail!(
            "invalid value '{value}' for {key}; frame syntax requires timecode_frame_rate or time_base=embedded_timecode"
        );
    }

    bail!("invalid value '{value}' for {key}; expected seconds.milliseconds or xf")
}

fn is_valid_atmos_timecode(value: &str) -> bool {
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

fn is_valid_frame_duration(value: &str) -> bool {
    let Some(frames) = value.strip_suffix('f') else {
        return false;
    };
    !frames.is_empty() && frames.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use crate::{
        config::EncodeMode,
        resolve::{ResolveOptions, resolve_job},
        test_support::sample_job_file,
    };

    use super::{
        is_valid_atmos_timecode, validate_atmos_boundary_timecode, validate_atmos_silence_duration,
    };

    #[test]
    fn accepts_documented_atmos_start_end_formats() {
        assert_eq!(
            validate_atmos_boundary_timecode("start", "first_frame_of_action").unwrap(),
            "first_frame_of_action"
        );
        assert_eq!(
            validate_atmos_boundary_timecode("start", "00:00:00.0").unwrap(),
            "00:00:00.0"
        );
        assert_eq!(
            validate_atmos_boundary_timecode("end", "00:00:00:00df").unwrap(),
            "00:00:00:00df"
        );
    }

    #[test]
    fn rejects_undocumented_atmos_start_end_formats() {
        let err = validate_atmos_boundary_timecode("start", "0:00:00.0")
            .unwrap_err()
            .to_string();
        assert!(err.contains("HH:MM:SS:FF[df], or HH:MM:SS.xx"));

        let err = validate_atmos_boundary_timecode("end", "00:00")
            .unwrap_err()
            .to_string();
        assert!(err.contains("HH:MM:SS:FF[df], or HH:MM:SS.xx"));
    }

    #[test]
    fn validates_atmos_silence_duration_formats() {
        assert_eq!(
            validate_atmos_silence_duration(
                "prepend_silence_duration",
                "0.005333",
                "not_indicated",
                "file_position"
            )
            .unwrap(),
            "0.005333"
        );
        assert_eq!(
            validate_atmos_silence_duration(
                "append_silence_duration",
                "1f",
                "23.976",
                "file_position"
            )
            .unwrap(),
            "1f"
        );
        assert_eq!(
            validate_atmos_silence_duration(
                "append_silence_duration",
                "1f",
                "not_indicated",
                "embedded_timecode"
            )
            .unwrap(),
            "1f"
        );
    }

    #[test]
    fn rejects_atmos_invalid_silence_duration_formats() {
        let err = validate_atmos_silence_duration(
            "prepend_silence_duration",
            "0:00:00.005333",
            "not_indicated",
            "file_position",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("seconds.milliseconds or xf"));

        let err = validate_atmos_silence_duration(
            "append_silence_duration",
            "1f",
            "not_indicated",
            "file_position",
        )
        .unwrap_err()
        .to_string();
        assert!(err.contains("frame syntax requires"));
    }

    #[test]
    fn resolves_atmos_bluray_frame_silence_with_embedded_time_base() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Bluray;
        job.filter.time_base = Some("embedded_timecode".to_string());
        job.filter.prepend_silence_duration = Some("1f".to_string());

        resolve_job(
            job,
            &ResolveOptions {
                template_override: None,
                allow_fixed_override: false,
                windows_drive: 'Y',
            },
        )
        .expect("bluray embedded frame silence should resolve");
    }

    #[test]
    fn rejects_atmos_streaming_frame_silence_without_timecode_context() {
        let mut job = sample_job_file();
        job.encode_mode = EncodeMode::Streaming;
        job.filter.prepend_silence_duration = Some("1f".to_string());

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

        assert!(err.contains("frame syntax requires"));
    }

    #[test]
    fn timecode_helper_accepts_drop_frame_and_decimal_notation() {
        assert!(is_valid_atmos_timecode("00:00:00:00df"));
        assert!(is_valid_atmos_timecode("00:00:00.0"));
        assert!(!is_valid_atmos_timecode("0:00:00.0"));
    }
}
