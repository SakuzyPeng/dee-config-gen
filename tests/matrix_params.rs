mod common;

use dee_config_gen::{
    config::{JobFile, Profile},
    schema::{Constraint, FixedValue, ModeAvailability, ParamRule},
    template::atmos_ec3_v1::{constraints::CONSTRAINTS, params},
};

use common::{
    base_job_file, contract_path_label, load_xsd_contract, resolve_with_defaults, set_encode_mode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidateValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

#[test]
#[ignore = "matrix test: schema + xsd driven"]
fn matrix_params_from_xsd_and_schema() {
    let contract = load_xsd_contract();

    for schema in params::PARAM_SCHEMAS {
        let xsd_path = contract_path_label(&contract, schema.key, schema.sources);

        for mode in params::VALID_ENCODE_MODES {
            for (label, candidate) in valid_values_for_param(schema.key, schema.rule, mode) {
                let case_id = format!("valid:{}:{}:{}", schema.key, mode, label);
                let mut job = base_job_file();
                set_encode_mode(&mut job, mode);
                set_override(&mut job, schema.key, candidate.clone());

                assert_case(
                    &case_id,
                    &xsd_path,
                    schema.key,
                    expected_for_candidate(schema.key, schema.mode_availability, mode, &candidate),
                    resolve_with_defaults(job),
                );
            }

            for (label, invalid) in invalid_values_for_param(schema.key, schema.rule, mode) {
                let case_id = format!("invalid:{}:{}:{}", schema.key, mode, label);
                let mut job = base_job_file();
                set_encode_mode(&mut job, mode);
                set_override(&mut job, schema.key, invalid);

                let err = resolve_with_defaults(job)
                    .expect_err("invalid value should fail")
                    .to_string();

                assert!(
                    err.contains(schema.key),
                    "case_id={} xsd_path={} param_key={} expected key in error, got '{}'",
                    case_id,
                    xsd_path,
                    schema.key,
                    err,
                );
            }
        }
    }

    for constraint in CONSTRAINTS {
        if let Constraint::Required {
            param,
            when_mode,
            value,
        } = constraint
        {
            let xsd_path = params::find_schema(param)
                .map(|schema| contract_path_label(&contract, schema.key, schema.sources))
                .unwrap_or_else(|| "<unknown_param>".to_string());
            for (label, alternative) in required_alternative_values(param, *value, when_mode) {
                let mut job = base_job_file();
                set_encode_mode(&mut job, when_mode);
                set_override(&mut job, param, alternative);

                let err = resolve_with_defaults(job)
                    .expect_err("required constraint mismatch should fail")
                    .to_string();
                let expected = format!("{when_mode} mode requires {param}=");
                assert!(
                    err.contains(&expected),
                    "case_id=required_mismatch:{param}:{when_mode}:{label} xsd_path={xsd_path} param_key={param} expected '{expected}' in error, got '{err}'",
                );
            }
        }
    }

    for constraint in CONSTRAINTS {
        if let Constraint::FixedValues {
            when_profile,
            fields,
            ..
        } = constraint
        {
            if *when_profile != "music" || fields.is_empty() {
                continue;
            }
            for (field, expected) in *fields {
                let xsd_path = params::find_schema(field)
                    .map(|schema| contract_path_label(&contract, schema.key, schema.sources))
                    .unwrap_or_else(|| "<unknown_param>".to_string());

                let mut job = base_job_file();
                job.profile = Profile::Music;
                set_override(&mut job, field, conflicting_value_for_fixed(*expected));

                let err = resolve_with_defaults(job)
                    .expect_err("fixed value conflict should fail")
                    .to_string();
                assert!(
                    err.contains("profile=music locks fixed fields"),
                    "case_id=fixed_value_conflict:{field} xsd_path={xsd_path} param_key={field} expected profile lock error got '{err}'",
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExpectedOutcome {
    Ok,
    ErrContains(String),
}

fn assert_case(
    case_id: &str,
    xsd_path: &str,
    param_key: &str,
    expected: ExpectedOutcome,
    result: anyhow::Result<dee_config_gen::ResolvedJob>,
) {
    match expected {
        ExpectedOutcome::Ok => {
            if let Err(err) = result {
                panic!(
                    "case_id={case_id} xsd_path={xsd_path} param_key={param_key} expected success got '{err}'"
                );
            }
        }
        ExpectedOutcome::ErrContains(expected_message) => {
            let err = result.expect_err("case should fail").to_string();
            assert!(
                err.contains(&expected_message),
                "case_id={case_id} xsd_path={xsd_path} param_key={param_key} expected '{expected_message}' got '{err}'",
            );
        }
    }
}

fn mode_allowed(mode_availability: ModeAvailability, mode: &str) -> bool {
    match mode_availability {
        ModeAvailability::All => true,
        ModeAvailability::Only(modes) => modes.contains(&mode),
        ModeAvailability::Except(modes) => !modes.contains(&mode),
    }
}

fn forbidden_message(param: &str, mode: &str) -> Option<&'static str> {
    for constraint in CONSTRAINTS {
        if let Constraint::Forbidden {
            param: forbidden_param,
            when_mode,
            message,
        } = constraint
            && *forbidden_param == param
            && when_mode.contains(&mode)
        {
            return Some(*message);
        }
    }
    None
}

fn required_value(param: &str, mode: &str) -> Option<CandidateValue> {
    for constraint in CONSTRAINTS {
        if let Constraint::Required {
            param: required_param,
            when_mode,
            value,
        } = constraint
            && *required_param == param
            && *when_mode == mode
        {
            return Some(fixed_to_candidate(*value));
        }
    }
    None
}

fn expected_for_candidate(
    param: &str,
    mode_availability: ModeAvailability,
    mode: &str,
    candidate: &CandidateValue,
) -> ExpectedOutcome {
    if let Some(message) = forbidden_message(param, mode) {
        return ExpectedOutcome::ErrContains(message.to_string());
    }

    if !mode_allowed(mode_availability, mode) {
        return ExpectedOutcome::ErrContains("not available for encode_mode".to_string());
    }

    if let Some(required) = required_value(param, mode)
        && required != *candidate
    {
        return ExpectedOutcome::ErrContains(format!("{mode} mode requires {param}="));
    }

    ExpectedOutcome::Ok
}

fn required_alternative_values(
    param: &str,
    expected: FixedValue,
    mode: &str,
) -> Vec<(&'static str, CandidateValue)> {
    let Some(schema) = params::find_schema(param) else {
        return Vec::new();
    };
    match (schema.rule, expected) {
        (ParamRule::Enum(allowed), FixedValue::Str(expected_str)) => allowed
            .iter()
            .copied()
            .filter(|candidate| *candidate != expected_str)
            .map(|candidate| ("enum_alt", CandidateValue::Str(candidate.to_string())))
            .collect(),
        (ParamRule::Bool, FixedValue::Bool(v)) => vec![("bool_flip", CandidateValue::Bool(!v))],
        (ParamRule::IntRange { min, max }, FixedValue::Int(v)) => {
            [("below_expected", v - 1), ("above_expected", v + 1)]
                .into_iter()
                .filter(|(_, alt)| *alt >= min && *alt <= max && *alt != v)
                .map(|(label, alt)| (label, CandidateValue::Int(alt)))
                .collect()
        }
        (ParamRule::DataRate, FixedValue::Int(v)) => {
            let Some(allowed) = params::bitrate_sets().get(mode).copied() else {
                return Vec::new();
            };
            allowed
                .iter()
                .copied()
                .filter(|candidate| i64::from(*candidate) != v)
                .map(|candidate| ("bitrate_alt", CandidateValue::Int(i64::from(candidate))))
                .collect()
        }
        _ => Vec::new(),
    }
}

fn fixed_to_candidate(value: FixedValue) -> CandidateValue {
    match value {
        FixedValue::Bool(v) => CandidateValue::Bool(v),
        FixedValue::Int(v) => CandidateValue::Int(v),
        FixedValue::Str(v) => CandidateValue::Str(v.to_string()),
    }
}

fn conflicting_value_for_fixed(value: FixedValue) -> CandidateValue {
    match value {
        FixedValue::Bool(v) => CandidateValue::Bool(!v),
        FixedValue::Int(v) => CandidateValue::Int(v.saturating_sub(1)),
        FixedValue::Str(v) => {
            let alt = if v == "film_light" {
                "music_light"
            } else {
                "film_light"
            };
            CandidateValue::Str(alt.to_string())
        }
    }
}

fn valid_values_for_param(key: &str, rule: ParamRule, mode: &str) -> Vec<(String, CandidateValue)> {
    match rule {
        ParamRule::Enum(allowed) => allowed
            .iter()
            .copied()
            .map(|value| {
                (
                    format!("enum={value}"),
                    CandidateValue::Str(value.to_string()),
                )
            })
            .collect(),
        ParamRule::IntRange { min, max } => int_range_valid_values(key, min, max)
            .into_iter()
            .map(|value| (format!("int={value}"), CandidateValue::Int(value)))
            .collect(),
        ParamRule::Bool => vec![
            ("bool=false".to_string(), CandidateValue::Bool(false)),
            ("bool=true".to_string(), CandidateValue::Bool(true)),
        ],
        ParamRule::FreeString => vec![
            (
                "string=sample".to_string(),
                CandidateValue::Str("sample_value".to_string()),
            ),
            (
                "string=timecode".to_string(),
                CandidateValue::Str("00:00:00:00".to_string()),
            ),
        ],
        ParamRule::DataRate => params::bitrate_sets()
            .get(mode)
            .copied()
            .unwrap_or(&[])
            .iter()
            .copied()
            .map(|value| {
                (
                    format!("bitrate={value}"),
                    CandidateValue::Int(i64::from(value)),
                )
            })
            .collect(),
    }
}

fn int_range_valid_values(key: &str, min: i64, max: i64) -> Vec<i64> {
    let span = max - min;
    if span <= 128 {
        return (min..=max)
            .filter(|value| can_encode_int_override(key, *value))
            .collect();
    }

    let midpoint = min + span / 2;
    [min, min + 1, midpoint, max - 1, max]
        .into_iter()
        .filter(|value| *value >= min && *value <= max)
        .filter(|value| can_encode_int_override(key, *value))
        .collect()
}

fn invalid_values_for_param(
    key: &str,
    rule: ParamRule,
    mode: &str,
) -> Vec<(String, CandidateValue)> {
    match rule {
        ParamRule::Enum(_) => vec![(
            "enum=__invalid__".to_string(),
            CandidateValue::Str("__invalid__".to_string()),
        )],
        ParamRule::IntRange { min, max } => [("below", min - 1), ("above", max + 1)]
            .into_iter()
            .filter(|(_, value)| can_encode_int_override(key, *value))
            .map(|(label, value)| (label.to_string(), CandidateValue::Int(value)))
            .collect(),
        ParamRule::Bool => Vec::new(),
        ParamRule::FreeString => Vec::new(),
        ParamRule::DataRate => {
            let Some(allowed) = params::bitrate_sets().get(mode).copied() else {
                return Vec::new();
            };
            let mut cases = Vec::new();
            let min_allowed = allowed[0];
            if min_allowed > 0 {
                let below = min_allowed - 1;
                if !allowed.contains(&below) {
                    cases.push((
                        format!("below_min={below}"),
                        CandidateValue::Int(i64::from(below)),
                    ));
                }
            }

            if let Some(hole) = first_missing_bitrate(allowed) {
                cases.push((
                    format!("unsupported_gap={hole}"),
                    CandidateValue::Int(i64::from(hole)),
                ));
            }

            cases.push((
                format!("hard_max_plus_one={}", params::BITRATE_HARD_MAX + 1),
                CandidateValue::Int(i64::from(params::BITRATE_HARD_MAX + 1)),
            ));
            cases
        }
    }
}

fn first_missing_bitrate(allowed: &[u16]) -> Option<u16> {
    let min = *allowed.iter().min()?;
    let max = *allowed.iter().max()?;
    (min..=max).find(|candidate| !allowed.contains(candidate))
}

fn can_encode_int_override(key: &str, value: i64) -> bool {
    match key {
        "speech_threshold" => u8::try_from(value).is_ok(),
        "data_rate" => u16::try_from(value).is_ok(),
        "custom_dialnorm" => i8::try_from(value).is_ok(),
        _ => true,
    }
}

fn set_override(job: &mut JobFile, key: &str, value: CandidateValue) {
    match (key, value) {
        (
            "metering_mode"
            | "timecode_frame_rate"
            | "start"
            | "end"
            | "time_base"
            | "prepend_silence_duration"
            | "append_silence_duration"
            | "line_mode_drc_profile"
            | "rf_mode_drc_profile"
            | "loro_center_mix_level"
            | "loro_surround_mix_level"
            | "ltrt_center_mix_level"
            | "ltrt_surround_mix_level"
            | "preferred_downmix_mode"
            | "surround_trim_5_1"
            | "surround_trim_7_1"
            | "height_trim_5_1"
            | "encoding_backend"
            | "encoder_mode",
            CandidateValue::Str(v),
        ) => match key {
            "metering_mode" => job.filter.metering_mode = Some(v),
            "timecode_frame_rate" => job.filter.timecode_frame_rate = Some(v),
            "start" => job.filter.start = Some(v),
            "end" => job.filter.end = Some(v),
            "time_base" => job.filter.time_base = Some(v),
            "prepend_silence_duration" => job.filter.prepend_silence_duration = Some(v),
            "append_silence_duration" => job.filter.append_silence_duration = Some(v),
            "line_mode_drc_profile" => job.filter.line_mode_drc_profile = Some(v),
            "rf_mode_drc_profile" => job.filter.rf_mode_drc_profile = Some(v),
            "loro_center_mix_level" => job.filter.loro_center_mix_level = Some(v),
            "loro_surround_mix_level" => job.filter.loro_surround_mix_level = Some(v),
            "ltrt_center_mix_level" => job.filter.ltrt_center_mix_level = Some(v),
            "ltrt_surround_mix_level" => job.filter.ltrt_surround_mix_level = Some(v),
            "preferred_downmix_mode" => job.filter.preferred_downmix_mode = Some(v),
            "surround_trim_5_1" => job.filter.surround_trim_5_1 = Some(v),
            "surround_trim_7_1" => job.filter.surround_trim_7_1 = Some(v),
            "height_trim_5_1" => job.filter.height_trim_5_1 = Some(v),
            "encoding_backend" => job.filter.encoding_backend = Some(v),
            "encoder_mode" => job.filter.encoder_mode = Some(v),
            _ => unreachable!(),
        },
        ("dialogue_intelligence", CandidateValue::Bool(v)) => {
            job.filter.dialogue_intelligence = Some(v);
        }
        ("speech_threshold", CandidateValue::Int(v)) => {
            job.filter.speech_threshold = Some(
                u8::try_from(v)
                    .unwrap_or_else(|_| panic!("speech_threshold out of u8 range in test: {v}")),
            );
        }
        ("data_rate", CandidateValue::Int(v)) => {
            job.filter.data_rate = Some(
                u16::try_from(v)
                    .unwrap_or_else(|_| panic!("data_rate out of u16 range in test: {v}")),
            );
        }
        ("custom_dialnorm", CandidateValue::Int(v)) => {
            job.filter.custom_dialnorm = Some(
                i8::try_from(v)
                    .unwrap_or_else(|_| panic!("custom_dialnorm out of i8 range in test: {v}")),
            );
        }
        (other, value) => {
            panic!("unsupported override in matrix test: key={other}, value={value:?}")
        }
    }
}
